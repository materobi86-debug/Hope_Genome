"""
Tests for Daemon and CLI Hook notification communication.
"""

import time
import threading
import pytest
from launchpad_zcode.daemon import LaunchpadDaemon
from launchpad_zcode.cli import send_notification
from launchpad_zcode.engine import TaskState
from launchpad_zcode.config import TASK_COLOR_SUCCESS, TASK_COLOR_ERROR
from launchpad_zcode.midi import LaunchpadMiniMK3

def test_daemon_cli_communication():
    port = 9879
    daemon = LaunchpadDaemon(port=port, virtual=True)

    daemon_thread = threading.Thread(target=daemon.start, daemon=True)
    daemon_thread.start()

    time.sleep(0.1)

    # 1. Send task_start with no animation for deterministic state check
    send_notification("task_start", task_id=3, animation="none", port=port)
    time.sleep(0.2)
    assert daemon.engine.tasks[3] == TaskState.ACTIVE

    # 2. Send task_success with no animation
    send_notification("task_success", task_id=3, animation="none", port=port)
    time.sleep(0.2)
    assert daemon.engine.tasks[3] == TaskState.SUCCESS

    # Check color buffer on Launchpad (task_id 3 -> row 0, col 3 -> note 84)
    note = LaunchpadMiniMK3.pad_to_note(0, 3)
    assert daemon.lp.buffer[note] == TASK_COLOR_SUCCESS

    daemon.stop()
