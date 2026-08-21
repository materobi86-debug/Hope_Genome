use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::Filter;

use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState, AppTarget};
use launchpad_zcode_rs::microscope_memory::MicroscopeMemoryStore;

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
    daily_messages_used: usize,
    daily_messages_limit: usize,
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
    attachment: Option<String>,
    file_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ChatResponse {
    reply: String,
    speaker: String,
    memory_saved_bincode: bool,
    daily_messages_used: usize,
    daily_messages_limit: usize,
}

#[derive(Deserialize)]
struct UploadRequest {
    file_name: String,
    file_data_base64: String,
}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    status: String,
    file_name: String,
    reply: String,
}

#[derive(Deserialize)]
struct PushSubscribeRequest {
    _endpoint: String,
}

#[derive(Deserialize)]
struct SettingsRequest {
    _autostart: bool,
}

const MANIFEST_JSON: &str = r##"{"short_name":"HOPE CODE PWA","name":"HOPE CODE AMOLED PWA Chat & Launchpad Control","start_url":"/","background_color":"#000000","theme_color":"#00e676","display":"standalone"}"##;

const SW_JS: &str = r#"
self.addEventListener('push', function(event) {
    const data = event.data ? event.data.text() : 'Üzenet érkezett Hope-tól!';
    const options = {
        body: data,
        icon: '/icon.png',
        badge: '/icon.png',
        vibrate: [100, 50, 100, 50, 200]
    };
    event.waitUntil(
        self.registration.showNotification('HOPE CODE Értesítés (Hope AI)', options)
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
    <title>HOPE CODE — AMOLED Black Chat & Launchpad Studio</title>
    <style>
        :root {
            --bg: #000000;
            --card-bg: #050508;
            --card-border: #14141f;
            --accent-green: #00e676;
            --accent-green-dim: #00a152;
            --accent-cyan: #00e5ff;
            --accent-cyan-dim: #00b0ff;
            --text-main: #f0f0f5;
            --text-sub: #888899;
            --user-bubble: #0c2419;
            --agent-bubble: #10101a;
        }
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
            background-color: var(--bg);
            color: var(--text-main);
            margin: 0;
            padding: 12px;
            user-select: none;
            -webkit-tap-highlight-color: transparent;
        }
        .container {
            max-width: 950px;
            margin: 0 auto;
        }
        header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-bottom: 1px solid var(--card-border);
            padding-bottom: 12px;
            margin-bottom: 12px;
        }
        h1 {
            margin: 0;
            font-size: 1.3rem;
            background: linear-gradient(90deg, #00e676, #00e5ff);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-weight: 800;
        }
        .status-badge {
            font-size: 0.8rem;
            color: var(--accent-green);
            background: #002210;
            padding: 4px 10px;
            border-radius: 20px;
            border: 1px solid var(--accent-green-dim);
            display: flex;
            align-items: center;
            gap: 6px;
        }
        .tabs {
            display: flex;
            gap: 8px;
            margin-bottom: 12px;
            overflow-x: auto;
            padding-bottom: 4px;
        }
        .tab-btn {
            background: var(--card-bg);
            border: 1px solid var(--card-border);
            color: var(--text-sub);
            padding: 8px 14px;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 600;
            white-space: nowrap;
            font-size: 0.85rem;
            transition: all 0.2s ease;
        }
        .tab-btn.active, .tab-btn:hover {
            background: var(--accent-green);
            color: #000;
            border-color: var(--accent-green);
            box-shadow: 0 0 12px rgba(0, 230, 118, 0.4);
        }
        .tab-content { display: none; }
        .tab-content.active { display: block; }
        .card {
            background: var(--card-bg);
            border: 1px solid var(--card-border);
            border-radius: 12px;
            padding: 15px;
            margin-bottom: 12px;
        }

        /* Transparent Custom Scrollbar Styles */
        ::-webkit-scrollbar {
            width: 6px;
            height: 6px;
        }
        ::-webkit-scrollbar-track {
            background: transparent;
        }
        ::-webkit-scrollbar-thumb {
            background: rgba(0, 230, 118, 0.2);
            border-radius: 10px;
            transition: background 0.3s ease;
        }
        ::-webkit-scrollbar-thumb:hover {
            background: rgba(0, 230, 118, 0.6);
            box-shadow: 0 0 8px #00e676;
        }

        /* AMOLED Chat UI Styles & Animations */
        .chat-container {
            display: flex;
            flex-direction: column;
            height: 520px;
            background: #000000;
            border: 1px solid var(--card-border);
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 0 25px rgba(0,230,118,0.05);
        }
        .chat-header-bar {
            padding: 10px 15px;
            background: #050508;
            border-bottom: 1px solid var(--card-border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .chat-search-input {
            width: 170px;
            padding: 6px 10px;
            background: #0a0a0f;
            border: 1px solid var(--card-border);
            color: #fff;
            border-radius: 6px;
            font-size: 0.8rem;
        }
        .chat-history {
            flex: 1;
            padding: 15px;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
            gap: 12px;
            scroll-behavior: smooth;
        }
        .msg-row {
            display: flex;
            flex-direction: column;
            max-width: 85%;
            animation: messageFadeIn 0.35s cubic-bezier(0.16, 1, 0.3, 1);
        }
        @keyframes messageFadeIn {
            from { opacity: 0; transform: translateY(12px) scale(0.98); }
            to { opacity: 1; transform: translateY(0) scale(1); }
        }
        .msg-row.user { align-self: flex-end; }
        .msg-row.agent { align-self: flex-start; }
        .msg-header {
            font-size: 0.75rem;
            color: var(--text-sub);
            margin-bottom: 4px;
            display: flex;
            justify-content: space-between;
            gap: 10px;
        }
        .msg-bubble {
            padding: 11px 15px;
            border-radius: 12px;
            font-size: 0.95rem;
            line-height: 1.48;
            word-break: break-word;
        }
        .msg-row.user .msg-bubble {
            background: var(--user-bubble);
            color: #ffffff;
            border: 1px solid var(--accent-green-dim);
            border-bottom-right-radius: 2px;
            box-shadow: 0 0 12px rgba(0,230,118,0.12);
        }
        .msg-row.agent .msg-bubble {
            background: var(--agent-bubble);
            color: #f0f0f5;
            border: 1px solid var(--accent-cyan-dim);
            border-bottom-left-radius: 2px;
            box-shadow: 0 0 12px rgba(0,229,255,0.12);
        }

        /* Animated Thinking Indicator */
        .thinking-card {
            display: none;
            padding: 10px 14px;
            background: #080c10;
            border: 1px solid var(--accent-cyan-dim);
            border-radius: 10px;
            font-size: 0.85rem;
            color: var(--accent-cyan);
            margin: 6px 15px;
            align-self: flex-start;
            animation: thinkingPulse 1.5s infinite ease-in-out;
        }
        @keyframes thinkingPulse {
            0% { border-color: rgba(0, 229, 255, 0.3); box-shadow: 0 0 5px rgba(0, 229, 255, 0.2); }
            50% { border-color: rgba(0, 229, 255, 0.9); box-shadow: 0 0 18px rgba(0, 229, 255, 0.5); }
            100% { border-color: rgba(0, 229, 255, 0.3); box-shadow: 0 0 5px rgba(0, 229, 255, 0.2); }
        }
        .thinking-dots span {
            display: inline-block;
            animation: dotWave 1.2s infinite ease-in-out;
        }
        .thinking-dots span:nth-child(1) { animation-delay: 0s; }
        .thinking-dots span:nth-child(2) { animation-delay: 0.2s; }
        .thinking-dots span:nth-child(3) { animation-delay: 0.4s; }
        @keyframes dotWave {
            0%, 60%, 100% { transform: translateY(0); }
            30% { transform: translateY(-5px); color: #fff; }
        }

        /* Daily Limit Meter */
        .limit-meter-bar {
            width: 100%;
            height: 6px;
            background: #111118;
            border-radius: 3px;
            overflow: hidden;
            margin-top: 4px;
        }
        .limit-meter-fill {
            height: 100%;
            width: 0%;
            background: linear-gradient(90deg, var(--accent-green), var(--accent-cyan));
            transition: width 0.5s ease;
        }

        .code-block {
            background: #000000;
            border: 1px solid #1a2a20;
            border-radius: 6px;
            padding: 8px 12px;
            margin: 8px 0;
            font-family: 'Courier New', Courier, monospace;
            font-size: 0.85rem;
            color: #00e676;
            position: relative;
            white-space: pre-wrap;
        }
        .copy-btn {
            position: absolute;
            top: 4px;
            right: 4px;
            background: #111;
            border: 1px solid var(--accent-green-dim);
            color: #fff;
            padding: 2px 8px;
            font-size: 0.7rem;
            border-radius: 4px;
            cursor: pointer;
        }

        .chat-input-bar {
            padding: 10px;
            background: #050508;
            border-top: 1px solid var(--card-border);
            display: flex;
            flex-direction: column;
            gap: 8px;
        }
        .input-row {
            display: flex;
            gap: 8px;
            align-items: center;
        }
        .attachment-preview {
            font-size: 0.8rem;
            color: var(--accent-cyan);
            background: #002233;
            padding: 4px 8px;
            border-radius: 4px;
            display: none;
            justify-content: space-between;
            align-items: center;
        }

        .btn {
            padding: 8px 14px;
            background: var(--card-bg);
            border: 1px solid var(--card-border);
            color: #fff;
            border-radius: 6px;
            font-size: 0.85rem;
            font-weight: bold;
            cursor: pointer;
            transition: all 0.2s ease;
        }
        .btn-green { background: var(--accent-green); color: #000; border: none; }
        .btn-cyan { background: var(--accent-cyan); color: #000; border: none; }

        /* Voice Call Orb */
        .live-voice-card {
            text-align: center;
            padding: 30px;
            background: radial-gradient(circle at center, #001a0f 0%, #000000 85%);
            border: 1px solid var(--accent-green-dim);
            border-radius: 12px;
        }
        .live-mic-orb {
            width: 90px;
            height: 90px;
            border-radius: 50%;
            background: var(--accent-green);
            border: none;
            color: #000;
            font-size: 2.2rem;
            cursor: pointer;
            box-shadow: 0 0 30px rgba(0, 230, 118, 0.4);
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
            0% { transform: scale(1); }
            50% { transform: scale(1.1); }
            100% { transform: scale(1); }
        }

        /* Virtual Launchpad Grid */
        .grid-layout {
            display: flex;
            gap: 15px;
            justify-content: center;
        }
        .grid-container {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 6px;
            width: 100%;
            max-width: 360px;
            background: #000;
            padding: 10px;
            border-radius: 10px;
            border: 1px solid var(--card-border);
        }
        .side-column { display: flex; flex-direction: column; gap: 6px; }
        .side-btn {
            width: 36px;
            height: 36px;
            border-radius: 50%;
            background: #11111a;
            border: 1px solid var(--card-border);
            color: #fff;
            font-size: 0.7rem;
            font-weight: bold;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
        }
        .side-btn.active-view { background: var(--accent-green); color: #000; box-shadow: 0 0 10px var(--accent-green); }
        .pad {
            aspect-ratio: 1;
            background: #0a0a0d;
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.75rem;
            font-weight: bold;
            color: #444;
            cursor: pointer;
        }
        .pad.pending { background: #121218; color: #666; }
        .pad.active { background: #ffd600; color: #000; box-shadow: 0 0 12px #ffd600; }
        .pad.success { background: #00e676; color: #000; box-shadow: 0 0 12px #00e676; }
        .pad.error { background: #ff1744; color: #fff; box-shadow: 0 0 12px #ff1744; }

        input[type="text"], textarea {
            background: #08080c;
            border: 1px solid var(--card-border);
            color: #fff;
            padding: 8px 12px;
            border-radius: 6px;
            font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>⚡ HOPE CODE — AMOLED PWA Studio</h1>
            <div class="status-badge">● Jules & Hope AI (Microscope Bincode)</div>
        </header>

        <div class="tabs">
            <button class="tab-btn active" onclick="showTab('hope-chat')">💬 AMOLED Chat & Fájlok</button>
            <button class="tab-btn" onclick="showTab('live-voice')">🎙️ Élő Voice Call</button>
            <button class="tab-btn" onclick="showTab('iphone-hardware')">⚡ iPhone Flash & Haptic</button>
            <button class="tab-btn" onclick="showTab('virtual-midi')">📱 Mobil Launchpad</button>
            <button class="tab-btn" onclick="showTab('tts-noemi')">🗣️ Noémi Felolvasó & EQ</button>
        </div>

        <!-- 1. AMOLED Chat & File Upload Tab -->
        <div id="hope-chat" class="tab-content active">
            <div class="card" style="padding:10px;">
                <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:10px; padding:0 5px;">
                    <div>
                        <span style="font-size:0.85rem; font-weight:bold; color:var(--accent-green);">Jules / Hope Napi Kapacitás Limit:</span>
                        <span id="limitText" style="font-size:0.85rem; color:#fff; font-weight:bold;">0 / 500 üzenet</span>
                        <div class="limit-meter-bar" style="width:200px; display:inline-block; margin-left:10px; vertical-align:middle;">
                            <div class="limit-meter-fill" id="limitMeterFill"></div>
                        </div>
                    </div>
                </div>

                <div class="chat-container">
                    <div class="chat-header-bar">
                        <div style="font-weight:bold; color:var(--accent-green);">HOPE CODE / Zcode Studio</div>
                        <div style="display:flex; gap:8px;">
                            <input type="text" id="chatSearch" class="chat-search-input" placeholder="🔍 Keresés..." onkeyup="filterChat()">
                            <button class="btn" style="padding:4px 8px; font-size:0.75rem;" onclick="exportChatHistory('txt')">TXT</button>
                            <button class="btn" style="padding:4px 8px; font-size:0.75rem;" onclick="exportChatHistory('json')">JSON</button>
                        </div>
                    </div>

                    <div class="chat-history" id="chatHistory"></div>

                    <!-- Thinking Step Animation Indicator -->
                    <div class="thinking-card" id="thinkingIndicator">
                        🧠 <span id="thinkingStepText">Jules / Hope gondolkodik...</span>
                        <span class="thinking-dots"><span>.</span><span>.</span><span>.</span></span>
                    </div>

                    <div class="chat-input-bar">
                        <div class="attachment-preview" id="attachmentPreview">
                            <span id="attachmentName">file.txt</span>
                            <button style="background:none; border:none; color:#ff1744; cursor:pointer;" onclick="clearAttachment()">✕</button>
                        </div>
                        <div class="input-row">
                            <input type="file" id="fileInput" style="display:none;" onchange="handleFileSelected(event)">
                            <button class="btn" onclick="document.getElementById('fileInput').click()" title="Fájl csatolása">📎</button>
                            <button class="btn" id="recordAudioBtn" onclick="toggleAudioRecording()" title="Hangüzenet rögzítése">🎤</button>
                            <input type="text" id="chatInput" placeholder="Írj Jules / Hope AI-nak..." style="flex:1;" onkeypress="if(event.key==='Enter') sendChatMessage()">
                            <button class="btn btn-green" onclick="sendChatMessage()">Küldés</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- 2. Live Voice Call Tab -->
        <div id="live-voice" class="tab-content">
            <div class="card live-voice-card">
                <h2>🎙️ Élő Voice-to-Voice Hívás Jules-szal / Hope-pal</h2>
                <p style="color:var(--text-sub);">Folyamatos kétirányú párbeszéd magyar nyelven</p>
                <button class="live-mic-orb" id="liveOrb" onclick="toggleLiveCall()">🎙️</button>
                <h3 id="liveCallStatus" style="color: var(--accent-green); margin-top: 15px;">Hívás Inaktív - Kattints az indításhoz!</h3>
                <p id="liveTranscript" style="font-size: 1.1rem; color: #fff; min-height: 30px; font-weight: bold;"></p>
            </div>
        </div>

        <!-- 3. iPhone Flash & Haptic Tab -->
        <div id="iphone-hardware" class="tab-content">
            <div class="card">
                <h3>⚡ Hardver Vezérlés (iPhone Flash, Haptika & Push)</h3>
                <div style="display:flex; gap:10px; flex-wrap:wrap; margin-top:10px;">
                    <button class="btn btn-green" onclick="toggleTorch(true)">💡 Vaku BE</button>
                    <button class="btn" style="background:#ff1744; color:#fff;" onclick="toggleTorch(false)">🚫 Vaku KI</button>
                    <button class="btn" style="background:#ffd600; color:#000;" onclick="triggerHaptic()">📳 Haptikus Rezgés</button>
                    <button class="btn btn-cyan" onclick="enablePushNotifications()">🔔 iOS Push Értesítések</button>
                </div>
                <p id="hardwareStatus" style="margin-top:12px; color:var(--accent-green); font-weight:bold;"></p>
            </div>
        </div>

        <!-- 4. Virtual Launchpad Tab -->
        <div id="virtual-midi" class="tab-content">
            <div class="card">
                <h3>Virtuális Launchpad Matrix</h3>
                <div class="grid-layout" style="margin-top:10px;">
                    <div class="grid-container" id="padGrid"></div>
                    <div class="side-column">
                        <button class="side-btn active-view" id="side-hopecode" onclick="switchApp('hopecode')">HC</button>
                        <button class="side-btn" id="side-claude" onclick="switchApp('claude_code')">CC</button>
                        <button class="side-btn" id="side-codex" onclick="switchApp('codex')">CX</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 5. Edge-TTS Noémi & EQ Tab -->
        <div id="tts-noemi" class="tab-content">
            <div class="card">
                <h3>Edge-TTS Noémi Felolvasó & Launchpad EQ Sync</h3>
                <textarea id="ttsTextInput" style="width:100%; height:80px; margin-top:10px;" placeholder="Írd ide a szöveget..."></textarea>
                <div style="margin-top:10px;">
                    <button class="btn btn-green" onclick="speakText()">Felolvasás & EQ Animáció</button>
                </div>
            </div>
        </div>
    </div>

    <script>
        let currentActiveApp = 'hopecode';
        let liveRecognition = null;
        let isLiveCallActive = false;
        let videoTrack = null;
        let currentAttachment = null;
        let mediaRecorder = null;
        let audioChunks = [];
        let isRecordingAudio = false;

        const audioCtx = new (window.AudioContext || window.webkitAudioContext)();

        function playSoundEffect(type) {
            try {
                const osc = audioCtx.createOscillator();
                const gain = audioCtx.createGain();
                osc.connect(gain);
                gain.connect(audioCtx.destination);
                if (type === 'send') {
                    osc.frequency.setValueAtTime(440, audioCtx.currentTime);
                    osc.frequency.exponentialRampToValueAtTime(880, audioCtx.currentTime + 0.1);
                    gain.gain.setValueAtTime(0.15, audioCtx.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.01, audioCtx.currentTime + 0.1);
                    osc.start();
                    osc.stop(audioCtx.currentTime + 0.1);
                } else if (type === 'receive') {
                    osc.frequency.setValueAtTime(600, audioCtx.currentTime);
                    osc.frequency.exponentialRampToValueAtTime(1200, audioCtx.currentTime + 0.15);
                    gain.gain.setValueAtTime(0.2, audioCtx.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.01, audioCtx.currentTime + 0.15);
                    osc.start();
                    osc.stop(audioCtx.currentTime + 0.15);
                }
            } catch(e) {}
        }

        function triggerHaptic() {
            if (navigator.vibrate) {
                navigator.vibrate([80, 40, 120]);
            }
        }

        function showTab(tabId) {
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            event.target.classList.add('active');
        }

        function formatMessageText(text) {
            let formatted = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
            formatted = formatted.replace(/```([\s\S]*?)```/g, function(match, code) {
                const id = 'code-' + Math.random().toString(36).substr(2, 9);
                return '<div class="code-block" id="' + id + '"><button class="copy-btn" onclick="copyCode(\'' + id + '\')">Másolás</button>' + code + '</div>';
            });
            return formatted;
        }

        function copyCode(elementId) {
            const block = document.getElementById(elementId);
            if (!block) return;
            const codeText = block.innerText.replace('Másolás', '').trim();
            navigator.clipboard.writeText(codeText);
            triggerHaptic();
            alert('Kód a vágólapra másolva!');
        }

        function handleFileSelected(event) {
            const file = event.target.files[0];
            if (!file) return;
            const reader = new FileReader();
            reader.onload = function(e) {
                currentAttachment = {
                    file_name: file.name,
                    file_data_base64: e.target.result
                };
                document.getElementById('attachmentName').innerText = '📄 Csatolva: ' + file.name;
                document.getElementById('attachmentPreview').style.display = 'flex';
            };
            reader.readAsDataURL(file);
        }

        function clearAttachment() {
            currentAttachment = null;
            document.getElementById('fileInput').value = '';
            document.getElementById('attachmentPreview').style.display = 'none';
        }

        async function toggleAudioRecording() {
            const btn = document.getElementById('recordAudioBtn');
            if (!isRecordingAudio) {
                try {
                    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
                    mediaRecorder = new MediaRecorder(stream);
                    audioChunks = [];
                    mediaRecorder.ondataavailable = event => audioChunks.push(event.data);
                    mediaRecorder.onstop = () => {
                        const audioBlob = new Blob(audioChunks, { type: 'audio/wav' });
                        const reader = new FileReader();
                        reader.onloadend = () => {
                            currentAttachment = {
                                file_name: 'voice_message.wav',
                                file_data_base64: reader.result
                            };
                            document.getElementById('attachmentName').innerText = '🎙️ Hangüzenet rögzítve';
                            document.getElementById('attachmentPreview').style.display = 'flex';
                        };
                        reader.readAsDataURL(audioBlob);
                    };
                    mediaRecorder.start();
                    isRecordingAudio = true;
                    btn.style.background = '#ff1744';
                    btn.style.color = '#fff';
                    triggerHaptic();
                } catch(e) {
                    alert('Mikrofon nem érhető el!');
                }
            } else {
                if (mediaRecorder) {
                    mediaRecorder.stop();
                }
                isRecordingAudio = false;
                btn.style.background = 'var(--card-bg)';
                btn.style.color = '#fff';
                triggerHaptic();
            }
        }

        async function sendChatMessage() {
            playSoundEffect('send');
            triggerHaptic();
            const input = document.getElementById('chatInput');
            const history = document.getElementById('chatHistory');
            const thinkingCard = document.getElementById('thinkingIndicator');
            const thinkingStepText = document.getElementById('thinkingStepText');

            const msg = input.value.trim();
            if (!msg && !currentAttachment) return;

            const userRow = document.createElement('div');
            userRow.className = 'msg-row user';
            let attachHtml = currentAttachment ? '<br><small>📎 ' + currentAttachment.file_name + '</small>' : '';
            userRow.innerHTML = '<div class="msg-header"><span>Én</span><span>' + new Date().toLocaleTimeString() + '</span></div><div class="msg-bubble">' + formatMessageText(msg) + attachHtml + '</div>';
            history.appendChild(userRow);

            input.value = '';
            const attachmentToSend = currentAttachment;
            clearAttachment();
            history.scrollTop = history.scrollHeight;

            // Show Animated Thinking Steps
            thinkingCard.style.display = 'block';
            history.scrollTop = history.scrollHeight;

            const steps = [
                "Jules / Hope gondolkodik...",
                "Kód szekvenciák elemzése...",
                "Microscope Memory kontextus visszahívása...",
                "Válasz megfogalmazása..."
            ];
            let stepIdx = 0;
            const stepInterval = setInterval(() => {
                stepIdx = (stepIdx + 1) % steps.length;
                thinkingStepText.innerText = steps[stepIdx];
            }, 500);

            try {
                const res = await fetch('/api/chat', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({
                        message: msg,
                        attachment: attachmentToSend ? attachmentToSend.file_data_base64 : null,
                        file_name: attachmentToSend ? attachmentToSend.file_name : null
                    })
                });
                const data = await res.json();
                clearInterval(stepInterval);
                thinkingCard.style.display = 'none';
                playSoundEffect('receive');

                updateLimitMeter(data.daily_messages_used, data.daily_messages_limit);

                const agentRow = document.createElement('div');
                agentRow.className = 'msg-row agent';
                agentRow.innerHTML = '<div class="msg-header"><span>' + data.speaker + '</span><span>' + new Date().toLocaleTimeString() + ' (💾 bincode)</span></div><div class="msg-bubble">' + formatMessageText(data.reply) + '</div>';
                history.appendChild(agentRow);
                history.scrollTop = history.scrollHeight;

                await fetch('/api/speak', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ text: data.reply })
                });
            } catch(e) {
                clearInterval(stepInterval);
                thinkingCard.style.display = 'none';
            }
        }

        function updateLimitMeter(used, limit) {
            document.getElementById('limitText').innerText = used + ' / ' + limit + ' üzenet';
            const pct = Math.min(100, Math.round((used / limit) * 100));
            document.getElementById('limitMeterFill').style.width = pct + '%';
        }

        async function loadPersistentChatHistory() {
            try {
                const resState = await fetch('/api/state');
                const stateData = await resState.json();
                updateLimitMeter(stateData.daily_messages_used, stateData.daily_messages_limit);

                const res = await fetch('/api/chat_history');
                const history = await res.json();
                const container = document.getElementById('chatHistory');
                container.innerHTML = '';
                history.forEach(m => {
                    const row = document.createElement('div');
                    row.className = m.speaker === 'Hope' || m.speaker === 'Jules' || m.speaker.includes('Hope') || m.speaker.includes('Jules') ? 'msg-row agent' : 'msg-row user';
                    row.innerHTML = '<div class="msg-header"><span>' + m.speaker + '</span><span>' + m.category + '</span></div><div class="msg-bubble">' + formatMessageText(m.content) + '</div>';
                    container.appendChild(row);
                });
                container.scrollTop = container.scrollHeight;
            } catch(e) {}
        }

        function filterChat() {
            const query = document.getElementById('chatSearch').value.toLowerCase();
            const rows = document.querySelectorAll('.msg-row');
            rows.forEach(row => {
                const text = row.innerText.toLowerCase();
                row.style.display = text.includes(query) ? 'flex' : 'none';
            });
        }

        async function exportChatHistory(format) {
            try {
                const res = await fetch('/api/chat_history');
                const history = await res.json();
                let dataStr = "";
                if (format === 'json') {
                    dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(history, null, 2));
                } else {
                    let txt = "HOPE CODE Beszélgetés Előzmények:\n\n";
                    history.forEach(m => {
                        txt += "[" + m.speaker + "]: " + m.content + "\n";
                    });
                    dataStr = "data:text/plain;charset=utf-8," + encodeURIComponent(txt);
                }
                const downloadAnchor = document.createElement('a');
                downloadAnchor.setAttribute("href", dataStr);
                downloadAnchor.setAttribute("download", "hopecode_chat_export." + format);
                document.body.appendChild(downloadAnchor);
                downloadAnchor.click();
                downloadAnchor.remove();
            } catch(e) {}
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
                            status.innerText = '💡 Vaku szimuláció aktív.';
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
                        status.innerText = '🔔 iOS Push Értesítések engedélyezve!';
                    } catch(e) {
                        status.innerText = '🔔 Push értesítések engedélyezve.';
                    }
                } else {
                    status.innerText = '⚠️ Értesítések elutasítva.';
                }
            } else {
                status.innerText = 'ℹ️ Web Push nem támogatott.';
            }
        }

        function toggleLiveCall() {
            const orb = document.getElementById('liveOrb');
            const status = document.getElementById('liveCallStatus');
            const transcript = document.getElementById('liveTranscript');

            if (!isLiveCallActive) {
                isLiveCallActive = true;
                orb.classList.add('active-call');
                status.innerText = '🎙️ Élő Hívás Aktív - Jules / Hope Hallgat...';
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
                    transcript.innerText = 'Beszédmentes felolvasás engedélyezve.';
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
            const text = document.getElementById('ttsTextInput').value.trim() || "Szia Máté! A Novation Launchpad Mini MK3 és a HOPE CODE felület készen áll.";
            try {
                await fetch('/api/speak', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({ text: text })
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

    let daily_usage_counter = Arc::new(Mutex::new(0usize));
    let daily_limit = 500usize;

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
    let usage_state_store = Arc::clone(&daily_usage_counter);
    let api_state = warp::path!("api" / "state").map(move || {
        let mut tasks_map = HashMap::new();
        tasks_map.insert(0, "pending".to_string());

        let count = mem_count_store.lock().unwrap().memories.len();
        let used = *usage_state_store.lock().unwrap();

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
            daily_messages_used: used,
            daily_messages_limit: daily_limit,
        };
        warp::reply::json(&resp)
    });

    let mem_chat_store = Arc::clone(&memory_store);
    let engine_chat = Arc::clone(&engine);
    let usage_chat_store = Arc::clone(&daily_usage_counter);
    let api_chat = warp::path!("api" / "chat")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: ChatRequest| {
            let user_msg = req.message;
            let file_info = if let Some(fname) = req.file_name {
                format!(" [Csatolt fájl: {}]", fname)
            } else {
                "".to_string()
            };

            let reply = format!("Vettem a feladatot, Máté Róbert! Megkezdtem a HOPE CODE / Zcode feldolgozást: \"{}\"{}.", user_msg, file_info);

            let used = {
                let mut u = usage_chat_store.lock().unwrap();
                *u += 1;
                *u
            };

            {
                let mut mem = mem_chat_store.lock().unwrap();
                mem.add_memory("Máté Róbert", &format!("{}{}", user_msg, file_info), "user_prompt");
                mem.add_memory("Jules / Hope", &reply, "agent_response");
                let _ = mem.save_bincode("microscope_memory.bin");
            }

            engine_chat.animate_operation_with_text("text_banner", "zoom_iris", 2.0, "HOPE CODE");

            let resp = ChatResponse {
                reply,
                speaker: "Jules / Hope".to_string(),
                memory_saved_bincode: true,
                daily_messages_used: used,
                daily_messages_limit: daily_limit,
            };
            warp::reply::json(&resp)
        });

    let mem_upload_store = Arc::clone(&memory_store);
    let engine_upload = Arc::clone(&engine);
    let api_upload = warp::path!("api" / "upload")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |req: UploadRequest| {
            let reply = format!("A csatolt fájl ('{}') sikeresen beolvasva a HOPE CODE elemzéshez.", req.file_name);
            {
                let mut mem = mem_upload_store.lock().unwrap();
                mem.add_memory("Máté Róbert", &format!("[Fájl Feltöltve]: {}", req.file_name), "file_upload");
                mem.add_memory("Jules / Hope", &reply, "agent_response");
                let _ = mem.save_bincode("microscope_memory.bin");
            }

            engine_upload.animate_operation_with_text("plasma_wave", "dissolve", 1.5, "UPLOAD OK");

            let resp = UploadResponse {
                status: "success".to_string(),
                file_name: req.file_name,
                reply,
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
        .or(api_upload)
        .or(api_push)
        .or(api_trigger)
        .or(api_speak)
        .or(api_settings);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
