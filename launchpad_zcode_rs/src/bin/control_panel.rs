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

const HTML_INDEX: &str = r#"<!DOCTYPE html>
<html lang="hu">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HOPE CODE Multi-App Launchpad Vezérlőpult 🎛️</title>
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
            padding: 20px;
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
            padding-bottom: 15px;
            margin-bottom: 20px;
        }
        h1 { margin: 0; font-size: 1.8rem; color: #fff; }
        .tabs {
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
        }
        .tab-btn {
            background: var(--card-bg);
            border: 1px solid var(--border);
            color: #aaa;
            padding: 10px 18px;
            border-radius: 6px;
            cursor: pointer;
            font-weight: bold;
            transition: all 0.2s;
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
            padding: 20px;
            margin-bottom: 20px;
        }
        .app-selector-bar {
            display: flex;
            gap: 15px;
            margin-bottom: 20px;
        }
        .app-card {
            flex: 1;
            background: #252530;
            border: 2px solid var(--border);
            border-radius: 8px;
            padding: 15px;
            text-align: center;
            cursor: pointer;
            transition: all 0.2s;
        }
        .app-card.active {
            border-color: var(--accent);
            background: #1b382b;
        }
        .app-card .status-dot {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            margin-right: 6px;
        }
        .status-running { background: #00e676; box-shadow: 0 0 8px #00e676; }
        .status-event { background: #ffd600; box-shadow: 0 0 8px #ffd600; animation: pulse 1s infinite; }
        .status-stopped { background: #666; }
        @keyframes pulse { 0% { opacity: 0.3; } 50% { opacity: 1.0; } 100% { opacity: 0.3; } }

        .grid-layout {
            display: flex;
            gap: 20px;
            justify-content: center;
            align-items: flex-start;
        }
        .grid-container {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 8px;
            width: 400px;
            background: #09090b;
            padding: 15px;
            border-radius: 10px;
            border: 2px solid var(--border);
        }
        .side-column {
            display: flex;
            flex-direction: column;
            gap: 8px;
            padding: 15px 5px;
        }
        .side-btn {
            width: 45px;
            height: 45px;
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
        .side-btn.flashing { background: #ffd600; color: #000; box-shadow: 0 0 10px #ffd600; }

        .pad {
            aspect-ratio: 1;
            background: #222;
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 0.8rem;
            font-weight: bold;
            color: rgba(255,255,255,0.4);
            cursor: pointer;
            transition: background 0.2s, box-shadow 0.2s;
        }
        .pad.pending { background: #333; color: #888; }
        .pad.active { background: #ffd600; color: #000; box-shadow: 0 0 12px #ffd600; }
        .pad.success { background: #00e676; color: #000; box-shadow: 0 0 12px #00e676; }
        .pad.error { background: #ff1744; color: #fff; box-shadow: 0 0 12px #ff1744; }

        .controls-row {
            display: flex;
            gap: 15px;
            margin-top: 15px;
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
        textarea {
            width: 100%;
            height: 90px;
            resize: vertical;
        }
        button.btn-action {
            background: var(--accent);
            color: #000;
            font-weight: bold;
            cursor: pointer;
            border: none;
        }
        button.btn-action:hover {
            background: var(--accent-dim);
        }
        pre {
            background: #0a0a0c;
            padding: 15px;
            border-radius: 6px;
            border: 1px solid var(--border);
            overflow-x: auto;
            color: #00e676;
            font-family: 'Courier New', Courier, monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🎛️ HOPE CODE Multi-App Launchpad Vezérlőpult</h1>
            <div>
                <span style="color: #00e676;">● Daemon Aktív (Port: 9876)</span>
            </div>
        </header>

        <!-- Multi-App Selector Dashboard Bar -->
        <div class="app-selector-bar">
            <div class="app-card active" id="app-card-hopecode" onclick="switchApp('hopecode')">
                <h4><span class="status-dot status-running" id="dot-hopecode"></span> HOPE CODE / Zcode</h4>
                <p style="font-size: 0.85rem; color: #aaa; margin: 5px 0 0 0;">Aktív Nézet (Oldalsó Gomb #1)</p>
            </div>
            <div class="app-card" id="app-card-claude" onclick="switchApp('claude_code')">
                <h4><span class="status-dot status-running" id="dot-claude"></span> Claude Code</h4>
                <p style="font-size: 0.85rem; color: #aaa; margin: 5px 0 0 0;">Háttérben Fut (Oldalsó Gomb #2)</p>
            </div>
            <div class="app-card" id="app-card-codex" onclick="switchApp('codex')">
                <h4><span class="status-dot status-running" id="dot-codex"></span> OpenAI Codex</h4>
                <p style="font-size: 0.85rem; color: #aaa; margin: 5px 0 0 0;">Háttérben Fut (Oldalsó Gomb #3)</p>
            </div>
        </div>

        <div class="tabs">
            <button class="tab-btn active" onclick="showTab('virtual-midi')">📱 Virtuális MIDI & App Nézet</button>
            <button class="tab-btn" onclick="showTab('tts-noemi')">🗣️ Beszélő Noémi & 8-Bar EQ</button>
            <button class="tab-btn" onclick="showTab('preview')">✨ Visualizációk & Futó Felirat</button>
            <button class="tab-btn" onclick="showTab('settings')">⚙️ Indulás & Beállítások</button>
            <button class="tab-btn" onclick="showTab('ai-generator')">🤖 Új Visual Készítése AI-val</button>
        </div>

        <!-- 1. Virtuális MIDI Kijelző & App Switcher Tab -->
        <div id="virtual-midi" class="tab-content active">
            <div class="card">
                <h3>Virtuális Launchpad 8x8 Mátrix & App Váltó Gombok</h3>
                <p>Kattints az oldalsó kör gombokra az App váltáshoz! A mátrix az éppen kiválasztott AI alkalmazás taskjait mutatja.</p>

                <div class="grid-layout">
                    <div class="grid-container" id="padGrid"></div>
                    <div class="side-column">
                        <button class="side-btn active-view" id="side-hopecode" onclick="switchApp('hopecode')" title="HOPE CODE / Zcode">HC</button>
                        <button class="side-btn" id="side-claude" onclick="switchApp('claude_code')" title="Claude Code">CC</button>
                        <button class="side-btn" id="side-codex" onclick="switchApp('codex')" title="OpenAI Codex">CX</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 2. Beszélő Noémi & 8-Bar EQ Sync Tab -->
        <div id="tts-noemi" class="tab-content">
            <div class="card">
                <h3>Edge-TTS Noémi Hangú Felolvasó & Real-Time 8-Bar EQ Sync</h3>
                <p>Írd be a felolvasandó szöveget! A felolvasás alatt a Launchpadon élőben 8-bar audio equalizer animáció látható.</p>
                <textarea id="ttsTextInput" placeholder="Írd ide a válasz szövegét, amit Noémi felolvas egymás utáni mondatokban..."></textarea>
                <div class="controls-row">
                    <button class="btn-action" onclick="speakText()">Felolvasás & Launchpad EQ Szinkron</button>
                </div>
            </div>
        </div>

        <!-- 3. Visualizációk & Futó Felirat Tab -->
        <div id="preview" class="tab-content">
            <div class="card">
                <h3>Visualizáció, Futó Felirat és Átmenet Tesztelése</h3>
                <div class="controls-row">
                    <label>Effekt:</label>
                    <select id="animSelect">
                        <option value="text_banner">text_banner (HOPE CODE / STOP!! futófelirat)</option>
                        <option value="vortex_whirl">vortex_whirl</option>
                        <option value="color_comb">color_comb</option>
                        <option value="hypnotic_rings">hypnotic_rings</option>
                        <option value="pulsar_burst">pulsar_burst</option>
                        <option value="equalizer_bars">equalizer_bars (8-bar audio spectrum)</option>
                        <option value="galaxy_spiral">galaxy_spiral</option>
                        <option value="plasma_wave">plasma_wave</option>
                        <option value="matrix_rain">matrix_rain</option>
                        <option value="fireworks">fireworks</option>
                        <option value="rainbow_wave">rainbow_wave</option>
                        <option value="snake">snake</option>
                        <option value="pulse_beacon">pulse_beacon</option>
                        <option value="strobe_pulse">strobe_pulse</option>
                    </select>

                    <label>Átmenet:</label>
                    <select id="transSelect">
                        <option value="zoom_iris">zoom_iris</option>
                        <option value="dissolve">dissolve</option>
                        <option value="wipe_right">wipe_right</option>
                        <option value="wipe_down">wipe_down</option>
                        <option value="none">none</option>
                    </select>

                    <label>Egyedi szöveg (Futófelirathoz):</label>
                    <input type="text" id="bannerTextInput" value="HOPE CODE" style="width: 120px;">

                    <button class="btn-action" onclick="triggerAnim()">Futtatás Kijelzőn & Hardware-en</button>
                </div>
            </div>
        </div>

        <!-- 4. Indulás & Beállítások Tab -->
        <div id="settings" class="tab-content">
            <div class="card">
                <h3>Rendszer Beállítások</h3>
                <div class="controls-row">
                    <input type="checkbox" id="autostartCheck" onchange="toggleAutostart()">
                    <label for="autostartCheck"><strong>Automatikus Indulás a Windows 11 Rendszerrel (Startup)</strong></label>
                </div>
                <p style="color: #aaa; margin-top: 10px;">Automatikusa elindítja a háttérben futó Launchpad daemont a Windows bejelentkezéskor.</p>
            </div>
        </div>

        <!-- 5. Új Visual Készítése AI-val Tab -->
        <div id="ai-generator" class="tab-content">
            <div class="card">
                <h3>Prompt Prompt-Útmutató Más AI Eszközökhöz (Claude / ChatGPT)</h3>
                <p>Másold ki az alábbi specifikációt és illeszd be bármelyik AI modellbe új visual effekt írásához:</p>
                <pre id="promptTemplate">
Szia! Egy új visual animációt szeretnék készíteni a Novation Launchpad Mini MK3 (8x8 RGB LED Mátrix) kontrolleremhez Rust nyelven.

Specifikációk:
- Mátrix méret: 8x8 (Sor: 0..7, Oszlop: 0..7)
- Függvény szignatúra: `fn anim_my_custom_effect(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration)`
- Szín konstansok: COLOR_CYAN, COLOR_MAGENTA, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT, COLOR_RED_BRIGHT, COLOR_WHITE, COLOR_PURPLE, COLOR_BLUE, COLOR_ORANGE.
- Pad színezés: `lp_guard.set_grid_pad(row, col, color);`
- Frissítés: `thread::sleep(Duration::from_millis(60));`

Kérlek, írj egy látványos [EFFEKT NEVE] effektet a fenti struktúrával!
                </pre>
            </div>
        </div>
    </div>

    <script>
        let currentActiveApp = 'hopecode';

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
                document.getElementById('autostartCheck').checked = data.autostart_enabled;
            } catch(e) {}
        }

        async function triggerAnim() {
            const anim = document.getElementById('animSelect').value;
            const trans = document.getElementById('transSelect').value;
            const textVal = document.getElementById('bannerTextInput').value;
            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'command', app: currentActiveApp, animation: anim, transition: trans, text: textVal })
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
                body: JSON.stringify({ event: 'task_start', app: currentActiveApp, task_id: taskId, animation: 'galaxy_spiral', transition: 'zoom_iris' })
            });
        }

        async function toggleAutostart() {
            const chk = document.getElementById('autostartCheck').checked;
            await fetch('/api/settings', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ _autostart: chk })
            });
        }

        buildGrid();
        setInterval(fetchState, 500);
    </script>
</body>
</html>
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Control Panel] Starting HOPE CODE Multi-App Web Control Panel on http://127.0.0.1:8080...");

    let lp = LaunchpadMiniMK3::new(true);
    let engine = Arc::new(LaunchpadEngine::new(lp));

    let _engine_state = Arc::clone(&engine);
    let api_state = warp::path!("api" / "state").map(move || {
        let mut tasks_map = HashMap::new();
        tasks_map.insert(0, "pending".to_string());

        let mut apps_map = HashMap::new();
        apps_map.insert("hopecode".to_string(), "running".to_string());
        apps_map.insert("claude_code".to_string(), "running".to_string());
        apps_map.insert("codex".to_string(), "running".to_string());

        let resp = StateResponse {
            active_app: "hopecode".to_string(),
            tasks: tasks_map,
            apps_status: apps_map,
            animations: vec![
                "text_banner".into(), "equalizer_bars".into(), "galaxy_spiral".into(), "plasma_wave".into(), "matrix_rain".into(),
                "fireworks".into(), "rainbow_wave".into(), "snake".into()
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
        .or(api_state)
        .or(api_trigger)
        .or(api_speak)
        .or(api_settings);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
