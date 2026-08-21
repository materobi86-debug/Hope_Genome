"""
Background UDP/TCP Daemon Server for Zcode Launchpad Mini MK3 Integration.
Receives JSON event payloads from Zcode CLI hooks and updates Launchpad state.
"""

import socket
import json
import logging
import argparse
import sys
from typing import Dict, Any, Optional

from .midi import LaunchpadMiniMK3
from .engine import LaunchpadEngine, TaskState
from .config import DEFAULT_HOST, DEFAULT_PORT

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger(__name__)


class LaunchpadDaemon:
    """
    Listens for Zcode event notifications via UDP socket and coordinates
    the Launchpad Mini MK3 display engine.
    """

    def __init__(self, host: str = DEFAULT_HOST, port: int = DEFAULT_PORT, virtual: bool = False):
        self.host = host
        self.port = port
        self.virtual = virtual
        self.lp = LaunchpadMiniMK3(virtual=virtual)
        self.engine: Optional[LaunchpadEngine] = None
        self.running = False

    def handle_event(self, data: Dict[str, Any]):
        """
        Process event payload from Zcode hooks.
        Payload format:
        {
            "event": "think" | "command" | "task_start" | "task_success" | "task_error" | "reset",
            "task_id": 0..63,
            "animation": "spinner" | "scan" | "success_ripple" | "error_flash" | "none"
        }
        """
        event = data.get("event", "").lower()
        task_id = data.get("task_id", 0)
        animation = data.get("animation")

        logger.info(f"Received Zcode event: {event} (task_id: {task_id}, anim: {animation})")

        if not self.engine:
            return

        # 1. Update task state if event is task related
        if event in ("task_start", "start", "active", "think"):
            self.engine.set_task_state(task_id, TaskState.ACTIVE)
            if not animation:
                animation = "spinner"
        elif event in ("task_success", "success", "done"):
            self.engine.set_task_state(task_id, TaskState.SUCCESS)
            if not animation:
                animation = "success_ripple"
        elif event in ("task_error", "error", "fail"):
            self.engine.set_task_state(task_id, TaskState.ERROR)
            if not animation:
                animation = "error_flash"
        elif event in ("task_pending", "pending"):
            self.engine.set_task_state(task_id, TaskState.PENDING)
        elif event == "reset":
            for i in range(64):
                self.engine.set_task_state(i, TaskState.PENDING)

        # 2. Trigger animation if specified or inferred
        if animation and animation != "none":
            self.engine.animate_operation(anim_type=animation)

    def start(self):
        """Start daemon server loop."""
        self.lp.connect()
        self.engine = LaunchpadEngine(self.lp)
        self.running = True

        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.bind((self.host, self.port))
        sock.settimeout(1.0)

        logger.info(f"Zcode Launchpad Daemon running on {self.host}:{self.port} (Virtual={self.virtual})")

        try:
            while self.running:
                try:
                    raw_data, addr = sock.recvfrom(4096)
                    try:
                        payload = json.loads(raw_data.decode("utf-8"))
                        self.handle_event(payload)
                    except json.JSONDecodeError:
                        logger.warning(f"Invalid JSON received from {addr}")
                except socket.timeout:
                    continue
        except KeyboardInterrupt:
            logger.info("Daemon stopping...")
        finally:
            self.stop()
            sock.close()

    def stop(self):
        self.running = False
        if self.engine:
            self.engine.stop()
        if self.lp:
            self.lp.disconnect()


def main():
    parser = argparse.ArgumentParser(description="Zcode Launchpad Mini MK3 Daemon")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Host to bind daemon socket")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to bind daemon socket")
    parser.add_argument("--virtual", action="store_true", help="Run in virtual mode without physical Launchpad")

    args = parser.parse_args()
    daemon = LaunchpadDaemon(host=args.host, port=args.port, virtual=args.virtual)
    daemon.start()


if __name__ == "__main__":
    main()
