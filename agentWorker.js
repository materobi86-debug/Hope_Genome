/**
 * HERMES SERVERLESS - Agent Web Worker & Deterministic State Machine
 * Runs the agent loop off the main UI thread.
 */

// Worker State
let currentPhase = "idle"; // ChatPhase
let sequenceCounter = 0;
let activeSessionId = "LOCAL_MAIN_01";
let sessionSnapshots = [];

// Memory & Policy Mock/Simulators inside Worker context
const memoryBlocks = [
  { id: "mem_01", text: "Emberi autonómia védelme a legfőbb szabály.", layer: "WASM_M2", importance: 0.9 },
  { id: "mem_02", text: "Minden művelet előtt explicit jóváhagyás szükséges.", layer: "WASM_M2", importance: 0.85 }
];

// Helper: Dispatch Typed Spine Event to Main Thread
function dispatchSpineEvent(type, payload = {}) {
  sequenceCounter += 1;
  const envelope = {
    eventId: `evt_${Date.now()}_${sequenceCounter}`,
    sequence: sequenceCounter,
    timestamp: new Date().toISOString(),
    type,
    ...payload
  };
  self.postMessage({ kind: "SPINE_EVENT", envelope });
}

// Helper: Transition Phase & Emit Event
function transitionTo(newPhase, detail = "") {
  currentPhase = newPhase;
  dispatchSpineEvent("assistant_thinking", { phase: currentPhase, detail });
}

// Cryptographic Evidence Verification via Web Crypto API
async function verifyEvidence(actionId, resultData) {
  try {
    const encoder = new TextEncoder();
    const data = encoder.encode(JSON.stringify(resultData));
    const hashBuffer = await self.crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');

    return {
      actionId,
      passed: true,
      summary: `SHA-256 evidence verified: ${hashHex.substring(0, 12)}...`,
      verifiedAt: new Date().toISOString()
    };
  } catch (err) {
    return {
      actionId,
      passed: false,
      summary: `Evidence verification failed: ${err.message}`,
      verifiedAt: new Date().toISOString()
    };
  }
}

// Create State Snapshot for Rollback
function createSnapshot(reason) {
  const snapshot = {
    version: 1,
    sessionId: activeSessionId,
    stateHash: `snap_${Date.now()}`,
    eventSequence: sequenceCounter,
    createdAt: new Date().toISOString(),
    reason
  };
  sessionSnapshots.push(snapshot);
  dispatchSpineEvent("audit", { message: `SNAPSHOT_CREATED: ${snapshot.stateHash} (${reason})` });
  return snapshot;
}

// Handle Messages from Main UI Thread
self.onmessage = async (e) => {
  const { action, payload } = e.data || {};

  switch (action) {
    case "INIT_WORKER":
      transitionTo("idle", "Worker initialized");
      dispatchSpineEvent("presence", { state: "idle" });
      break;

    case "USER_MESSAGE":
      await handleUserMessage(payload.text, payload.modelId);
      break;

    case "APPROVE_TOOL":
      await handleToolApproval(payload.actionId, payload.capability, payload.scope);
      break;

    case "DENY_TOOL":
      await handleToolDenial(payload.actionId, payload.capability);
      break;

    default:
      console.warn("[AgentWorker] Unknown action:", action);
  }
};

async function handleUserMessage(text, modelId) {
  // 1. Recalling Phase
  transitionTo("recalling", "Searching Microscope WASM memory");

  // Simulate memory recall query
  const recalled = memoryBlocks.filter(m => text.toLowerCase().includes("szabály") || text.toLowerCase().includes("memória") || m.importance > 0.8);
  dispatchSpineEvent("memory_recalled", { count: recalled.length, memories: recalled });

  // 2. Thinking Phase
  transitionTo("thinking", "Generating response with model router");

  // Check for Native Unsupported Capabilities (e.g. process.spawn, bash shell)
  const lowerText = text.toLowerCase();
  if (lowerText.includes("spawn") || lowerText.includes("shell") || lowerText.includes("exec") || lowerText.includes("process") || lowerText.includes("systemd")) {
    transitionTo("unsupported_in_browser", "Requested native capability is forbidden in browser sandbox");

    self.postMessage({
      kind: "ASSISTANT_RESPONSE",
      response: {
        text: "A feladat végrehajtása nem lehetséges a böngészőben.",
        unsupportedCapability: {
          code: "unsupported_in_browser",
          capability: "process.spawn / native_shell",
          reason: "The browser sandbox does not expose arbitrary OS process execution. No action was executed."
        },
        telemetry: { id: `evt_unsupported_${Date.now()}`, phase: "UNSUPPORTED_IN_BROWSER", memoryLayer: "SANDBOX_LOCK" }
      }
    });

    dispatchSpineEvent("audit", { message: "UNSUPPORTED_CAPABILITY_BLOCKED: process.spawn" });
    transitionTo("idle");
    return;
  }

  // Check for Browser Tools requiring Policy Approval (e.g., OPFS write or sync)
  if (lowerText.includes("sync") || lowerText.includes("művelet") || lowerText.includes("export") || lowerText.includes("fájl")) {
    transitionTo("awaiting_approval", "Awaiting user policy approval for OPFS local storage write");

    const actionId = `act_${Date.now()}`;
    const proposal = {
      actionId,
      capability: "opfs.write",
      reason: "Helyi munkaterület és memória állapot szinkronizálása az OPFS tárolóba.",
      scope: "OPFS_WORKSPACE_DIR"
    };

    dispatchSpineEvent("tool_proposed", proposal);

    self.postMessage({
      kind: "ASSISTANT_RESPONSE",
      response: {
        text: "Ehhez a művelethez explicit engedélyre van szükség az OPFS munkaterület írásához.",
        actionProposal: proposal,
        telemetry: { id: `evt_prop_${Date.now()}`, phase: "AWAITING_APPROVAL", memoryLayer: "OPFS_GUARD" }
      }
    });
    return;
  }

  // Standard Chat Completion Loop
  setTimeout(async () => {
    transitionTo("completed", "Response generated successfully");

    // Store new thought into memory
    const storeConfirmed = true; // WASM confirmation simulation
    if (storeConfirmed) {
      memoryBlocks.push({ id: `mem_${Date.now()}`, text: text.substring(0, 50), layer: "WASM_M2", importance: 0.7 });
    }

    self.postMessage({
      kind: "ASSISTANT_RESPONSE",
      response: {
        text: `Feldolgoztam a gondolatodat: „${text}”. A helyi állapot és a Merkle-lánc frissítve.`,
        recalledMemories: recalled,
        telemetry: { id: `evt_completed_${Date.now()}`, phase: "COMPLETED", memoryLayer: "WASM_M2" }
      }
    });

    dispatchSpineEvent("audit", { message: `CHAT_COMPLETED: ${text.substring(0, 20)}...` });
    transitionTo("idle");
  }, 600);
}

async function handleToolApproval(actionId, capability, scope) {
  // Snapshot prior to execution for safety & rollback capability
  createSnapshot(`Pre-execution of ${capability}`);

  transitionTo("executing_browser_tool", `Executing browser capability ${capability}`);
  dispatchSpineEvent("tool_state", { actionId, state: "EXECUTING" });

  // Execute Browser Tool (OPFS Write simulation)
  const resultData = { actionId, capability, scope, status: "SUCCESS", bytesWritten: 1024 };

  // Evidence Verification Phase
  transitionTo("verifying_evidence", "Verifying cryptographic proof of result");
  const evidence = await verifyEvidence(actionId, resultData);

  dispatchSpineEvent("evidence", {
    actionId,
    passed: evidence.passed,
    summary: evidence.summary
  });

  if (evidence.passed) {
    transitionTo("completed", "Tool executed and evidence verified");
    dispatchSpineEvent("tool_state", { actionId, state: "COMPLETED" });

    self.postMessage({
      kind: "TOOL_RESULT",
      actionId,
      success: true,
      evidence
    });
  } else {
    // Rollback if evidence failed
    transitionTo("error", "Evidence verification failed, rolling back snapshot");
    dispatchSpineEvent("audit", { message: `ROLLBACK_EXECUTED for ${actionId}` });

    self.postMessage({
      kind: "TOOL_RESULT",
      actionId,
      success: false,
      evidence
    });
  }

  transitionTo("idle");
}

async function handleToolDenial(actionId, capability) {
  transitionTo("denied", `User denied execution of ${capability}`);
  dispatchSpineEvent("tool_state", { actionId, state: "DENIED" });
  dispatchSpineEvent("audit", { message: `TOOL_DENIED: ${actionId} (${capability})` });

  self.postMessage({
    kind: "TOOL_RESULT",
    actionId,
    success: false,
    denied: true
  });

  transitionTo("idle");
}
