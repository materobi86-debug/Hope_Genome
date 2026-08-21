"""
Tests for Launchpad Engine and Task State Manager.
"""

import time
import pytest
from launchpad_zcode.midi import LaunchpadMiniMK3
from launchpad_zcode.engine import LaunchpadEngine, TaskState
from launchpad_zcode.config import TASK_COLOR_PENDING, TASK_COLOR_SUCCESS, TASK_COLOR_ERROR

def test_task_id_grid_mapping():
    lp = LaunchpadMiniMK3(virtual=True)
    engine = LaunchpadEngine(lp)

    assert engine.task_id_to_grid(0) == (0, 0)
    assert engine.task_id_to_grid(7) == (0, 7)
    assert engine.task_id_to_grid(8) == (1, 0)
    assert engine.task_id_to_grid(63) == (7, 7)
    engine.stop()

def test_task_state_changes():
    lp = LaunchpadMiniMK3(virtual=True)
    lp.connect()
    engine = LaunchpadEngine(lp)

    engine.set_task_state(0, TaskState.SUCCESS)
    note_0 = LaunchpadMiniMK3.pad_to_note(0, 0)
    assert lp.buffer[note_0] == TASK_COLOR_SUCCESS

    engine.set_task_state(1, TaskState.ERROR)
    note_1 = LaunchpadMiniMK3.pad_to_note(0, 1)
    assert lp.buffer[note_1] == TASK_COLOR_ERROR

    engine.stop()

def test_animation_trigger():
    lp = LaunchpadMiniMK3(virtual=True)
    lp.connect()
    engine = LaunchpadEngine(lp)

    engine.animate_operation(anim_type="spinner", duration=0.2)
    time.sleep(0.3)
    assert engine._animating is False

    engine.stop()
