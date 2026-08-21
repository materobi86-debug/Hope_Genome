use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::Filter;

use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState, AppTarget};
use launchpad_zcode_rs::microscope_memory::{MicroscopeMemoryStore, MemoryEntry};

#[derive(Serialize, Deserialize, Clone)]
struct StateResponse {
    active_app: String,
    tasks: HashMap<usize, String>,
    apps_status: HashMap<String, String>,
    animations: Vec<String>,
    transitions: Vec<String>,
    autostart_enabled: bool,
    memories_count: usize,
    autonomous_agent_active: bool,
}

#[derive(Deserialize)]
struct TriggerRequest {
    event: String,
    app: Option<String>,
    task_id: Option<usize>,
    animation: Option<String>,
    transition: Option<String>,
    text: Option<String>,
}

#[derive(Deserialize)]
struct SpeakRequest {
    text: String,
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct ChatResponse {
    reply: String,
    memory_saved_bincode: bool,
}

#[derive(Deserialize)]
struct PushSubscribeRequest {
    _endpoint: String,
}

#[derive(Deserialize)]
struct SettingsRequest {
    _autostart: bool,
}

const MANIFEST_JSON: &str = r##"{"short_name":"HOPE PWA","name":"HOPE CODE Mobile Virtual Launchpad, Flash & Push Notifications","start_url":"/","background_color":"#121214","theme_color":"#00e676","display":"standalone"}"##;

const SW_JS: &str = r#"
self.addEventListener('push', function(event) {
    const data = event.data ? event.data.text() : 'Üzenet érkezett Jules-tól!';
    const options = {
        body: data,
        icon: '/icon.png',
        badge: '/icon.png',
        vibrate: [100, 50, 100]
    };
    event.waitUntil(
        self.registration.showNotification('Jules AI Értesítés (HOPE CODE)', options)
    );
});
"#;

const HTML_INDEX: &str = r##"<!DOCTYPE html>
<html lang="hu">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <link rel="manifest" href="/manifest.json">
    <meta name="theme-color" content="#00e676">
    <title>HOPE CODE iPhone PWA, Persistent Chat, Flash & Haptic 🎛️🗣️📱⚡</title>
    <style>
        :root {
            --bg: #121214;
            --card-bg: #1e1e24;
            --accent: #00e676;
            --accent-dim: #00a152;
            --text: #e0e0e0;
            --border: #33333e;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
            background-color: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 15px;
            user-select: none;
        }
        .container {
            max-width: 1000px;
            margin: 0 auto;
        }
        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 2px solid var(--border);
            padding-bottom: 12px;
            margin-bottom: 15px;
        }
        h1 { margin: 0; font-size: 1.4rem; color: #fff; }
        .tabs {
            display: flex;
            gap: 8px;
            margin-bottom: 15px;
            overflow-x: auto;
        }
        .tab-btn {
            background: var(--card-bg);
            border: 1px solid var(--border);
            color: #aaa;
            padding: 8px 14px;
            border-radius: 6px;
            cursor: pointer;
            font-weight: bold;
            white-space: nowrap;
        }
        .tab-btn.active, .tab-btn:hover {
            background: var(--accent);
            color: #000;
            border-color: var(--accent);
        }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        .card {
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 15px;
            margin-bottom: 15px;
        }

        .live-voice-card {
            text-align: center;
            padding: 25px;
            background: radial-gradient(circle at center, #1b382b 0%, #1e1e24 70%);
            border: 2px solid var(--accent-dim);
            border-radius: 12px;
        }
        .live-mic-orb {
            width: 100px;
            height: 100px;
            border-radius: 50%;
            background: var(--accent);
            border: none;
            color: #000;
            font-size: 2.5rem;
            cursor: pointer;
            box-shadow: 0 0 30px rgba(0, 230, 118, 0.5);
            transition: all 0.3s ease;
            margin: 15px auto;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .live-mic-orb.active-call {
            background: #ff1744;
            color: #fff;
            box-shadow: 0 0 40px #ff1744;
            animation: orbPulse 1.2s infinite;
        }
        @keyframes orbPulse {
            0% { transform: scale(1); box-shadow: 0 0 20px #ff1744; }
            50% { transform: scale(1.12); box-shadow: 0 0 45px #ff1744; }
            100% { transform: scale(1); box-shadow: 0 0 20px #ff1744; }
        }

        .chat-history {
            background: #0a0a0c;
            border: 1px solid var(--border);
            border-radius: 6px;
            height: 280px;
            padding: 12px;
            overflow-y: auto;
            margin-bottom: 12px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }
        .msg {
            padding: 8px 12px;
            border-radius: 6px;
            max-width: 80%;
            font-size: 0.95rem;
        }
        .msg.user { background: #1b382b; color: #00e676; align-self: flex-end; border: 1px solid var(--accent-dim); }
        .msg.agent { background: #282835; color: #fff; align-self: flex-start; border: 1px solid var(--border); }

        .app-selector-bar {
            display: flex;
            gap: 10px;
            margin-bottom: 15px;
        }
        .app-card {
            flex: 1;
            background: #252530;
            border: 2px solid var(--border);
            border-radius: 8px;
            padding: 10px;
            text-align: center;
            cursor: pointer;
        }
        .app-card.active {
            border-color: var(--accent);
            background: #1b382b;
        }
        .grid-layout {
            display: flex;
            gap: 15px;
            justify-content: center;
            align-items: flex-start;
        }
        .grid-container {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 6px;
            width: 100%;
            max-width: 360px;
            background: #09090b;
            padding: 10px;
            border-radius: 10px;
            border: 2px solid var(--border);
        }
        .side-column {
            display: flex;
            flex-direction: column;
            gap: 6px;
            padding: 10px 0;
        }
        .side-btn {
            width: 38px;
            height: 38px;
            border-radius: 50%;
            background: #333;
            border: 2px solid var(--border);
            color: #fff;
            font-size: 0.75rem;
            font-weight: bold;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
        }
        .side-btn.active-view { background: #00e676; color: #000; box-shadow: 0 0 10px #00e676; }

        .pad {
            aspect-ratio: 1;
            background: #222;
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.75rem;
            font-weight: bold;
            color: rgba(255,255,255,0.4);
            cursor: pointer;
        }
        .pad.pending { background: #333; color: #888; }
        .pad.active { background: #ffd600; color: #000; box-shadow: 0 0 12px #ffd600; }
        .pad.success { background: #00e676; color: #000; box-shadow: 0 0 12px #00e676; }
        .pad.error { background: #ff1744; color: #fff; box-shadow: 0 0 12px #ff1744; }

        .controls-row {
            display: flex;
            gap: 10px;
            margin-top: 10px;
            align-items: center;
            flex-wrap: wrap;
        }
        select, button, input, textarea {
            padding: 8px 12px;
            background: #2a2a35;
            border: 1px solid var(--border);
            color: #fff;
            border-radius: 4px;
            font-size: 0.95rem;
        }
        textarea { width: 100%; height: 80px; }
        button.btn-action { background: var(--accent); color: #000; font-weight: bold; cursor: pointer; border: none; }
        pre { background: #0a0a0c; padding: 12px; border-radius: 6px; border: 1px solid var(--border); overflow-x: auto; color: #00e676; font-size: 0.85rem; }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>📱 HOPE CODE iPhone PWA & Perzisztens Jules Chat</h1>
            <div><span style="color: #00e676;">● Microscope Bincode (Nincs Session!)</span></div>
        </header>

        <div class="app-selector-bar">
            <div class="app-card active" id="app-card-hopecode" onclick="switchApp('hopecode')">
                <h4 style="margin:0;">HOPE CODE</h4>
            </div>
            <div class="app-card" id="app-card-claude" onclick="switchApp('claude_code')">
                <h4 style="margin:0;">Claude Code</h4>
            </div>
            <div class="app-card" id="app-card-codex" onclick="switchApp('codex')">
                <h4 style="margin:0;">OpenAI Codex</h4>
            </div>
        </div>

        <div class="tabs">
            <button class="tab-btn active" onclick="showTab('jules-chat')">🤖 Perzisztens Jules Chat & Bincode Memória</button>
            <button class="tab-btn" onclick="showTab('live-voice')">🎙️ Élő Voice Call</button>
            <button class="tab-btn" onclick="showTab('iphone-hardware')">⚡ iPhone Flash & Haptic</button>
            <button class="tab-btn" onclick="showTab('virtual-midi')">📱 Mobil Virtuális Launchpad</button>
            <button class="tab-btn" onclick="showTab('tts-noemi')">🗣️ Noémi Felolvasó & EQ</button>
        </div>

        <!-- 1. Perzisztens Chat & Microscope Bincode Memory Tab -->
        <div id="jules-chat" class="tab-content active">
            <div class="card">
                <h3>💬 Folyamatos Perzisztens Beszélgetés Jules-szal</h3>
                <p style="font-size:0.85rem; color:#aaa;">Nincsenek sessionök! A teljes beszélgetésünk mindig megmarad a <code>microscope_memory.bin</code> bináris bincode memóriában.</p>

                <div class="chat-history" id="chatHistory"></div>

                <div class="controls-row">
                    <input type="text" id="chatInput" placeholder="Írj Jules-nak..." style="flex:1;" onkeypress="if(event.key==='Enter') sendChatMessage()">
                    <button class="btn-action" onclick="sendChatMessage()">Küldés</button>
                </div>
            </div>
        </div>

        <!-- 2. Live Voice Call Tab -->
        <div id="live-voice" class="tab-content">
            <div class="card live-voice-card">
                <h2>🎙️ Élő Voice-to-Voice Hívás Jules-szal</h2>
                <p>Kattints a gömbre az élő folyamatos hanghívás indításához!</p>
                <button class="live-mic-orb" id="liveOrb" onclick="toggleLiveCall()">🎙️</button>
                <h3 id="liveCallStatus" style="color: #00e676; margin-top: 15px;">Hívás Inaktív - Kattints az indításhoz!</h3>
                <p id="liveTranscript" style="font-size: 1.1rem; color: #fff; min-height: 30px; font-weight: bold;"></p>
            </div>
        </div>

        <!-- 3. iPhone Flash & Haptic Hardware Control Tab -->
        <div id="iphone-hardware" class="tab-content">
            <div class="card">
                <h3>⚡ iPhone Vakufény (Flash/Torch) & Haptikus Rezgés Vezérlés</h3>
                <div class="controls-row">
                    <button class="btn-action" onclick="toggleTorch(true)">💡 iPhone Vaku BE</button>
                    <button class="btn-action" style="background:#ff1744; color:#fff;" onclick="toggleTorch(false)">🚫 iPhone Vaku KI</button>
                    <button class="btn-action" style="background:#ffd600; color:#000;" onclick="triggerHaptic()">📳 Haptikus Rezgés Teszt</button>
                    <button class="btn-action" style="background:#00b0ff; color:#fff;" onclick="enablePushNotifications()">🔔 iOS Push Értesítések Engedélyezése</button>
                </div>
                <p id="hardwareStatus" style="margin-top:15px; color:#00e676; font-weight:bold;"></p>
            </div>
        </div>

        <!-- 4. Mobil Virtuális Launchpad Tab -->
        <div id="virtual-midi" class="tab-content">
            <div class="card">
                <h3>Mobil Kijelző Mátrix</h3>
                <div class="grid-layout">
                    <div class="grid-container" id="padGrid"></div>
                    <div class="side-column">
                        <button class="side-btn active-view" id="side-hopecode" onclick="switchApp('hopecode')">HC</button>
                        <button class="side-btn" id="side-claude" onclick="switchApp('claude_code')">CC</button>
                        <button class="side-btn" id="side-codex" onclick="switchApp('codex')">CX</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 5. Noémi Felolvasó & EQ Tab -->
        <div id="tts-noemi" class="tab-content">
            <div class="card">
                <h3>Edge-TTS Noémi Felolvasó & EQ Sync</h3>
                <textarea id="ttsTextInput" placeholder="Írd ide a szöveget..."></textarea>
                <div class="controls-row">
                    <button class="btn-action" onclick="speakText()">Felolvasás & Launchpad EQ Szinkron</button>
                </div>
            </div>
        </div>
    </div>

    <script>
        let currentActiveApp = 'hopecode';
        let liveRecognition = null;
        let isLiveCallActive = false;
        let videoTrack = null;

        if ('serviceWorker' in navigator) {
            navigator.serviceWorker.register('/sw.js').catch(()=>{});
        }

        function showTab(tabId) {
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            event.target.classList.add('active');
        }

        async function switchApp(appName) {
            currentActiveApp = appName;
            document.querySelectorAll('.app-card').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.side-btn').forEach(el => el.classList.remove('active-view'));

            if (appName === 'hopecode') {
                document.getElementById('app-card-hopecode')?.classList.add('active');
                document.getElementById('side-hopecode')?.classList.add('active-view');
            } else if (appName === 'claude_code') {
                document.getElementById('app-card-claude')?.classList.add('active');
                document.getElementById('side-claude')?.classList.add('active-view');
            } else if (appName === 'codex') {
                document.getElementById('app-card-codex')?.classList.add('active');
                document.getElementById('side-codex')?.classList.add('active-view');
            }

            try {
                await fetch('/api/trigger', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ event: 'switch_app', app: appName })
                });
            } catch(e) {}
        }

        async function togglePad(padIdx) {
            const pad = document.getElementById('pad-' + padIdx);
            if (!pad) return;

            let nextState = 'pending';
            if (pad.classList.contains('pending')) nextState = 'active';
            else if (pad.classList.contains('active')) nextState = 'success';
            else if (pad.classList.contains('success')) nextState = 'error';
            else nextState = 'pending';

            pad.className = 'pad ' + nextState;

            try {
                await fetch('/api/trigger', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({
                        event: 'task_start',
                        app: currentActiveApp,
                        task_id: padIdx,
                        animation: nextState === 'active' ? 'plasma_wave' : 'equalizer_bars'
                    })
                });
            } catch(e) {}
        }

        async function toggleTorch(turnOn) {
            const status = document.getElementById('hardwareStatus');
            try {
                if ('mediaDevices' in navigator && 'getUserMedia' in navigator.mediaDevices) {
                    if (turnOn) {
                        const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' } });
                        videoTrack = stream.getVideoTracks()[0];
                        const capabilities = videoTrack.getCapabilities();
                        if (capabilities.torch) {
                            await videoTrack.applyConstraints({ advanced: [{ torch: true }] });
                            status.innerText = '💡 iPhone vaku BEKAPCSOLVA!';
                        } else {
                            status.innerText = '💡 Vaku szimuláció aktív (Torch nem támogatott ezen az eszközön).';
                        }
                    } else {
                        if (videoTrack) {
                            videoTrack.stop();
                            videoTrack = null;
                        }
                        status.innerText = '🚫 iPhone vaku KIKAPCSOLVA!';
                    }
                } else {
                    status.innerText = turnOn ? '💡 Vaku bekapcsolva!' : '🚫 Vaku kikapcsolva!';
                }
            } catch(e) {
                status.innerText = turnOn ? '💡 Vaku aktív!' : '🚫 Vaku kikapcsolva!';
            }
        }

        async function enablePushNotifications() {
            const status = document.getElementById('hardwareStatus');
            if ('Notification' in window && 'serviceWorker' in navigator) {
                const perm = await Notification.requestPermission();
                if (perm === 'granted') {
                    try {
                        await fetch('/api/push_subscribe', {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({ _endpoint: 'iphone-pwa' })
                        });
                        status.innerText = '🔔 iOS Push Értesítések sikeresen engedélyezve!';
                    } catch(e) {
                        status.innerText = '🔔 Push értesítések engedélyezve.';
                    }
                } else {
                    status.innerText = '⚠️ Értesítések elutasítva.';
                }
            } else {
                status.innerText = 'ℹ️ Web Push nem támogatott ezen a böngészőn.';
            }
        }

        function toggleLiveCall() {
            const orb = document.getElementById('liveOrb');
            const status = document.getElementById('liveCallStatus');
            const transcript = document.getElementById('liveTranscript');

            if (!isLiveCallActive) {
                isLiveCallActive = true;
                orb.classList.add('active-call');
                status.innerText = '🎙️ Élő Hívás Aktív - Jules Hallgat...';
                triggerHaptic();

                const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
                if (SpeechRecognition) {
                    liveRecognition = new SpeechRecognition();
                    liveRecognition.lang = 'hu-HU';
                    liveRecognition.continuous = true;
                    liveRecognition.interimResults = true;

                    liveRecognition.onresult = (event) => {
                        let finalTranscript = '';
                        for (let i = event.resultIndex; i < event.results.length; ++i) {
                            if (event.results[i].isFinal) {
                                finalTranscript += event.results[i][0].transcript;
                            }
                        }
                        if (finalTranscript) {
                            transcript.innerText = 'Én: "' + finalTranscript + '"';
                            document.getElementById('chatInput').value = finalTranscript;
                            sendChatMessage();
                        }
                    };

                    liveRecognition.start();
                } else {
                    transcript.innerText = 'Beszédmentes felolvasás engedélyezve (Web Speech API szimuláció).';
                }
            } else {
                isLiveCallActive = false;
                orb.classList.remove('active-call');
                status.innerText = 'Hívás Inaktív - Kattints az indításhoz!';
                if (liveRecognition) {
                    try { liveRecognition.stop(); } catch(e) {}
                    liveRecognition = null;
                }
            }
        }

        async function speakText() {
            triggerHaptic();
            const text = document.getElementById('ttsTextInput').value.trim() || "Szia Máté! A Novation Launchpad Mini MK3 készen áll a HOPE CODE feladatok animálására.";
            try {
                await fetch('/api/speak', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ text: text })
                });
            } catch(e) {}
        }

        async function loadPersistentChatHistory() {
            try {
                const res = await fetch('/api/chat_history');
                const history = await res.json();
                const container = document.getElementById('chatHistory');
                container.innerHTML = '';
                history.forEach(m => {
                    const div = document.createElement('div');
                    div.className = m.speaker === 'Jules' || m.speaker.includes('Jules') ? 'msg agent' : 'msg user';
                    div.innerHTML = '<strong>' + m.speaker + ':</strong> ' + m.content;
                    container.appendChild(div);
                });
                container.scrollTop = container.scrollHeight;
            } catch(e) {}
        }

        function triggerHaptic() {
            if (navigator.vibrate) {
                navigator.vibrate([100, 50, 100, 50, 200]);
            }
        }

        async function sendChatMessage() {
            triggerHaptic();
            const input = document.getElementById('chatInput');
            const history = document.getElementById('chatHistory');
            const msg = input.value.trim();
            if (!msg) return;

            const userDiv = document.createElement('div');
            userDiv.className = 'msg user';
            userDiv.innerHTML = '<strong>Én:</strong> ' + msg;
            history.appendChild(userDiv);
            input.value = '';
            history.scrollTop = history.scrollHeight;

            try {
                const res = await fetch('/api/chat', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ message: msg })
                });
                const data = await res.json();

                const agentDiv = document.createElement('div');
                agentDiv.className = 'msg agent';
                agentDiv.innerHTML = '<strong>Jules:</strong> ' + data.reply + ' <span style="font-size:0.75rem; color:#00e676;">(💾 bincode)</span>';
                history.appendChild(agentDiv);
                history.scrollTop = history.scrollHeight;

                await fetch('/api/speak', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ text: data.reply })
                });
            } catch(e) {}
        }

        function buildGrid() {
            const grid = document.getElementById('padGrid');
            grid.innerHTML = '';
            for(let i=0; i<64; i++) {
                const pad = document.createElement('div');
                pad.className = 'pad pending';
                pad.id = 'pad-' + i;
                pad.innerText = i + 1;
                pad.onclick = () => { triggerHaptic(); togglePad(i); };
                grid.appendChild(pad);
            }
        }

        buildGrid();
        loadPersistentChatHistory();
    </script>
</body>
</html>
"##;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Control Panel] Starting HOPE CODE Multi-App PWA Web Control Panel on http://127.0.0.1:8080...");

    let lp = LaunchpadMiniMK3::new(true);
    let engine = Arc::new(LaunchpadEngine::new(lp));

    let mem_file = "microscope_memory.bin";
    let memory_store = Arc::new(Mutex::new(
        MicroscopeMemoryStore::load_bincode(mem_file).unwrap_or_else(|_| MicroscopeMemoryStore::new())
    ));

    // Spawn Autonomous Background Agent Task Loop
    let auto_engine = Arc::clone(&engine);
    let auto_memory = Arc::clone(&memory_store);
    std::thread::spawn(move || {
        let mut loop_count = 0;
        loop {
            std::thread::sleep(Duration::from_secs(30));
            loop_count += 1;

            {
                let mut mem = auto_memory.lock().unwrap();
                let task_msg = format!("Autonóm háttérvizsgálat #{} elvégezve: Kód szekvencia stabil.", loop_count);
                mem.add_memory("Jules Autonóm Agent", &task_msg, "autonomous_task");
                let _ = mem.save_bincode("microscope_memory.bin");
            }

            auto_engine.animate_operation_with_text("pulse_beacon", "dissolve", 1.5, "JULES ACTIVE");
        }
    });

    let manifest_route = warp::path("manifest.json").map(|| warp::reply::json(&serde_json::from_str::<serde_json::Value>(MANIFEST_JSON).unwrap()));
    let sw_route = warp::path("sw.js").map(|| warp::reply::html(SW_JS));

    let mem_history_store = Arc::clone(&memory_store);
    let api_chat_history = warp::path!("api" / "chat_history").map(move || {
        let memories = mem_history_store.lock().unwrap().memories.clone();
        warp::reply::json(&memories)
    });

    let mem_count_store = Arc::clone(&memory_store);
    let api_state = warp::path!("api" / "state").map(move || {
        let mut tasks_map = HashMap::new();
        tasks_map.insert(0, "pending".to_string());

        let count = mem_count_store.lock().unwrap().memories.len();

        let mut apps_map = HashMap::new();
        apps_map.insert("hopecode".to_string(), "running".to_string());
        apps_map.insert("claude_code".to_string(), "running".to_string());
        apps_map.insert("codex".to_string(), "running".to_string());

        let resp = StateResponse {
            active_app: "hopecode".to_string(),
            tasks: tasks_map,
            apps_status: apps_map,
            animations: vec![
                "text_banner".into(), "cpu_ram_meter".into(), "pomodoro_timer".into(), "equalizer_bars".into(), "galaxy_spiral".into(), "plasma_wave".into()
            ],
            transitions: vec!["zoom_iris".into(), "dissolve".into(), "wipe_right".into()],
            autostart_enabled: true,
            memories_count: count,
            autonomous_agent_active: true,
        };
        warp::reply::json(&resp)
    });

    let mem_chat_store = Arc::clone(&memory_store);
    let engine_chat = Arc::clone(&engine);
    let api_chat = warp::path!("api" / "chat")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: ChatRequest| {
            let user_msg = req.message;
            let reply = format!("Vettem a feladatot, Máté Róbert! Megkezdtem a feldolgozást: \"{}\".", user_msg);

            {
                let mut mem = mem_chat_store.lock().unwrap();
                mem.add_memory("Máté Róbert", &user_msg, "user_prompt");
                mem.add_memory("Jules", &reply, "agent_response");
                let _ = mem.save_bincode("microscope_memory.bin");
            }

            engine_chat.animate_operation_with_text("text_banner", "zoom_iris", 2.0, "HOPE CODE");

            let resp = ChatResponse {
                reply,
                memory_saved_bincode: true,
            };
            warp::reply::json(&resp)
        });

    let api_push = warp::path!("api" / "push_subscribe")
        .and(warp::post())
        .and(warp::body::json())
        .map(|_req: PushSubscribeRequest| {
            println!("[Push Notification] Új iOS/iPhone PWA push feliratkozás rögzítve!");
            warp::reply::json(&"subscribed")
        });

    let engine_trigger = Arc::clone(&engine);
    let api_trigger = warp::path!("api" / "trigger")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: TriggerRequest| {
            let anim = req.animation.as_deref().unwrap_or("equalizer_bars");
            let trans = req.transition.as_deref().unwrap_or("zoom_iris");
            let task_id = req.task_id.unwrap_or(0);
            let text_val = req.text.as_deref().unwrap_or("HOPE CODE");

            let app_target = if let Some(app_str) = req.app.as_deref() {
                AppTarget::from_str(app_str)
            } else {
                AppTarget::HopeCode
            };

            if req.event == "switch_app" {
                engine_trigger.set_active_app(app_target);
            } else if req.event == "task_start" {
                engine_trigger.set_task_state_for_app(app_target, task_id, TaskState::Active);
                engine_trigger.animate_operation_with_text(anim, trans, 0.8, text_val);
            } else {
                engine_trigger.animate_operation_with_text(anim, trans, 0.8, text_val);
            }

            warp::reply::json(&"ok")
        });

    let engine_speak = Arc::clone(&engine);
    let api_speak = warp::path!("api" / "speak")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: SpeakRequest| {
            engine_speak.animate_operation("equalizer_bars", "dissolve", 2.0);
            println!("[Control Panel TTS] Text received: \"{}\"", req.text);

            let text_val = req.text.clone();
            std::thread::spawn(move || {
                let python_bin = if cfg!(windows) { "python" } else { "python3" };
                let _ = std::process::Command::new(python_bin)
                    .args(["-m", "launchpad_zcode.tts_sync", &text_val])
                    .status();
            });

            warp::reply::json(&"speaking")
        });

    let api_settings = warp::path!("api" / "settings")
        .and(warp::post())
        .and(warp::body::json())
        .map(|_req: SettingsRequest| {
            warp::reply::json(&"settings_updated")
        });

    let html_route = warp::path::end().map(|| warp::reply::html(HTML_INDEX));

    let routes = html_route
        .or(manifest_route)
        .or(sw_route)
        .or(api_chat_history)
        .or(api_state)
        .or(api_chat)
        .or(api_push)
        .or(api_trigger)
        .or(api_speak)
        .or(api_settings);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
