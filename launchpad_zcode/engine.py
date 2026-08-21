"""
Animation Engine and Task State Grid Manager for Launchpad Mini MK3.
Renders task states on the 8x8 matrix and performs visual animations on operations.
"""

import time
import threading
import logging
from typing import Dict, Optional, List, Tuple
from .midi import LaunchpadMiniMK3
from .config import (
    COLOR_OFF,
    COLOR_GREY_DIM,
    COLOR_YELLOW_BRIGHT,
    COLOR_YELLOW_DIM,
    COLOR_GREEN_BRIGHT,
    COLOR_RED_BRIGHT,
    COLOR_CYAN,
    COLOR_WHITE,
    TASK_COLOR_PENDING,
    TASK_COLOR_ACTIVE,
    TASK_COLOR_SUCCESS,
    TASK_COLOR_ERROR,
)

logger = logging.getLogger(__name__)


class TaskState:
    PENDING = "pending"
    ACTIVE = "active"
    SUCCESS = "success"
    ERROR = "error"


class LaunchpadEngine:
    """
    Manages task grid state (8x8 pads mapped to tasks 0..63)
    and executes temporary visual animations for active operations.
    """

    def __init__(self, launchpad: LaunchpadMiniMK3):
        self.lp = launchpad
        # Map task_id (0..63) -> state ('pending', 'active', 'success', 'error')
        self.tasks: Dict[int, str] = {i: TaskState.PENDING for i in range(64)}
        self._lock = threading.Lock()
        self._animating = False
        self._pulsing_thread = None
        self._running = True

        # Start background pulse thread for active tasks
        self._pulse_thread = threading.Thread(target=self._pulse_loop, daemon=True)
        self._pulse_thread.start()

    def stop(self):
        self._running = False

    def task_id_to_grid(self, task_id: int) -> Tuple[int, int]:
        """Convert task ID (0..63) to 8x8 grid (row, col)."""
        task_id = task_id % 64
        row = task_id // 8
        col = task_id % 8
        return (row, col)

    def set_task_state(self, task_id: int, state: str):
        """Update a task's state."""
        with self._lock:
            if 0 <= task_id < 64:
                self.tasks[task_id] = state
        self.render_grid()

    def get_color_for_state(self, state: str, pulse_bright: bool = True) -> int:
        if state == TaskState.PENDING:
            return TASK_COLOR_PENDING
        elif state == TaskState.ACTIVE:
            return TASK_COLOR_ACTIVE if pulse_bright else COLOR_YELLOW_DIM
        elif state == TaskState.SUCCESS:
            return TASK_COLOR_SUCCESS
        elif state == TaskState.ERROR:
            return TASK_COLOR_ERROR
        return COLOR_OFF

    def render_grid(self, pulse_bright: bool = True):
        """Render main task matrix state to Launchpad."""
        if self._animating:
            return

        with self._lock:
            for task_id, state in self.tasks.items():
                row, col = self.task_id_to_grid(task_id)
                color = self.get_color_for_state(state, pulse_bright)
                self.lp.set_grid_pad(row, col, color)

    def _pulse_loop(self):
        """Background loop to pulse active tasks."""
        toggle = False
        while self._running:
            has_active = any(s == TaskState.ACTIVE for s in self.tasks.values())
            if has_active and not self._animating:
                toggle = not toggle
                self.render_grid(pulse_bright=toggle)
            time.sleep(0.5)

    def animate_operation(self, anim_type: str = "spinner", duration: float = 0.8):
        """
        Trigger an operation animation on the Launchpad matrix.
        Supported types: 'spinner', 'scan', 'success_ripple', 'error_flash'
        """
        def _run_anim():
            self._animating = True
            try:
                if anim_type == "spinner":
                    self._anim_spinner(duration)
                elif anim_type == "scan":
                    self._anim_scan(duration)
                elif anim_type == "success_ripple":
                    self._anim_ripple(COLOR_GREEN_BRIGHT, duration)
                elif anim_type == "error_flash":
                    self._anim_flash(COLOR_RED_BRIGHT, duration)
                else:
                    self._anim_spinner(duration)
            finally:
                self._animating = False
                self.render_grid()

        anim_thread = threading.Thread(target=_run_anim, daemon=True)
        anim_thread.start()

    def _anim_spinner(self, duration: float):
        """Rotating spinner around inner 4x4 matrix."""
        coords = [
            (2, 2), (2, 3), (2, 4), (2, 5),
            (3, 5), (4, 5), (5, 5),
            (5, 4), (5, 3), (5, 2),
            (4, 2), (3, 2)
        ]
        start_time = time.time()
        idx = 0
        while time.time() - start_time < duration:
            self.lp.clear()
            r, c = coords[idx % len(coords)]
            self.lp.set_grid_pad(r, c, COLOR_CYAN)
            idx += 1
            time.sleep(0.06)

    def _anim_scan(self, duration: float):
        """Scanning line top to bottom."""
        start_time = time.time()
        row = 0
        direction = 1
        while time.time() - start_time < duration:
            self.lp.clear()
            for col in range(8):
                self.lp.set_grid_pad(row, col, COLOR_WHITE)
            row += direction
            if row > 7 or row < 0:
                direction *= -1
                row += direction * 2
            time.sleep(0.08)

    def _anim_ripple(self, color: int, duration: float):
        """Expanding ripple from center."""
        rings = [
            [(3, 3), (3, 4), (4, 3), (4, 4)],
            [(2, 2), (2, 3), (2, 4), (2, 5), (3, 2), (3, 5), (4, 2), (4, 5), (5, 2), (5, 3), (5, 4), (5, 5)],
            [(1, 1), (1, 6), (6, 1), (6, 6)],
            [(0, 0), (0, 7), (7, 0), (7, 7)]
        ]
        step_delay = duration / len(rings)
        for ring in rings:
            self.lp.clear()
            for r, c in ring:
                self.lp.set_grid_pad(r, c, color)
            time.sleep(step_delay)

    def _anim_flash(self, color: int, duration: float):
        """Flash entire grid."""
        flash_count = 3
        delay = duration / (flash_count * 2)
        for _ in range(flash_count):
            for r in range(8):
                for c in range(8):
                    self.lp.set_grid_pad(r, c, color)
            time.sleep(delay)
            self.lp.clear()
            time.sleep(delay)
