/**
 * HERMES SERVERLESS - Modular Adapter Architecture
 * Clean abstraction layer for WASM memory, File System Access API, Model Router, Spine Event Bus, Octopus Policy Engine, and OPFS/IndexedDB Persistence.
 */

// 1. WASM Memory Adapter Interface (Microscope Memory v2 compatible)
class WasmMemoryAdapter {
  constructor() {
    this.memorySize = 131072; // 128 KB base
    this.initialized = true;
    this.moduleName = "Microscope Memory v2 (WASM)";
    this.recalledBlocks = [];
  }

  async getMemorySize() {
    return this.memorySize;
  }

  async allocate(bytes) {
    this.memorySize += bytes;
    return this.memorySize;
  }

  async recall(query, k = 3) {
    this.recalledBlocks = [
      { id: "mem_01", text: "Emberi autonómia védelme a legfőbb szabály.", layer: "WASM_M2", importance: 0.9 },
      { id: "mem_02", text: "Minden művelet előtt explicit jóváhagyás szükséges.", layer: "WASM_M2", importance: 0.85 }
    ];
    return this.recalledBlocks;
  }

  async store(text, layer = "WASM_M2", importance = 0.8) {
    await this.allocate(text.length * 2);
    return {
      confirmed: true,
      text,
      layer,
      importance,
      timestamp: new Date().toISOString()
    };
  }

  async getFormattedSize() {
    const kb = (this.memorySize / 1024).toFixed(1);
    return `${kb} KB`;
  }

  getModuleName() {
    return this.moduleName;
  }
}

// 2. OPFS & IndexedDB Persistence Manager
class OpfsWorkspaceStore {
  constructor() {
    this.dbName = "HermesSessionDB";
    this.dbVersion = 1;
    this.db = null;
    this.initDb();
  }

  async initDb() {
    if (!('indexedDB' in window)) return;
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.dbVersion);
      request.onupgradeneeded = (e) => {
        const db = e.target.result;
        if (!db.objectStoreNames.contains('snapshots')) {
          db.createObjectStore('snapshots', { keyPath: 'stateHash' });
        }
        if (!db.objectStoreNames.contains('audit_logs')) {
          db.createObjectStore('audit_logs', { keyPath: 'id', autoIncrement: true });
        }
      };
      request.onsuccess = (e) => {
        this.db = e.target.result;
        resolve(this.db);
      };
      request.onerror = (e) => reject(e);
    });
  }

  async saveSnapshot(snapshot) {
    if (!this.db) await this.initDb();
    if (!this.db) return;
    return new Promise((resolve, reject) => {
      const tx = this.db.transaction('snapshots', 'readwrite');
      const store = tx.objectStore('snapshots');
      store.put(snapshot);
      tx.oncomplete = () => resolve(true);
      tx.onerror = () => reject(false);
    });
  }

  async saveOpfsBinary(filename, content) {
    if ('navigator' in window && 'storage' in navigator && 'getDirectory' in navigator.storage) {
      try {
        const root = await navigator.storage.getDirectory();
        const fileHandle = await root.getFileHandle(filename, { create: true });
        const writable = await fileHandle.createWritable();
        await writable.write(content);
        await writable.close();
        return true;
      } catch (err) {
        console.warn("[OpfsWorkspaceStore] OPFS write error:", err);
      }
    }
    return false;
  }
}

// 3. File System Sync Adapter Interface
class FileSystemSyncAdapter {
  constructor() {
    this.lastSyncTimestamp = new Date();
    this.directoryHandle = null;
    this.isSupported = 'showDirectoryPicker' in window;
  }

  async syncLocalState(payload) {
    this.lastSyncTimestamp = new Date();
    return {
      success: true,
      timestamp: this.lastSyncTimestamp.toISOString(),
      bytesWritten: JSON.stringify(payload).length
    };
  }

  getLastSyncFormatted() {
    const secondsAgo = Math.floor((new Date() - this.lastSyncTimestamp) / 1000);
    if (secondsAgo < 5) return 'Most';
    if (secondsAgo < 60) return `${secondsAgo} mp-e`;
    const minsAgo = Math.floor(secondsAgo / 60);
    return `${minsAgo} perc-e`;
  }
}

// 4. Model Router Adapter Interface
class ModelRouterAdapter {
  constructor() {
    this.mode = 'LOCAL_ONLY';
    this.engineName = 'Octopus Runtime (Local Router)';
    this.activeModel = 'opencode-zen-free';
    this.availableModels = [
      { id: 'opencode-zen-free', name: 'OpenCode Zen (Ingyenes / Local)', type: 'Free' },
      { id: 'hope-local-v2', name: 'HOPE Local Core v2', type: 'Local' }
    ];
  }

  setModel(modelId) {
    const model = this.availableModels.find(m => m.id === modelId);
    if (model) {
      this.activeModel = model.id;
      return model;
    }
    return null;
  }

  getActiveModel() {
    return this.availableModels.find(m => m.id === this.activeModel);
  }

  async generateResponse(prompt) {
    await new Promise((resolve) => setTimeout(resolve, 800));

    const responses = [
      "A gondolatmenet rögzítésre került a lokális láncon. Nincs szükség külső beavatkozásra.",
      "A megadott kontextus alapján az emberi autonómia megőrzése az elsődleges lépés.",
      "A lokális memória állapota szinkronban van. Minden etikai szabályzat ellenőrizve.",
      "Elemzés kész: Az állítás nem tartalmaz külső hálózati függőségeket.",
      "Érintettség nélkül feldolgozva. A döntési opciók nyitva állnak az operátor számára."
    ];

    const randomIndex = Math.floor(Math.random() * responses.length);
    return responses[randomIndex];
  }

  getEngineName() {
    return this.engineName;
  }
}

// 5. Merkle Chain & Cryptographic Log Manager
class MerkleLogChain {
  constructor() {
    this.logs = [];
    this.merkleRoot = "a3f8901b2c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d99e1";
  }

  async computeHash(data) {
    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  async appendLog(action, detail) {
    const timestamp = new Date().toLocaleTimeString('hu-HU', { hour12: false });
    const fullTimeString = new Date().toISOString();
    const logItem = {
      id: this.logs.length + 1,
      time: timestamp,
      fullTime: fullTimeString,
      action: action,
      detail: detail
    };

    const newHashPart = await this.computeHash(JSON.stringify(logItem) + this.merkleRoot);
    this.merkleRoot = newHashPart;
    this.logs.unshift(logItem);

    return {
      logItem,
      merkleRoot: this.merkleRoot
    };
  }

  getShortHash() {
    if (!this.merkleRoot) return "0x00";
    return `${this.merkleRoot.substring(0, 8)}...${this.merkleRoot.substring(this.merkleRoot.length - 4)}`;
  }
}

// 6. Typed Browser Spine Event Bus
class BrowserSpineBus {
  constructor() {
    this.listeners = [];
    this.eventHistory = [];
    this.seenEventIds = new Set();
  }

  subscribe(callback) {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter(cb => cb !== callback);
    };
  }

  emit(envelope) {
    if (this.seenEventIds.has(envelope.eventId)) {
      return;
    }
    this.seenEventIds.add(envelope.eventId);
    this.eventHistory.push(envelope);

    this.listeners.forEach(cb => cb(envelope));
  }
}

// 7. Browser Octopus Policy Engine
class BrowserOctopusPolicy {
  constructor() {
    this.allowedCapabilities = new Set([
      "opfs.read",
      "opfs.write",
      "file.pick",
      "crypto.hash",
      "data.export",
      "wasm.run"
    ]);

    this.unsupportedCapabilities = new Set([
      "process.spawn",
      "shell.execute",
      "systemd.control",
      "native.mmap",
      "arbitrary.filesystem"
    ]);
  }

  evaluateCapability(capabilityName) {
    if (this.unsupportedCapabilities.has(capabilityName)) {
      return {
        allowed: false,
        reason: "The browser sandbox does not expose arbitrary OS process/native capability execution.",
        code: "unsupported_in_browser"
      };
    }

    if (this.allowedCapabilities.has(capabilityName)) {
      return {
        allowed: true,
        requiresApproval: true,
        code: "requires_user_approval"
      };
    }

    return {
      allowed: false,
      reason: "Capability is not registered in the Browser Octopus Policy matrix.",
      code: "capability_denied"
    };
  }
}

// Global Adapter Instances
window.HermesAdapters = {
  wasmMemory: new WasmMemoryAdapter(),
  opfsWorkspace: new OpfsWorkspaceStore(),
  fileSync: new FileSystemSyncAdapter(),
  modelRouter: new ModelRouterAdapter(),
  merkleChain: new MerkleLogChain(),
  spineBus: new BrowserSpineBus(),
  policy: new BrowserOctopusPolicy()
};
