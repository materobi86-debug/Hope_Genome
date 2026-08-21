"""
CLI Notify Tool for Zcode Hook Integration.
Called by Zcode event hooks to notify the Launchpad daemon.
Usage:
    zcode-launchpad-notify <event> [--task-id 0] [--anim spinner]
"""

import socket
import json
import argparse
import sys
from .config import DEFAULT_HOST, DEFAULT_PORT


def send_notification(event: str, task_id: int = 0, animation: str = None, host: str = DEFAULT_HOST, port: int = DEFAULT_PORT):
    """Send UDP notification to Launchpad daemon."""
    payload = {
        "event": event,
        "task_id": task_id
    }
    if animation:
        payload["animation"] = animation

    data = json.dumps(payload).encode("utf-8")

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.sendto(data, (host, port))
    except Exception as e:
        sys.stderr.write(f"Failed to send Launchpad notification: {e}\n")
    finally:
        sock.close()


def main():
    parser = argparse.ArgumentParser(description="Notify Zcode Launchpad Mini MK3 Daemon")
    parser.add_argument("event", help="Event type (think, command, task_start, task_success, task_error, reset)")
    parser.add_argument("--task-id", type=int, default=0, help="Task index (0..63)")
    parser.add_argument("--anim", type=str, default=None, choices=["spinner", "scan", "success_ripple", "error_flash", "none"], help="Animation type")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Daemon host")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Daemon port")

    args = parser.parse_args()
    send_notification(
        event=args.event,
        task_id=args.task_id,
        animation=args.anim,
        host=args.host,
        port=args.port
    )


if __name__ == "__main__":
    main()
