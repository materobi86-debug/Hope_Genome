use std::net::UdpSocket;
use serde::Serialize;
use clap::Parser;

use launchpad_zcode_rs::config::{DEFAULT_HOST, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[command(author, version, about = "HOPE CODE Launchpad Mini MK3 CLI Notifier in Rust")]
struct Args {
    #[arg(help = "Event type (think, command, task_start, task_success, task_error, switch_app, reset)")]
    event: String,

    #[arg(long, default_value_t = 0, help = "Task index (0..63)")]
    task_id: usize,

    #[arg(long, help = "Target application (hopecode, claude_code, codex)")]
    app: Option<String>,

    #[arg(long, help = "Animation type")]
    anim: Option<String>,

    #[arg(long, default_value = "dissolve", help = "Transition effect type")]
    trans: String,

    #[arg(long, help = "Custom text for scrolling banner (e.g. HOPE CODE / STOP!!)")]
    text: Option<String>,

    #[arg(long, default_value = DEFAULT_HOST)]
    host: String,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Serialize)]
struct EventPayload<'a> {
    event: &'a str,
    task_id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    animation: Option<&'a str>,
    transition: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let payload = EventPayload {
        event: &args.event,
        task_id: args.task_id,
        app: args.app.as_deref(),
        animation: args.anim.as_deref(),
        transition: &args.trans,
        text: args.text.as_deref(),
    };

    let data = serde_json::to_vec(&payload)?;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target_addr = format!("{}:{}", args.host, args.port);
    socket.send_to(&data, target_addr)?;

    Ok(())
}
