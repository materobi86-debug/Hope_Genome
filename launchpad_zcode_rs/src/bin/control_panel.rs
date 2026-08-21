use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::Filter;

use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState, AppTarget};

#[derive(Serialize, Deserialize, Clone)]
struct StateResponse {
    active_app: String,
    tasks: HashMap<usize, String>,
    apps_status: HashMap<String, String>,
    animations: Vec<String>,
    transitions: Vec<String>,
    autostart_enabled: bool,
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
struct SettingsRequest {
    _autostart: bool,
}

const MANIFEST_JSON: &str = r##"{"short_name":"HOPE PWA","name":"HOPE CODE Mobile Virtual Launchpad & Voice Controller","start_url":"/","background_color":"#121214","theme_color":"#00e676","display":"standalone"}"##;

const HTML_INDEX: &str = r##"<!DOCTYPE html>
<html lang="hu">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <link rel="manifest" href="/manifest.json">
    <meta name="theme-color" content="#00e676">
    <title>HOPE CODE Mobile PWA & Voice Launchpad 🎛️🗣️</title>
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
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
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
        h1 { margin: 0; font-size: 1.5rem; color: #fff; }
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

        .voice-box {
            text-align: center;
            padding: 20px;
        }
        .mic-btn {
            width: 80px;
            height: 80px;
            border-radius: 50%;
            background: var(--accent);
            border: none;
            color: #000;
            font-size: 2rem;
            cursor: pointer;
            box-shadow: 0 0 20px rgba(0, 230, 118, 0.4);
            transition: transform 0.2s;
        }
        .mic-btn.listening {
            background: #ff1744;
            color: #fff;
            box-shadow: 0 0 25px #ff1744;
            animation: pulse 1s infinite;
        }
        @keyframes pulse { 0% { transform: scale(1); } 50% { transform: scale(1.08); } 100% { transform: scale(1); } }
        .controls-row {
            display: flex;
            gap: 10px;
            margin-top: 10px;
            align-items: center;
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
            <h1>📱 HOPE CODE Mobile PWA & Voice Launchpad</h1>
            <div><span style="color: #00e676;">● Live Web Audio</span></div>
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
            <button class="tab-btn active" onclick="showTab('virtual-midi')">📱 Mobil Virtuális Launchpad</button>
            <button class="tab-btn" onclick="showTab('voice-control')">🎙️ Magyar Hangvezérlés</button>
            <button class="tab-btn" onclick="showTab('tts-noemi')">🗣️ Noémi Felolvasó & EQ</button>
            <button class="tab-btn" onclick="showTab('preview')">✨ Visualok & Futófelirat</button>
        </div>

        <!-- 1. Mobil Virtuális Launchpad Tab -->
        <div id="virtual-midi" class="tab-content active">
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

        <!-- 2. Magyar Hangvezérlés Tab -->
        <div id="voice-control" class="tab-content">
            <div class="card voice-box">
                <h3>Magyar Hangvezérlés AI Kódoláshoz</h3>
                <p>Mondd ki pl: <i>"Futtass tesztet"</i>, <i>"Válts Claude-ra"</i>, <i>"Állítsd le"</i>, <i>"Olvasd fel a választ"</i></p>
                <button class="mic-btn" id="micBtn" onclick="toggleVoiceRecognition()">🎙️</button>
                <p id="voiceStatus" style="margin-top: 15px; color: #aaa;">Kattints a mikrofonra a beszédhez...</p>
                <p id="speechResult" style="font-weight: bold; color: var(--accent); min-height: 24px;"></p>
            </div>
        </div>

        <!-- 3. Noémi Felolvasó & EQ Tab -->
        <div id="tts-noemi" class="tab-content">
            <div class="card">
                <h3>Edge-TTS Noémi Felolvasó & EQ Sync</h3>
                <textarea id="ttsTextInput" placeholder="Írd ide a szöveget..."></textarea>
                <div class="controls-row">
                    <button class="btn-action" onclick="speakText()">Felolvasás & Launchpad EQ Szinkron</button>
                </div>
            </div>
        </div>

        <!-- 4. Visualok & Futófelirat Tab -->
        <div id="preview" class="tab-content">
            <div class="card">
                <h3>Visualizáció és Futófelirat</h3>
                <div class="controls-row">
                    <select id="animSelect">
                        <option value="text_banner">text_banner (HOPE CODE futófelirat)</option>
                        <option value="cpu_ram_meter">cpu_ram_meter (Élő Monitor)</option>
                        <option value="pomodoro_timer">pomodoro_timer (Fókusz Óra)</option>
                        <option value="equalizer_bars">equalizer_bars (8-bar audio spectrum)</option>
                        <option value="galaxy_spiral">galaxy_spiral</option>
                        <option value="plasma_wave">plasma_wave</option>
                        <option value="matrix_rain">matrix_rain</option>
                        <option value="fireworks">fireworks</option>
                    </select>
                    <input type="text" id="bannerTextInput" value="HOPE CODE" style="width: 100px;">
                    <button class="btn-action" onclick="triggerAnim()">Futtatás</button>
                </div>
            </div>
        </div>
    </div>

    <script>
        let currentActiveApp = 'hopecode';
        let recognition = null;

        function showTab(tabId) {
            document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            event.target.classList.add('active');
        }

        function buildGrid() {
            const grid = document.getElementById('padGrid');
            grid.innerHTML = '';
            for(let i=0; i<64; i++) {
                const pad = document.createElement('div');
                pad.className = 'pad pending';
                pad.id = 'pad-' + i;
                pad.innerText = i + 1;
                pad.onclick = () => togglePad(i);
                grid.appendChild(pad);
            }
        }

        async function switchApp(appName) {
            currentActiveApp = appName;
            document.querySelectorAll('.app-card').forEach(c => c.classList.remove('active'));
            document.querySelectorAll('.side-btn').forEach(b => b.classList.remove('active-view'));

            if (appName === 'hopecode') {
                document.getElementById('app-card-hopecode').classList.add('active');
                document.getElementById('side-hopecode').classList.add('active-view');
            } else if (appName === 'claude_code') {
                document.getElementById('app-card-claude').classList.add('active');
                document.getElementById('side-claude').classList.add('active-view');
            } else if (appName === 'codex') {
                document.getElementById('app-card-codex').classList.add('active');
                document.getElementById('side-codex').classList.add('active-view');
            }

            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'switch_app', app: appName })
            });
        }

        async function fetchState() {
            try {
                const res = await fetch('/api/state');
                const data = await res.json();
                for (let i = 0; i < 64; i++) {
                    const pad = document.getElementById('pad-' + i);
                    if (pad) {
                        const st = data.tasks[i] || 'pending';
                        pad.className = 'pad ' + st.toLowerCase();
                    }
                }
            } catch(e) {}
        }

        async function triggerAnim() {
            const anim = document.getElementById('animSelect').value;
            const textVal = document.getElementById('bannerTextInput').value;
            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'command', app: currentActiveApp, animation: anim, text: textVal })
            });
        }

        async function speakText() {
            const text = document.getElementById('ttsTextInput').value;
            if (!text) return;
            await fetch('/api/speak', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ text: text })
            });
        }

        async function togglePad(taskId) {
            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'task_start', app: currentActiveApp, task_id: taskId, animation: 'galaxy_spiral' })
            });
        }

        function toggleVoiceRecognition() {
            const micBtn = document.getElementById('micBtn');
            const voiceStatus = document.getElementById('voiceStatus');
            const speechResult = document.getElementById('speechResult');

            if (!('webkitSpeechRecognition' in window) && !('SpeechRecognition' in window)) {
                alert('A böngésződ nem támogatja a Web Speech API-t.');
                return;
            }

            if (recognition) {
                recognition.stop();
                recognition = null;
                micBtn.classList.remove('listening');
                voiceStatus.innerText = 'Kattints a mikrofonra a beszédhez...';
                return;
            }

            const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
            recognition = new SpeechRecognition();
            recognition.lang = 'hu-HU';
            recognition.interimResults = false;

            recognition.onstart = () => {
                micBtn.classList.add('listening');
                voiceStatus.innerText = 'Hallgatom a parancsot (hu-HU)...';
            };

            recognition.onresult = async (event) => {
                const transcript = event.results[0][0].transcript.toLowerCase();
                speechResult.innerText = '"' + transcript + '"';
                voiceStatus.innerText = 'Parancs feldolgozva!';

                if (transcript.includes('claude') || transcript.includes('klód')) {
                    await switchApp('claude_code');
                } else if (transcript.includes('codex') || transcript.includes('kodex')) {
                    await switchApp('codex');
                } else if (transcript.includes('hope') || transcript.includes('hop')) {
                    await switchApp('hopecode');
                } else if (transcript.includes('stop') || transcript.includes('állj')) {
                    await fetch('/api/trigger', {
                        method: 'POST',
                        headers: {'Content-Type': 'application/json'},
                        body: JSON.stringify({ event: 'task_error', text: 'STOP!!' })
                    });
                } else {
                    await speakText();
                }

                micBtn.classList.remove('listening');
                recognition = null;
            };

            recognition.onerror = () => {
                micBtn.classList.remove('listening');
                voiceStatus.innerText = 'Hiba történt a felismeréskor.';
                recognition = null;
            };

            recognition.start();
        }

        buildGrid();
        setInterval(fetchState, 500);
    </script>
</body>
</html>
"##;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Control Panel] Starting HOPE CODE Multi-App PWA Web Control Panel on http://127.0.0.1:8080...");

    let lp = LaunchpadMiniMK3::new(true);
    let engine = Arc::new(LaunchpadEngine::new(lp));

    let manifest_route = warp::path("manifest.json").map(|| warp::reply::json(&serde_json::from_str::<serde_json::Value>(MANIFEST_JSON).unwrap()));

    let _engine_state = Arc::clone(&engine);
    let api_state = warp::path!("api" / "state").map(move || {
        let mut tasks_map = HashMap::new();
        // Query current active tasks from engine
        for i in 0..64 {
            tasks_map.insert(i, "pending".to_string());
        }

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
        };
        warp::reply::json(&resp)
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
        .or(api_state)
        .or(api_trigger)
        .or(api_speak)
        .or(api_settings);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
