use std::net::SocketAddr;
use tokio::net::UdpSocket;
use serde::Deserialize;
use clap::Parser;

use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState, AppTarget};
use launchpad_zcode_rs::config::{DEFAULT_HOST, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[command(author, version, about = "HOPE CODE Launchpad Mini MK3 Daemon in Rust")]
struct Args {
    #[arg(long, default_value = DEFAULT_HOST)]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    #[arg(long, default_value_t = false)]
    virtual_mode: bool,
}

#[derive(Deserialize, Debug)]
struct EventPayload {
    event: String,
    #[serde(default)]
    task_id: usize,
    #[serde(default)]
    app: Option<String>,
    #[serde(default)]
    animation: Option<String>,
    #[serde(default)]
    transition: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut lp = LaunchpadMiniMK3::new(args.virtual_mode);
    let _ = lp.connect();

    let engine = LaunchpadEngine::new(lp);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let socket = UdpSocket::bind(addr).await?;

    println!("[Daemon] HOPE CODE Launchpad Rust Daemon listening on {}", addr);

    let mut buf = [0u8; 4096];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _src)) => {
                if let Ok(payload) = serde_json::from_slice::<EventPayload>(&buf[..len]) {
                    println!("[Daemon] Received payload: {:?}", payload);

                    let event = payload.event.to_lowercase();
                    let anim = payload.animation.as_deref().unwrap_or("spinner");
                    let trans = payload.transition.as_deref().unwrap_or("dissolve");
                    let text = payload.text.as_deref().unwrap_or("HOPE CODE");

                    let app_target = if let Some(app_str) = payload.app.as_deref() {
                        AppTarget::from_str(app_str)
                    } else {
                        AppTarget::HopeCode
                    };

                    match event.as_str() {
                        "switch_app" | "select_app" => {
                            engine.set_active_app(app_target);
                        }
                        "task_start" | "start" | "active" | "think" => {
                            engine.set_task_state_for_app(app_target, payload.task_id, TaskState::Active);
                            engine.animate_operation_with_text(anim, trans, 1.2, text);
                        }
                        "task_success" | "success" | "done" => {
                            engine.set_task_state_for_app(app_target, payload.task_id, TaskState::Success);
                            engine.animate_operation_with_text(anim, trans, 1.2, text);
                        }
                        "task_error" | "error" | "fail" => {
                            engine.set_task_state_for_app(app_target, payload.task_id, TaskState::Error);
                            engine.animate_operation_with_text("text_banner", trans, 2.5, "STOP!!");
                        }
                        "reset" => {
                            engine.reset();
                        }
                        _ => {
                            engine.animate_operation_with_text(anim, trans, 1.2, text);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Daemon] Socket error: {}", e);
            }
        }
    }
}
