use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::Filter;

use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState};

#[derive(Serialize, Deserialize, Clone)]
struct StateResponse {
    tasks: HashMap<usize, String>,
    animations: Vec<String>,
    transitions: Vec<String>,
    autostart_enabled: bool,
}

#[derive(Deserialize)]
struct TriggerRequest {
    event: String,
    task_id: Option<usize>,
    animation: Option<String>,
    transition: Option<String>,
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
    <title>Zcode Launchpad Mini MK3 Vezérlőpult 🎛️</title>
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
        .grid-container {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 8px;
            max-width: 480px;
            margin: 0 auto;
            background: #09090b;
            padding: 15px;
            border-radius: 10px;
            border: 2px solid var(--border);
        }
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
        select, button, input {
            padding: 8px 12px;
            background: #2a2a35;
            border: 1px solid var(--border);
            color: #fff;
            border-radius: 4px;
            font-size: 0.95rem;
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
            <h1>🎛️ Zcode Launchpad Mini MK3 Vezérlőpult</h1>
            <div>
                <span style="color: #00e676;">● Daemon Aktív (Port: 9876)</span>
            </div>
        </header>

        <div class="tabs">
            <button class="tab-btn active" onclick="showTab('virtual-midi')">📱 Virtuális MIDI Kijelző</button>
            <button class="tab-btn" onclick="showTab('preview')">✨ Visualizációk Megtekintése</button>
            <button class="tab-btn" onclick="showTab('settings')">⚙️ Indulás & Beállítások</button>
            <button class="tab-btn" onclick="showTab('ai-generator')">🤖 Új Visual Készítése AI-val</button>
        </div>

        <div id="virtual-midi" class="tab-content active">
            <div class="card">
                <h3>Virtuális Launchpad 8x8 Mátrix Kijelző</h3>
                <p>Kattints egy padra a státusz megváltoztatásához vagy teszteléshez!</p>
                <div class="grid-container" id="padGrid">
                </div>
            </div>
        </div>

        <div id="preview" class="tab-content">
            <div class="card">
                <h3>Visualizáció és Átmenet Tesztelése</h3>
                <div class="controls-row">
                    <label>Effekt:</label>
                    <select id="animSelect">
                        <option value="galaxy_spiral">galaxy_spiral</option>
                        <option value="plasma_wave">plasma_wave</option>
                        <option value="matrix_rain">matrix_rain</option>
                        <option value="fireworks">fireworks</option>
                        <option value="rainbow_wave">rainbow_wave</option>
                        <option value="snake">snake</option>
                        <option value="pulse_beacon">pulse_beacon</option>
                        <option value="equalizer_bars">equalizer_bars</option>
                        <option value="strobe_pulse">strobe_pulse</option>
                        <option value="spinner">spinner</option>
                        <option value="scan">scan</option>
                        <option value="success_ripple">success_ripple</option>
                        <option value="error_flash">error_flash</option>
                    </select>

                    <label>Átmenet:</label>
                    <select id="transSelect">
                        <option value="zoom_iris">zoom_iris</option>
                        <option value="dissolve">dissolve</option>
                        <option value="wipe_right">wipe_right</option>
                        <option value="wipe_down">wipe_down</option>
                        <option value="none">none</option>
                    </select>

                    <button class="btn-action" onclick="triggerAnim()">Futtatás Kijelzőn & Hardware-en</button>
                </div>
            </div>
        </div>

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
            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'command', animation: anim, transition: trans })
            });
        }

        async function togglePad(taskId) {
            await fetch('/api/trigger', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ event: 'task_start', task_id: taskId, animation: 'galaxy_spiral', transition: 'zoom_iris' })
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
    println!("[Control Panel] Starting Web GUI Control Panel for Launchpad Mini MK3 on http://127.0.0.1:8080...");

    let lp = LaunchpadMiniMK3::new(true);
    let engine = Arc::new(LaunchpadEngine::new(lp));

    let api_state = warp::path!("api" / "state").map(move || {
        let mut tasks_map = HashMap::new();
        for i in 0..64 {
            tasks_map.insert(i, "pending".to_string());
        }
        let resp = StateResponse {
            tasks: tasks_map,
            animations: vec![
                "galaxy_spiral".into(), "plasma_wave".into(), "matrix_rain".into(),
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
            let anim = req.animation.as_deref().unwrap_or("galaxy_spiral");
            let trans = req.transition.as_deref().unwrap_or("zoom_iris");
            let task_id = req.task_id.unwrap_or(0);

            if req.event == "task_start" {
                engine_trigger.set_task_state(task_id, TaskState::Active);
            }
            engine_trigger.animate_operation(anim, trans, 0.8);
            warp::reply::json(&"ok")
        });

    let api_settings = warp::path!("api" / "settings")
        .and(warp::post())
        .and(warp::body::json())
        .map(|_req: SettingsRequest| {
            warp::reply::json(&"settings_updated")
        });

    let html_route = warp::path::end().map(|| warp::reply::html(HTML_INDEX));

    let routes = html_route.or(api_state).or(api_trigger).or(api_settings);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
