#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::mpsc;
    use std::thread;
    use std::net::SocketAddr;
    use tokio::net::UdpSocket;
    use serde::Deserialize;
    use tray_item::{TrayItem, IconSource};

    use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
    use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState};
    use launchpad_zcode_rs::config::{DEFAULT_HOST, DEFAULT_PORT};

    #[derive(Deserialize, Debug)]
    struct EventPayload {
        event: String,
        #[serde(default)]
        task_id: usize,
        #[serde(default)]
        animation: Option<String>,
        #[serde(default)]
        transition: Option<String>,
    }

    enum TrayMessage {
        Quit,
    }

    println!("[Tray] Starting Zcode Launchpad System Tray Application for Windows 11...");

    let tray_res = TrayItem::new("Zcode Launchpad Mini MK3", IconSource::Resource("main_icon"));
    let mut tray = match tray_res {
        Ok(item) => item,
        Err(_) => {
            // Fallback gracefully to default executable icon
            TrayItem::new("Zcode Launchpad Mini MK3", IconSource::Resource(""))
                .unwrap_or_else(|_| panic!("Failed to initialize Windows system tray item."))
        }
    };

    let (tx, rx) = mpsc::channel();

    let tx_quit = tx.clone();
    tray.add_menu_item("Quit", move || {
        let _ = tx_quit.send(TrayMessage::Quit);
    })?;

    // Spawn async Tokio runtime for UDP daemon handling
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut lp = LaunchpadMiniMK3::new(false);
            let _ = lp.connect();
            let engine = LaunchpadEngine::new(lp);

            let addr_str = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT);
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                if let Ok(socket) = UdpSocket::bind(addr).await {
                    println!("[Daemon] Tray UDP listener bound on {}", addr);
                    let mut buf = [0u8; 4096];
                    loop {
                        if let Ok((len, _)) = socket.recv_from(&mut buf).await {
                            if let Ok(payload) = serde_json::from_slice::<EventPayload>(&buf[..len]) {
                                let event = payload.event.to_lowercase();
                                let anim = payload.animation.as_deref().unwrap_or("spinner");
                                let trans = payload.transition.as_deref().unwrap_or("dissolve");

                                match event.as_str() {
                                    "task_start" | "start" | "active" | "think" => {
                                        engine.set_task_state(payload.task_id, TaskState::Active);
                                        engine.animate_operation(anim, trans, 0.8);
                                    }
                                    "task_success" | "success" | "done" => {
                                        engine.set_task_state(payload.task_id, TaskState::Success);
                                        engine.animate_operation(anim, trans, 0.8);
                                    }
                                    "task_error" | "error" | "fail" => {
                                        engine.set_task_state(payload.task_id, TaskState::Error);
                                        engine.animate_operation(anim, trans, 0.8);
                                    }
                                    "reset" => {
                                        engine.reset();
                                    }
                                    _ => {
                                        engine.animate_operation(anim, trans, 0.8);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    });

    loop {
        match rx.recv() {
            Ok(TrayMessage::Quit) => {
                println!("[Tray] Exiting...");
                break;
            }
            Err(_) => break,
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    println!("[Tray] System tray GUI binary is designed for Windows 11 target.");
    println!("[Tray] Please run `zcode-launchpad-daemon` on Linux/macOS.");
}
