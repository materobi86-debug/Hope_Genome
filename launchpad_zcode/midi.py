"""
Launchpad Mini MK3 MIDI Interface Controller module.
Handles MIDI device discovery, SysEx Programmer Mode activation,
LED lighting, and grid pad mapping.
"""

import logging
from typing import Optional, List, Tuple, Dict
import mido

from .config import (
    DEVICE_NAME_KEYWORDS,
    MODE_PROGRAMMER,
    MODE_LIVE,
    COLOR_OFF,
    SYSEX_HEADER,
    SYSEX_FOOTER,
)

logger = logging.getLogger(__name__)


class LaunchpadMiniMK3:
    """
    Controller for Novation Launchpad Mini MK3 in Programmer Mode.
    Layout in Programmer Mode:
    Grid pads (8x8):
      - Top-left pad (row 0, col 0) = Note 81 (or 11 in Note mode, 81 in MIDI grid standard layout: 10*row + col + 11)
      - Standard Programmer Mode Grid layout:
        Row 0 (top) to Row 7 (bottom), Col 0 (left) to Col 7 (right).
        Pad note index = (8 - row) * 10 + col + 1 = 10 * (8 - row) + col + 1 (i.e. Row 0: 81..88, Row 7: 11..18).
      - Top logo / Mode buttons (CC 91..98)
      - Right side scene buttons (Note 19, 29, 39, 49, 59, 69, 79, 89)
    """

    def __init__(self, port_name: Optional[str] = None, virtual: bool = False):
        self.port_name = port_name
        self.virtual = virtual
        self.inport = None
        self.outport = None
        self.connected = False
        self.buffer: Dict[int, int] = {}  # Index -> velocity color

    def find_ports(self) -> Tuple[Optional[str], Optional[str]]:
        """Find MIDI input and output ports for Launchpad Mini MK3."""
        input_names = mido.get_input_names()
        output_names = mido.get_output_names()

        in_port = None
        out_port = None

        for name in input_names:
            if any(kw.lower() in name.lower() for kw in DEVICE_NAME_KEYWORDS):
                # Prefer DAW / MIDI port
                in_port = name
                break

        for name in output_names:
            if any(kw.lower() in name.lower() for kw in DEVICE_NAME_KEYWORDS):
                out_port = name
                break

        return in_port, out_port

    def connect(self) -> bool:
        """Connect to the MIDI ports and enter Programmer Mode."""
        if self.virtual:
            logger.info("Running LaunchpadMiniMK3 in virtual/mock mode.")
            self.connected = True
            return True

        try:
            in_name, out_name = self.find_ports()
            if self.port_name:
                out_name = self.port_name
                in_name = self.port_name

            if not out_name:
                logger.warning("Launchpad Mini MK3 output port not found.")
                return False

            self.outport = mido.open_output(out_name)
            if in_name:
                self.inport = mido.open_input(in_name)

            self.connected = True
            self.enter_programmer_mode()
            logger.info(f"Connected to Launchpad Mini MK3 on port: {out_name}")
            return True
        except Exception as e:
            logger.error(f"Failed to connect to Launchpad Mini MK3: {e}")
            self.connected = False
            return False

    def disconnect(self):
        """Disconnect and reset to Live mode."""
        if self.connected:
            self.exit_programmer_mode()
            self.clear()
            if self.outport:
                self.outport.close()
            if self.inport:
                self.inport.close()
            self.connected = False
            logger.info("Disconnected from Launchpad Mini MK3.")

    def send_sysex(self, data: List[int]):
        """Send SysEx raw message to device."""
        if not self.connected or not self.outport:
            return
        msg = mido.Message('sysex', data=data[1:-1])  # mido sysex excludes F0 and F7
        self.outport.send(msg)

    def enter_programmer_mode(self):
        """Switch Launchpad Mini MK3 into Programmer Mode for raw LED control."""
        self.send_sysex(MODE_PROGRAMMER)

    def exit_programmer_mode(self):
        """Switch Launchpad Mini MK3 back to Live Mode."""
        self.send_sysex(MODE_LIVE)

    @staticmethod
    def pad_to_note(row: int, col: int) -> int:
        """
        Convert 0-indexed (row, col) (row 0-7, col 0-7) to Programmer Mode MIDI note.
        Row 0 is top row, Col 0 is left column.
        Pad Note = 10 * (8 - row) + (col + 1).
        E.g. Row 0, Col 0 -> 81. Row 7, Col 7 -> 18.
        """
        if not (0 <= row <= 7 and 0 <= col <= 7):
            raise ValueError(f"Invalid row/col: ({row}, {col}). Must be 0..7.")
        return 10 * (8 - row) + (col + 1)

    @staticmethod
    def note_to_pad(note: int) -> Optional[Tuple[int, int]]:
        """Convert MIDI note back to (row, col) if inside the 8x8 grid."""
        row_tens = note // 10
        col_ones = note % 10
        if 1 <= row_tens <= 8 and 1 <= col_ones <= 8:
            row = 8 - row_tens
            col = col_ones - 1
            return (row, col)
        return None

    def set_led(self, note_or_cc: int, color_velocity: int, is_cc: bool = False):
        """Set an individual pad or control LED color."""
        self.buffer[note_or_cc] = color_velocity
        if not self.connected or not self.outport:
            return

        if is_cc:
            msg = mido.Message('control_change', control=note_or_cc, value=color_velocity)
        else:
            msg = mido.Message('note_on', note=note_or_cc, velocity=color_velocity)
        self.outport.send(msg)

    def set_grid_pad(self, row: int, col: int, color_velocity: int):
        """Set an 8x8 grid pad LED color by row and column."""
        note = self.pad_to_note(row, col)
        self.set_led(note, color_velocity, is_cc=False)

    def set_batch_leds(self, led_colors: List[Tuple[int, int, bool]]):
        """
        Set multiple LEDs in batch.
        List of tuples: (note_or_cc, color_velocity, is_cc)
        """
        for note_or_cc, color, is_cc in led_colors:
            self.set_led(note_or_cc, color, is_cc)

    def clear(self):
        """Turn off all LEDs on the matrix."""
        self.buffer.clear()
        if not self.connected or not self.outport:
            return

        # Turn off all 8x8 grid pads & side buttons via MIDI notes 11..89
        for row in range(8):
            for col in range(8):
                note = self.pad_to_note(row, col)
                self.set_led(note, COLOR_OFF, is_cc=False)

        # Clear right side scene buttons (19, 29, 39, 49, 59, 69, 79, 89)
        for row in range(8):
            scene_note = 10 * (8 - row) + 9
            self.set_led(scene_note, COLOR_OFF, is_cc=False)

        # Clear top CC buttons (91..98)
        for cc in range(91, 99):
            self.set_led(cc, COLOR_OFF, is_cc=True)
