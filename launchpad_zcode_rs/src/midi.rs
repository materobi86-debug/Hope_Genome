use std::collections::HashMap;
use midir::{MidiOutput, MidiOutputConnection};
use crate::config::*;

pub struct LaunchpadMiniMK3 {
    conn: Option<MidiOutputConnection>,
    virtual_mode: bool,
    pub buffer: HashMap<u8, u8>,
}

impl LaunchpadMiniMK3 {
    pub fn new(virtual_mode: bool) -> Self {
        Self {
            conn: None,
            virtual_mode,
            buffer: HashMap::new(),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        if self.virtual_mode {
            println!("[MIDI] Launchpad running in virtual mode.");
            return Ok(());
        }

        let midi_out = MidiOutput::new("zcode_launchpad_out").map_err(|e| e.to_string())?;
        let out_ports = midi_out.ports();

        let mut target_port = None;
        for port in &out_ports {
            if let Ok(name) = midi_out.port_name(port) {
                if DEVICE_NAME_KEYWORDS.iter().any(|kw| name.to_lowercase().contains(&kw.to_lowercase())) {
                    target_port = Some(port.clone());
                    println!("[MIDI] Found Launchpad port: {}", name);
                    break;
                }
            }
        }

        if let Some(port) = target_port {
            let conn = midi_out.connect(&port, "launchpad_out").map_err(|e| e.to_string())?;
            self.conn = Some(conn);
            self.enter_programmer_mode();
            println!("[MIDI] Connected and switched to Programmer Mode.");
            Ok(())
        } else {
            println!("[MIDI] No Launchpad device found. Falling back to virtual mode.");
            self.virtual_mode = true;
            Ok(())
        }
    }

    pub fn enter_programmer_mode(&mut self) {
        if let Some(ref mut conn) = self.conn {
            let _ = conn.send(SYSEX_MODE_PROGRAMMER);
        }
    }

    pub fn exit_programmer_mode(&mut self) {
        if let Some(ref mut conn) = self.conn {
            let _ = conn.send(SYSEX_MODE_LIVE);
        }
    }

    pub fn pad_to_note(row: u8, col: u8) -> u8 {
        assert!(row < 8 && col < 8, "Row and Col must be 0..7");
        10 * (8 - row) + (col + 1)
    }

    pub fn note_to_pad(note: u8) -> Option<(u8, u8)> {
        let row_tens = note / 10;
        let col_ones = note % 10;
        if (1..=8).contains(&row_tens) && (1..=8).contains(&col_ones) {
            let row = 8 - row_tens;
            let col = col_ones - 1;
            Some((row, col))
        } else {
            None
        }
    }

    pub fn set_led(&mut self, note_or_cc: u8, color_velocity: u8, is_cc: bool) {
        self.buffer.insert(note_or_cc, color_velocity);
        if let Some(ref mut conn) = self.conn {
            let msg = if is_cc {
                [0xB0, note_or_cc, color_velocity]
            } else {
                [0x90, note_or_cc, color_velocity]
            };
            let _ = conn.send(&msg);
        }
    }

    pub fn set_grid_pad(&mut self, row: u8, col: u8, color_velocity: u8) {
        let note = Self::pad_to_note(row, col);
        self.set_led(note, color_velocity, false);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        for row in 0..8 {
            for col in 0..8 {
                let note = Self::pad_to_note(row, col);
                self.set_led(note, COLOR_OFF, false);
            }
        }
    }
}
