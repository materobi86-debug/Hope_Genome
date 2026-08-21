use std::net::UdpSocket;
use serde::Serialize;
use clap::Parser;

use launchpad_zcode_rs::config::{DEFAULT_HOST, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[command(author, version, about = "Zcode Launchpad Mini MK3 CLI Notifier in Rust")]
struct Args {
    #[arg(help = "Event type (think, command, task_start, task_success, task_error, reset)")]
    event: String,

    #[arg(long, default_value_t = 0, help = "Task index (0..63)")]
    task_id: usize,

    #[arg(long, help = "Animation type")]
    anim: Option<String>,

    #[arg(long, default_value = "dissolve", help = "Transition effect type")]
    trans: String,

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
    animation: Option<&'a str>,
    transition: &'a str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let payload = EventPayload {
        event: &args.event,
        task_id: args.task_id,
        animation: args.anim.as_deref(),
        transition: &args.trans,
    };

    let data = serde_json::to_vec(&payload)?;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let target_addr = format!("{}:{}", args.host, args.port);
    socket.send_to(&data, target_addr)?;

    Ok(())
}
