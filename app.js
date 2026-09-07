/**
 * HERMES SERVERLESS - Main Application Controller
 */

document.addEventListener('DOMContentLoaded', () => {
  const adapters = window.HermesAdapters;

  // State
  let isLiveMode = false;
  let notifications = [
    {
      id: 1,
      title: "Rendszer elindítva",
      body: "HERMES SERVERLESS v2.5.0 készenlétben. Lokális memória aktív.",
      tone: "cyan",
      read: false,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    }
  ];

  let messages = [
    {
      sender: "HOPE",
      text: "Üdvözöllek. A helyi memória készen áll. Milyen gondolatot szeretnél rögzíteni vagy feldolgozni?",
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    }
  ];

  // UI Elements
  const headerStatusText = document.getElementById('headerStatusText');
  const headerStatusBadge = document.getElementById('headerStatusBadge');
  const presencePanel = document.getElementById('presencePanel');
  const presenceStateText = document.getElementById('presenceStateText');
  const liveToggleBtn = document.getElementById('liveToggleBtn');
  const liveBtnText = document.getElementById('liveBtnText');
  const stopLiveBtn = document.getElementById('stopLiveBtn');

  const thinkingStateBadge = document.getElementById('thinkingStateBadge');
  const thinkingStateText = document.getElementById('thinkingStateText');
  const waveformContainer = document.getElementById('waveformContainer');
  const messageStream = document.getElementById('messageStream');
  const composerInput = document.getElementById('composerInput');
  const sendBtn = document.getElementById('sendBtn');

  const memorySizeVal = document.getElementById('memorySizeVal');
  const memoryAdapterText = document.getElementById('memoryAdapterText');
  const lastSyncVal = document.getElementById('lastSyncVal');
  const merkleHashVal = document.getElementById('merkleHashVal');
  const appendLogList = document.getElementById('appendLogList');

  const notifBtn = document.getElementById('notifBtn');
  const notifCounter = document.getElementById('notifCounter');
  const notificationDrawer = document.getElementById('notificationDrawer');
  const closeNotifDrawer = document.getElementById('closeNotifDrawer');
  const notificationList = document.getElementById('notificationList');
  const clearNotifsBtn = document.getElementById('clearNotifsBtn');

  const syncDiffBtn = document.getElementById('syncDiffBtn');
  const syncDiffModal = document.getElementById('syncDiffModal');
  const closeSyncModal = document.getElementById('closeSyncModal');
  const confirmSyncBtn = document.getElementById('confirmSyncBtn');
  const exportApv2Btn = document.getElementById('exportApv2Btn');

  // --- 1. Notification Management ---
  function addNotification(title, body, tone = 'copper') {
    const newNotif = {
      id: Date.now(),
      title,
      body,
      tone, // "copper" | "cyan" | "muted"
      read: false,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    };
    notifications.unshift(newNotif);
    renderNotifications();
  }

  function renderNotifications() {
    const unreadCount = notifications.filter(n => !n.read).length;
    notifCounter.textContent = unreadCount;
    if (unreadCount === 0) {
      notifCounter.style.display = 'none';
    } else {
      notifCounter.style.display = 'block';
    }

    notificationList.innerHTML = notifications.map(n => `
      <div class="notif-card tone-${n.tone}">
        <div class="notif-card-header">
          <span class="notif-card-title">${escapeHtml(n.title)}</span>
          <span class="notif-card-time mono">${n.time}</span>
        </div>
        <div class="notif-card-body">${escapeHtml(n.body)}</div>
      </div>
    `).join('');
  }

  notifBtn.addEventListener('click', () => {
    notificationDrawer.classList.toggle('hidden');
  });

  closeNotifDrawer.addEventListener('click', () => {
    notificationDrawer.classList.add('hidden');
  });

  clearNotifsBtn.addEventListener('click', () => {
    notifications.forEach(n => n.read = true);
    renderNotifications();
  });

  // --- 2. Presence & Live State Toggles ---
  async function setLiveMode(active) {
    isLiveMode = active;
    if (active) {
      presencePanel.classList.add('live');
      headerStatusBadge.classList.add('live');
      headerStatusText.textContent = 'LIVE / FIGYEL';
      presenceStateText.textContent = 'Figyelek.';
      liveToggleBtn.classList.add('hidden');
      stopLiveBtn.classList.remove('hidden');

      await adapters.merkleChain.appendLog('PRESENCE_CHANGE', 'LIVE_MODE_ACTIVATED');
      addNotification("LIVE mód aktív", "A társ figyel és készen áll a folyamatos visszajelzésre.", "copper");
    } else {
      presencePanel.classList.remove('live');
      headerStatusBadge.classList.remove('live');
      headerStatusText.textContent = 'LOCAL / KÉSZENLÉT';
      presenceStateText.textContent = 'Itt vagyok.';
      liveToggleBtn.classList.remove('hidden');
      stopLiveBtn.classList.add('hidden');

      await adapters.merkleChain.appendLog('PRESENCE_CHANGE', 'LOCAL_MODE_ACTIVATED');
      addNotification("LOCAL mód aktív", "A rendszer várakozási állapotba lépett.", "cyan");
    }
    updateSystemMetrics();
  }

  liveToggleBtn.addEventListener('click', () => setLiveMode(true));
  stopLiveBtn.addEventListener('click', () => setLiveMode(false));

  // --- 3. Message & Thinking Stream ---
  function renderMessages() {
    messageStream.innerHTML = messages.map(m => `
      <div class="message-item ${m.sender === 'TE' ? 'user' : 'assistant'}">
        <div class="msg-header">
          <span class="msg-sender mono">${m.sender === 'TE' ? 'TE' : 'HOPE'}</span>
          <span class="msg-time mono">${m.time}</span>
        </div>
        <div class="msg-body">${escapeHtml(m.text)}</div>
      </div>
    `).join('');
    messageStream.scrollTop = messageStream.scrollHeight;
  }

  async function handleSendMessage() {
    const text = composerInput.value.trim();
    if (!text) return;

    // 1. Render User Message immediately
    const userMsg = {
      sender: 'TE',
      text: text,
      time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
    };
    messages.push(userMsg);
    composerInput.value = '';
    renderMessages();

    // 2. Instant Local Append Log
    await adapters.merkleChain.appendLog('USER_INPUT', text.substring(0, 30));
    await adapters.wasmMemory.allocate(text.length * 2);
    updateSystemMetrics();

    // 3. Thinking Block State Transition Sequence
    setThinkingState('JEL ÉL');
    waveformContainer.classList.add('active');

    setTimeout(() => {
      setThinkingState('GONDOLKODOM');
    }, 400);

    // 4. Delayed Assistant Response
    setTimeout(async () => {
      const responseText = await adapters.modelRouter.generateResponse(text);
      const assistantMsg = {
        sender: 'HOPE',
        text: responseText,
        time: new Date().toLocaleTimeString('hu-HU', { hour: '2-digit', minute: '2-digit' })
      };
      messages.push(assistantMsg);
      renderMessages();

      setThinkingState('VÁRAKOZIK');
      waveformContainer.classList.remove('active');

      await adapters.merkleChain.appendLog('ASSISTANT_RESPONSE', responseText.substring(0, 30));
      updateSystemMetrics();

      // Custom notification requirement
      addNotification("Új gondolati válasz", responseText.substring(0, 50) + "...", "cyan");
    }, 1200);
  }

  function setThinkingState(state) {
    thinkingStateText.textContent = state;
    if (state === 'VÁRAKOZIK') {
      thinkingStateBadge.classList.remove('active');
    } else {
      thinkingStateBadge.classList.add('active');
    }
  }

  composerInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  });

  sendBtn.addEventListener('click', handleSendMessage);

  // --- 4. System Metrics & Merkle Chain ---
  async function updateSystemMetrics() {
    memorySizeVal.textContent = await adapters.wasmMemory.getFormattedSize();
    if (memoryAdapterText && adapters.wasmMemory.getModuleName) {
      memoryAdapterText.textContent = adapters.wasmMemory.getModuleName();
    }
    lastSyncVal.textContent = adapters.fileSync.getLastSyncFormatted();
    merkleHashVal.textContent = adapters.merkleChain.getShortHash();

    // Render Append Log
    appendLogList.innerHTML = adapters.merkleChain.logs.map(log => `
      <div class="log-entry">
        <span class="log-time">${log.time}</span>
        <span class="log-event">[${log.action}] ${escapeHtml(log.detail)}</span>
      </div>
    `).join('');
  }

  // --- 5. Sync Diff & Modals ---
  syncDiffBtn.addEventListener('click', () => {
    syncDiffModal.classList.remove('hidden');
  });

  closeSyncModal.addEventListener('click', () => {
    syncDiffModal.classList.add('hidden');
  });

  confirmSyncBtn.addEventListener('click', async () => {
    await adapters.fileSync.syncLocalState({ messages, logs: adapters.merkleChain.logs });
    addNotification("Szinkron sikeres", "A helyi állapotok egyezése megerősítve.", "copper");
    syncDiffModal.classList.add('hidden');
    updateSystemMetrics();
  });

  // --- 6. APv2 Export ---
  exportApv2Btn.addEventListener('click', () => {
    const exportData = {
      app: "HERMES SERVERLESS",
      version: "2.5.0",
      timestamp: new Date().toISOString(),
      merkleRoot: adapters.merkleChain.merkleRoot,
      logs: adapters.merkleChain.logs,
      messages: messages,
      genomeRules: [
        "RULE-001: Emberi autonómia védelme",
        "RULE-002: Null-szerver adatvédelmi zár",
        "RULE-003: Tamper-evident naplózás"
      ]
    };

    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(exportData, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute("href", dataStr);
    downloadAnchor.setAttribute("download", `hermes_apv2_export_${Date.now()}.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();

    addNotification("APv2 Exportálva", "A tanúsított adatok letöltésre kerültek.", "cyan");
  });

  // Helper Utility
  function escapeHtml(str) {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  // Initialize
  renderNotifications();
  renderMessages();
  adapters.merkleChain.appendLog('SYS_INIT', 'SYSTEM_BOOT_COMPLETED').then(() => {
    updateSystemMetrics();
  });

  // Service Worker Registration
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('./sw.js').catch((err) => {
      console.log('SW registration failed:', err);
    });
  }
});
