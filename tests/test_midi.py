"""
Tests for Launchpad Mini MK3 MIDI mapping and controller logic.
"""

import pytest
from launchpad_zcode.midi import LaunchpadMiniMK3
from launchpad_zcode.config import COLOR_RED_BRIGHT, COLOR_GREEN_BRIGHT

def test_pad_note_conversion():
    # Top-left (0,0) -> 81
    assert LaunchpadMiniMK3.pad_to_note(0, 0) == 81
    # Top-right (0,7) -> 88
    assert LaunchpadMiniMK3.pad_to_note(0, 7) == 88
    # Bottom-left (7,0) -> 11
    assert LaunchpadMiniMK3.pad_to_note(7, 0) == 11
    # Bottom-right (7,7) -> 18
    assert LaunchpadMiniMK3.pad_to_note(7, 7) == 18

    # Reverse conversion
    assert LaunchpadMiniMK3.note_to_pad(81) == (0, 0)
    assert LaunchpadMiniMK3.note_to_pad(88) == (0, 7)
    assert LaunchpadMiniMK3.note_to_pad(11) == (7, 0)
    assert LaunchpadMiniMK3.note_to_pad(18) == (7, 7)

def test_virtual_controller():
    lp = LaunchpadMiniMK3(virtual=True)
    assert lp.connect() is True
    assert lp.connected is True

    lp.set_grid_pad(0, 0, COLOR_RED_BRIGHT)
    note_0_0 = LaunchpadMiniMK3.pad_to_note(0, 0)
    assert lp.buffer[note_0_0] == COLOR_RED_BRIGHT

    lp.clear()
    assert len(lp.buffer) == 0
    lp.disconnect()
    assert lp.connected is False
