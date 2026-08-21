// Launchpad Mini MK3 color velocities and SysEx constants

pub const DEVICE_NAME_KEYWORDS: &[&str] = &["Launchpad Mini MK3", "LPMiniMK3", "Launchpad"];

// SysEx headers for Novation Launchpad Mini MK3 Programmer Mode
pub const SYSEX_MODE_PROGRAMMER: &[u8] = &[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x0E, 0x01, 0xF7];
pub const SYSEX_MODE_LIVE: &[u8] = &[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x0E, 0x00, 0xF7];

// Color Palette Constants
pub const COLOR_OFF: u8 = 0;
pub const COLOR_GREY_DIM: u8 = 1;
pub const COLOR_GREY: u8 = 2;
pub const COLOR_WHITE: u8 = 3;
pub const COLOR_RED_DIM: u8 = 5;
pub const COLOR_RED: u8 = 6;
pub const COLOR_RED_BRIGHT: u8 = 7;
pub const COLOR_ORANGE: u8 = 9;
pub const COLOR_YELLOW_DIM: u8 = 12;
pub const COLOR_YELLOW_BRIGHT: u8 = 15;
pub const COLOR_GREEN_DIM: u8 = 17;
pub const COLOR_GREEN_BRIGHT: u8 = 25;
pub const COLOR_CYAN: u8 = 37;
pub const COLOR_BLUE: u8 = 45;
pub const COLOR_PURPLE: u8 = 53;
pub const COLOR_MAGENTA: u8 = 57;

// Task State Colors
pub const TASK_COLOR_PENDING: u8 = COLOR_GREY_DIM;
pub const TASK_COLOR_ACTIVE: u8 = COLOR_YELLOW_BRIGHT;
pub const TASK_COLOR_SUCCESS: u8 = COLOR_GREEN_BRIGHT;
pub const TASK_COLOR_ERROR: u8 = COLOR_RED_BRIGHT;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 9876;
