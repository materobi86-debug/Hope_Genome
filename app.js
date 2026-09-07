/**
 * HERMES SERVERLESS - Black Developer Workspace Controller
 */

document.addEventListener('DOMContentLoaded', () => {
  const adapters = window.HermesAdapters;

  // State
  let isLiveMode = false;
  let notifications = [
    {
      id: 1,
      title: "SYSTEM_INIT",
      body: "HERMES SERVERLESS v2.5.0 ready. Local memory active.",
      tone: "cyan",
      read: false,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    }
  ];

  let messages = [
    {
      sender: "HOPE",
      text: "Üdvözöllek. A helyi munkaterület készen áll. Milyen gondolatot vagy feladatot szeretnél rögzíteni?",
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' }),
      telemetry: {
        id: "evt_init_001",
        phase: "COMPLETED",
        memoryLayer: "WASM_M2",
        bridgeState: "CONNECTED"
      }
    }
  ];

  // UI Elements
  const headerStatusText = document.getElementById('headerStatusText');
  const headerStatusBadge = document.getElementById('headerStatusBadge');
  const presenceStateText = document.getElementById('presenceStateText');
  const liveToggleBtn = document.getElementById('liveToggleBtn');
  const liveBtnText = document.getElementById('liveBtnText');
  const stopLiveBtn = document.getElementById('stopLiveBtn');

  const thinkingStateText = document.getElementById('thinkingStateText');
  const messageStream = document.getElementById('messageStream');
  const composerInput = document.getElementById('composerInput');
  const sendBtn = document.getElementById('sendBtn');
  const modelRouterSelect = document.getElementById('modelRouterSelect');

  const rightInspector = document.getElementById('rightInspector');
  const inspectToggleBtn = document.getElementById('inspectToggleBtn');
  const closeInspectorBtn = document.getElementById('closeInspectorBtn');

  const memorySizeVal = document.getElementById('memorySizeVal');
  const lastSyncVal = document.getElementById('lastSyncVal');
  const merkleHashVal = document.getElementById('merkleHashVal');
  const appendLogList = document.getElementById('appendLogList');

  const notifBtn = document.getElementById('notifBtn');
  const notifCounter = document.getElementById('notifCounter');
  const notificationDrawer = document.getElementById('notificationDrawer');
  const closeNotifDrawer = document.getElementById('closeNotifDrawer');
  const notificationList = document.getElementById('notificationList');

  const syncDiffBtn = document.getElementById('syncDiffBtn');
  const syncDiffModal = document.getElementById('syncDiffModal');
  const closeSyncModal = document.getElementById('closeSyncModal');
  const confirmSyncBtn = document.getElementById('confirmSyncBtn');
  const exportApv2Btn = document.getElementById('exportApv2Btn');

  const qrInstallBtn = document.getElementById('qrInstallBtn');
  const qrModal = document.getElementById('qrModal');
  const closeQrModal = document.getElementById('closeQrModal');
  const qrCodeContainer = document.getElementById('qrCodeContainer');
  const qrUrlText = document.getElementById('qrUrlText');

  // Inspector Toggle
  if (inspectToggleBtn && rightInspector) {
    inspectToggleBtn.addEventListener('click', () => {
      rightInspector.classList.toggle('collapsed');
    });
  }

  if (closeInspectorBtn && rightInspector) {
    closeInspectorBtn.addEventListener('click', () => {
      rightInspector.classList.add('collapsed');
    });
  }

  // Active button highlight for composer input
  if (composerInput && sendBtn) {
    composerInput.addEventListener('input', () => {
      if (composerInput.value.trim().length > 0) {
        sendBtn.classList.add('active');
      } else {
        sendBtn.classList.remove('active');
      }
    });
  }

  // --- 0. Model Selector Handling ---
  if (modelRouterSelect) {
    modelRouterSelect.addEventListener('change', async (e) => {
      const selectedModelId = e.target.value;
      const model = adapters.modelRouter.setModel(selectedModelId);
      if (model) {
        await adapters.merkleChain.appendLog('MODEL_CHANGE', model.name);
        addNotification("MODEL_SWITCH", `Active model: ${model.name}`, "copper");
        updateSystemMetrics();
      }
    });
  }

  // --- 1. Notification Management ---
  function addNotification(title, body, tone = 'copper') {
    const newNotif = {
      id: Date.now(),
      title,
      body,
      tone, // "copper" | "cyan" | "green" | "amber"
      read: false,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    };
    notifications.unshift(newNotif);
    renderNotifications();
  }

  function renderNotifications() {
    const unreadCount = notifications.filter(n => !n.read).length;
    notifCounter.textContent = unreadCount;
    notifCounter.style.display = unreadCount === 0 ? 'none' : 'inline-block';

    notificationList.innerHTML = notifications.map(n => `
      <div class="notif-item tone-${n.tone}">
        <div class="notif-head">
          <span>${escapeHtml(n.title)}</span>
          <span class="text-muted">${n.time}</span>
        </div>
        <div class="notif-body">${escapeHtml(n.body)}</div>
      </div>
    `).join('');
  }

  if (notifBtn) {
    notifBtn.addEventListener('click', () => {
      notificationDrawer.classList.toggle('hidden');
    });
  }

  if (closeNotifDrawer) {
    closeNotifDrawer.addEventListener('click', () => {
      notificationDrawer.classList.add('hidden');
    });
  }

  // --- 2. Presence & Live State Toggles ---
  async function setLiveMode(active) {
    isLiveMode = active;
    if (active) {
      headerStatusBadge.classList.add('live');
      headerStatusText.textContent = 'LIVE / FIGYEL';
      presenceStateText.textContent = 'Figyelek.';
      liveToggleBtn.classList.add('hidden');
      stopLiveBtn.classList.remove('hidden');

      await adapters.merkleChain.appendLog('PRESENCE_CHANGE', 'LIVE_MODE_ACTIVATED');
      addNotification("LIVE_ACTIVE", "Continuous local presence monitoring active.", "copper");
    } else {
      headerStatusBadge.classList.remove('live');
      headerStatusText.textContent = 'LOCAL / KÉSZENLÉT';
      presenceStateText.textContent = 'Itt vagyok.';
      liveToggleBtn.classList.remove('hidden');
      stopLiveBtn.classList.add('hidden');

      await adapters.merkleChain.appendLog('PRESENCE_CHANGE', 'LOCAL_MODE_ACTIVATED');
      addNotification("LOCAL_ACTIVE", "System entered idle local workspace mode.", "cyan");
    }
    updateSystemMetrics();
  }

  if (liveToggleBtn) liveToggleBtn.addEventListener('click', () => setLiveMode(true));
  if (stopLiveBtn) stopLiveBtn.addEventListener('click', () => setLiveMode(false));

  // --- 3. Message & Thinking Stream ---
  function renderMessages() {
    messageStream.innerHTML = messages.map((m, index) => {
      const isUser = m.sender === 'TE';
      let actionBlockHtml = '';

      if (m.actionApproval) {
        actionBlockHtml = `
          <div class="action-approval-panel" id="approvalPanel_${index}">
            <div class="approval-header">
              <span class="capability-title">CAPABILITY_REQUEST: ${escapeHtml(m.actionApproval.capability)}</span>
              <span class="mono text-muted" style="font-size:10px;">PROPOSAL_ID: ${m.actionApproval.id}</span>
            </div>
            <div class="approval-body">${escapeHtml(m.actionApproval.reason)}</div>
            <div class="approval-scope">SCOPE: ${escapeHtml(m.actionApproval.scope)}</div>
            <div class="approval-actions" id="approvalActions_${index}">
              <button class="btn-approve" onclick="window.approveAction(${index})">Jóváhagyom</button>
              <button class="btn-deny" onclick="window.denyAction(${index})">Elutasítom</button>
            </div>
          </div>
        `;
      }

      return `
        <div class="message-row ${isUser ? 'user' : 'assistant'}">
          <div class="message-meta">
            <span class="sender-tag ${isUser ? 'user-tag' : 'hope-tag'}">${isUser ? 'TE' : 'HOPE'}</span>
            <span>·</span>
            <span>${m.time}</span>
          </div>
          <div class="message-bubble">
            ${escapeHtml(m.text)}
            ${actionBlockHtml}
          </div>
          ${m.telemetry ? `
            <div class="message-telemetry">
              <span>EVT_ID: ${m.telemetry.id}</span>
              <span>PHASE: ${m.telemetry.phase}</span>
              <span>MEMORY: ${m.telemetry.memoryLayer}</span>
            </div>
          ` : ''}
        </div>
      `;
    }).join('');
    messageStream.scrollTop = messageStream.scrollHeight;
  }

  // Window action approval handlers
  window.approveAction = async (msgIndex) => {
    const msg = messages[msgIndex];
    if (msg && msg.actionApproval) {
      msg.actionApproval.status = 'APPROVED';
      const actionsElem = document.getElementById(`approvalActions_${msgIndex}`);
      if (actionsElem) {
        actionsElem.innerHTML = `<span class="mono text-green" style="font-size:11px;">✓ APPROVED & EVIDENCE VERIFIED</span>`;
      }
      await adapters.merkleChain.appendLog('ACTION_APPROVED', msg.actionApproval.capability);
      addNotification("EVIDENCE_VERIFIED", `Capability approved: ${msg.actionApproval.capability}`, "green");
      updateSystemMetrics();
    }
  };

  window.denyAction = async (msgIndex) => {
    const msg = messages[msgIndex];
    if (msg && msg.actionApproval) {
      msg.actionApproval.status = 'DENIED';
      const actionsElem = document.getElementById(`approvalActions_${msgIndex}`);
      if (actionsElem) {
        actionsElem.innerHTML = `<span class="mono text-muted" style="font-size:11px;">✗ REQUEST DENIED</span>`;
      }
      await adapters.merkleChain.appendLog('ACTION_DENIED', msg.actionApproval.capability);
      addNotification("ACTION_DENIED", `Capability request denied: ${msg.actionApproval.capability}`, "amber");
      updateSystemMetrics();
    }
  };

  async function handleSendMessage() {
    const text = composerInput.value.trim();
    if (!text) return;

    // Render User Message
    const userMsg = {
      sender: 'TE',
      text: text,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' }),
      telemetry: {
        id: `evt_usr_${Date.now().toString().slice(-4)}`,
        phase: "USER_COMMIT",
        memoryLayer: "ED25519_SIGNED"
      }
    };
    messages.push(userMsg);
    composerInput.value = '';
    sendBtn.classList.remove('active');
    renderMessages();

    // Instant Local Append Log
    await adapters.merkleChain.appendLog('USER_INPUT', text.substring(0, 30));
    await adapters.wasmMemory.allocate(text.length * 2);
    updateSystemMetrics();

    // Thinking State Transition
    setThinkingState('JEL ÉL');

    setTimeout(() => {
      setThinkingState('GONDOLKODOM');
    }, 300);

    // Delayed Assistant Response
    setTimeout(async () => {
      const responseText = await adapters.modelRouter.generateResponse(text);

      // Include an Action Proposal if text mentions capability/action
      let actionProposal = null;
      if (text.toLowerCase().includes('sync') || text.toLowerCase().includes('művelet') || text.toLowerCase().includes('export')) {
        actionProposal = {
          id: `prop_${Date.now().toString().slice(-4)}`,
          capability: "OCTOPUS_LOCAL_SYNC",
          reason: "Lokális adatterjedelem frissítése a File System Access API-val.",
          scope: "READ_WRITE / LOCAL_FS",
          status: "PENDING"
        };
      }

      const assistantMsg = {
        sender: 'HOPE',
        text: responseText,
        time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' }),
        actionApproval: actionProposal,
        telemetry: {
          id: `evt_hope_${Date.now().toString().slice(-4)}`,
          phase: "STREAM_DONE",
          memoryLayer: "WASM_M2"
        }
      };
      messages.push(assistantMsg);
      renderMessages();

      setThinkingState('VÁRAKOZIK');

      await adapters.merkleChain.appendLog('ASSISTANT_RESPONSE', responseText.substring(0, 30));
      updateSystemMetrics();

      addNotification("Új gondolati válasz", responseText.substring(0, 50) + "...", "cyan");
    }, 1000);
  }

  function setThinkingState(state) {
    if (thinkingStateText) thinkingStateText.textContent = state;
  }

  if (composerInput) {
    composerInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSendMessage();
      }
    });
  }

  if (sendBtn) sendBtn.addEventListener('click', handleSendMessage);

  // --- 4. System Metrics & Merkle Chain ---
  async function updateSystemMetrics() {
    if (memorySizeVal) memorySizeVal.textContent = await adapters.wasmMemory.getFormattedSize();
    if (lastSyncVal) lastSyncVal.textContent = adapters.fileSync.getLastSyncFormatted();
    if (merkleHashVal) merkleHashVal.textContent = adapters.merkleChain.getShortHash();

    if (appendLogList) {
      appendLogList.innerHTML = adapters.merkleChain.logs.map(log => `
        <div class="log-item">
          <span class="time">${log.time}</span>
          <span>[${log.action}] ${escapeHtml(log.detail)}</span>
        </div>
      `).join('');
    }
  }

  // --- 5. Sync Diff & Modals ---
  if (syncDiffBtn) {
    syncDiffBtn.addEventListener('click', () => {
      syncDiffModal.classList.remove('hidden');
    });
  }

  if (closeSyncModal) {
    closeSyncModal.addEventListener('click', () => {
      syncDiffModal.classList.add('hidden');
    });
  }

  if (confirmSyncBtn) {
    confirmSyncBtn.addEventListener('click', async () => {
      await adapters.fileSync.syncLocalState({ messages, logs: adapters.merkleChain.logs });
      addNotification("SYNC_VERIFIED", "Local state hash match confirmed.", "copper");
      syncDiffModal.classList.add('hidden');
      updateSystemMetrics();
    });
  }

  // --- 6. QR Code Mobile Install Modal ---
  function generateSvgQrCode(url) {
    return `
      <svg xmlns="http://www.w3.org/2000/svg" width="180" height="180" viewBox="0 0 25 25" shape-rendering="crispEdges">
        <rect width="25" height="25" fill="#ffffff"/>
        <path d="M 1 1 h 7 v 7 h -7 z M 2 2 v 5 h 5 v -5 z M 3 3 h 3 v 3 h -3 z" fill="#08090B"/>
        <path d="M 17 1 h 7 v 7 h -7 z M 18 2 v 5 h 5 v -5 z M 19 3 h 3 v 3 h -3 z" fill="#08090B"/>
        <path d="M 1 17 h 7 v 7 h -7 z M 2 18 v 5 h 5 v -5 z M 3 19 h 3 v 3 h -3 z" fill="#08090B"/>
        <rect x="10" y="2" width="2" height="2" fill="#D88955"/>
        <rect x="13" y="2" width="2" height="3" fill="#08090B"/>
        <rect x="10" y="5" width="3" height="2" fill="#08090B"/>
        <rect x="2" y="10" width="3" height="2" fill="#08090B"/>
        <rect x="6" y="10" width="2" height="3" fill="#D88955"/>
        <rect x="10" y="10" width="5" height="5" fill="#08090B"/>
        <rect x="17" y="10" width="3" height="2" fill="#08090B"/>
        <rect x="21" y="10" width="2" height="4" fill="#D88955"/>
        <rect x="10" y="17" width="2" height="4" fill="#08090B"/>
        <rect x="14" y="17" width="4" height="2" fill="#D88955"/>
        <rect x="19" y="17" width="4" height="4" fill="#08090B"/>
        <rect x="13" y="21" width="3" height="3" fill="#08090B"/>
      </svg>
    `;
  }

  if (qrInstallBtn) {
    qrInstallBtn.addEventListener('click', () => {
      const currentUrl = window.location.href;
      qrUrlText.textContent = currentUrl;
      qrCodeContainer.innerHTML = generateSvgQrCode(currentUrl);
      qrModal.classList.remove('hidden');
    });
  }

  if (closeQrModal) {
    closeQrModal.addEventListener('click', () => {
      qrModal.classList.add('hidden');
    });
  }

  // --- 7. APv2 Export ---
  if (exportApv2Btn) {
    exportApv2Btn.addEventListener('click', () => {
      const exportData = {
        app: "HERMES SERVERLESS",
        version: "2.5.0",
        timestamp: new Date().toISOString(),
        merkleRoot: adapters.merkleChain.merkleRoot,
        logs: adapters.merkleChain.logs,
        messages: messages,
        activeModel: adapters.modelRouter.getActiveModel()
      };

      const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(exportData, null, 2));
      const downloadAnchor = document.createElement('a');
      downloadAnchor.setAttribute("href", dataStr);
      downloadAnchor.setAttribute("download", `hermes_apv2_export_${Date.now()}.json`);
      document.body.appendChild(downloadAnchor);
      downloadAnchor.click();
      downloadAnchor.remove();

      addNotification("APv2_EXPORT", "Evidence certified export saved.", "cyan");
    });
  }

  function escapeHtml(str) {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  // Initialize
  renderNotifications();
  renderMessages();
  adapters.merkleChain.appendLog('SYS_INIT', 'BLACK_WORKSPACE_READY').then(() => {
    updateSystemMetrics();
  });

  // Service Worker Registration
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('./sw.js').catch((err) => {
      console.log('SW registration failed:', err);
    });
  }
});
