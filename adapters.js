/**
 * HERMES SERVERLESS - Modular Adapter Architecture
 * Clean abstraction layer for WASM memory, File System Access API, and Model Router.
 */

// 1. WASM Memory Adapter Interface & Mock Implementation
class WasmMemoryAdapter {
  constructor() {
    this.memorySize = 131072; // 128 KB
    this.initialized = true;
  }

  async getMemorySize() {
    return this.memorySize;
  }

  async allocate(bytes) {
    this.memorySize += bytes;
    return this.memorySize;
  }

  async getFormattedSize() {
    const kb = (this.memorySize / 1024).toFixed(1);
    return `${kb} KB`;
  }
}

// 2. File System Sync Adapter Interface & Mock Implementation
class FileSystemSyncAdapter {
  constructor() {
    this.lastSyncTimestamp = new Date();
    this.directoryHandle = null;
    this.isSupported = 'showDirectoryPicker' in window;
  }

  async syncLocalState(payload) {
    this.lastSyncTimestamp = new Date();
    // Simulate File System Access API write
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

// 3. Model Router Adapter Interface & Mock Implementation
class ModelRouterAdapter {
  constructor() {
    this.mode = 'LOCAL_ONLY';
  }

  async generateResponse(prompt) {
    // Simulate local inference delay
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
}

// 4. Merkle Chain & Cryptographic Log Manager
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
    this.logs.unshift(logItem); // Newest first for live display

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

// Global Adapter Instances
window.HermesAdapters = {
  wasmMemory: new WasmMemoryAdapter(),
  fileSync: new FileSystemSyncAdapter(),
  modelRouter: new ModelRouterAdapter(),
  merkleChain: new MerkleLogChain()
};
