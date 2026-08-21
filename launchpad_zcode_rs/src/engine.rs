use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;

use crate::midi::LaunchpadMiniMK3;
use crate::config::*;

#[derive(Clone, Debug, PartialEq)]
pub enum TaskState {
    Pending,
    Active,
    Success,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppTarget {
    HopeCode,
    ClaudeCode,
    Codex,
}

impl AppTarget {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" | "claude_code" | "claude-code" => AppTarget::ClaudeCode,
            "codex" | "openai_codex" | "openai-codex" => AppTarget::Codex,
            _ => AppTarget::HopeCode,
        }
    }

    pub fn to_side_note(&self) -> u8 {
        match self {
            AppTarget::HopeCode => 89,
            AppTarget::ClaudeCode => 79,
            AppTarget::Codex => 69,
        }
    }
}

fn get_char_bitmap(c: char) -> [u8; 8] {
    match c.to_ascii_uppercase() {
        'H' => [0b01100110, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
        'S' => [0b00111100, 0b01100000, 0b01100000, 0b00111100, 0b00000110, 0b00000110, 0b00111100, 0b00000000],
        'T' => [0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000],
        'O' => [0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00000000],
        'P' => [0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b00000000],
        '!' => [0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00000000],
        'Z' => [0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110, 0b00000000],
        'C' => [0b00111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b01100110, 0b00111100, 0b00000000],
        'D' => [0b01111000, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01111000, 0b00000000],
        'E' => [0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01111110, 0b00000000],
        'A' => [0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
        'K' => [0b01100110, 0b01101100, 0b01111000, 0b01110000, 0b01111000, 0b01101100, 0b01100110, 0b00000000],
        'N' => [0b01100110, 0b01110110, 0b01111110, 0b01101110, 0b01100110, 0b01100110, 0b01100110, 0b00000000],
        _   => [0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000],
    }
}

pub struct LaunchpadEngine {
    lp: Arc<Mutex<LaunchpadMiniMK3>>,
    app_tasks: Arc<Mutex<HashMap<AppTarget, HashMap<usize, TaskState>>>>,
    active_app: Arc<Mutex<AppTarget>>,
    app_running: Arc<Mutex<HashMap<AppTarget, bool>>>,
    app_flashing: Arc<Mutex<HashMap<AppTarget, bool>>>,
    animating: Arc<Mutex<bool>>,
    running: Arc<Mutex<bool>>,
}

impl LaunchpadEngine {
    pub fn new(lp: LaunchpadMiniMK3) -> Self {
        let mut app_tasks_map = HashMap::new();
        for app in [AppTarget::HopeCode, AppTarget::ClaudeCode, AppTarget::Codex] {
            let mut tasks = HashMap::new();
            for i in 0..64 {
                tasks.insert(i, TaskState::Pending);
            }
            app_tasks_map.insert(app, tasks);
        }

        let mut running_map = HashMap::new();
        let mut flashing_map = HashMap::new();
        for app in [AppTarget::HopeCode, AppTarget::ClaudeCode, AppTarget::Codex] {
            running_map.insert(app.clone(), true);
            flashing_map.insert(app, false);
        }

        let engine = Self {
            lp: Arc::new(Mutex::new(lp)),
            app_tasks: Arc::new(Mutex::new(app_tasks_map)),
            active_app: Arc::new(Mutex::new(AppTarget::HopeCode)),
            app_running: Arc::new(Mutex::new(running_map)),
            app_flashing: Arc::new(Mutex::new(flashing_map)),
            animating: Arc::new(Mutex::new(false)),
            running: Arc::new(Mutex::new(true)),
        };

        engine.start_pulse_thread();
        engine
    }

    pub fn stop(&self) {
        if let Ok(mut r) = self.running.lock() {
            *r = false;
        }
    }

    pub fn set_active_app(&self, app: AppTarget) {
        if let Ok(mut curr) = self.active_app.lock() {
            *curr = app;
        }
        if let Ok(mut flash) = self.app_flashing.lock() {
            let curr_app = self.active_app.lock().unwrap().clone();
            flash.insert(curr_app, false);
        }
        self.render_grid(true);
    }

    pub fn set_task_state_for_app(&self, app: AppTarget, task_id: usize, state: TaskState) {
        if task_id < 64 {
            if let Ok(mut all_tasks) = self.app_tasks.lock() {
                if let Some(tasks) = all_tasks.get_mut(&app) {
                    tasks.insert(task_id, state);
                }
            }

            let curr_app = self.active_app.lock().unwrap().clone();
            if app != curr_app {
                if let Ok(mut flash) = self.app_flashing.lock() {
                    flash.insert(app, true);
                }
            }

            self.render_grid(true);
        }
    }

    pub fn set_task_state(&self, task_id: usize, state: TaskState) {
        let curr_app = self.active_app.lock().unwrap().clone();
        self.set_task_state_for_app(curr_app, task_id, state);
    }

    pub fn reset(&self) {
        if let Ok(mut all_tasks) = self.app_tasks.lock() {
            for tasks in all_tasks.values_mut() {
                for i in 0..64 {
                    tasks.insert(i, TaskState::Pending);
                }
            }
        }
        self.render_grid(true);
    }

    pub fn get_color_for_state(state: &TaskState, pulse_bright: bool) -> u8 {
        match state {
            TaskState::Pending => TASK_COLOR_PENDING,
            TaskState::Active => if pulse_bright { TASK_COLOR_ACTIVE } else { COLOR_YELLOW_DIM },
            TaskState::Success => TASK_COLOR_SUCCESS,
            TaskState::Error => TASK_COLOR_ERROR,
        }
    }

    pub fn render_grid(&self, pulse_bright: bool) {
        if *self.animating.lock().unwrap() {
            return;
        }

        let curr_app = self.active_app.lock().unwrap().clone();
        let all_tasks_guard = self.app_tasks.lock().unwrap();
        let tasks_guard = &all_tasks_guard[&curr_app];
        let mut lp_guard = self.lp.lock().unwrap();

        for (&task_id, state) in tasks_guard.iter() {
            let row = (task_id / 8) as u8;
            let col = (task_id % 8) as u8;
            let color = Self::get_color_for_state(state, pulse_bright);
            lp_guard.set_grid_pad(row, col, color);
        }

        let flash_guard = self.app_flashing.lock().unwrap();
        let run_guard = self.app_running.lock().unwrap();

        for app in [AppTarget::HopeCode, AppTarget::ClaudeCode, AppTarget::Codex] {
            let note = app.to_side_note();
            let is_running = run_guard.get(&app).copied().unwrap_or(false);
            let is_flashing = flash_guard.get(&app).copied().unwrap_or(false);

            let side_color = if app == curr_app {
                COLOR_GREEN_BRIGHT
            } else if is_flashing {
                if pulse_bright { COLOR_YELLOW_BRIGHT } else { COLOR_OFF }
            } else if is_running {
                COLOR_CYAN
            } else {
                COLOR_GREY_DIM
            };

            lp_guard.set_led(note, side_color, false);
        }
    }

    fn start_pulse_thread(&self) {
        let app_tasks_clone = Arc::clone(&self.app_tasks);
        let active_app_clone = Arc::clone(&self.active_app);
        let app_flashing_clone = Arc::clone(&self.app_flashing);
        let app_running_clone = Arc::clone(&self.app_running);
        let animating_clone = Arc::clone(&self.animating);
        let running_clone = Arc::clone(&self.running);
        let lp_clone = Arc::clone(&self.lp);

        thread::spawn(move || {
            let mut toggle = false;
            while *running_clone.lock().unwrap() {
                let curr_app = active_app_clone.lock().unwrap().clone();
                let has_active = {
                    let all_tasks = app_tasks_clone.lock().unwrap();
                    let t = &all_tasks[&curr_app];
                    t.values().any(|s| *s == TaskState::Active)
                };

                let has_flashing = {
                    let flash = app_flashing_clone.lock().unwrap();
                    flash.values().any(|&f| f)
                };

                if (has_active || has_flashing) && !*animating_clone.lock().unwrap() {
                    toggle = !toggle;
                    let all_tasks_guard = app_tasks_clone.lock().unwrap();
                    let tasks_guard = &all_tasks_guard[&curr_app];
                    let mut lp_guard = lp_clone.lock().unwrap();

                    for (&task_id, state) in tasks_guard.iter() {
                        let row = (task_id / 8) as u8;
                        let col = (task_id % 8) as u8;
                        let color = Self::get_color_for_state(state, toggle);
                        lp_guard.set_grid_pad(row, col, color);
                    }

                    let flash_guard = app_flashing_clone.lock().unwrap();
                    let run_guard = app_running_clone.lock().unwrap();
                    for app in [AppTarget::HopeCode, AppTarget::ClaudeCode, AppTarget::Codex] {
                        let note = app.to_side_note();
                        let is_running = run_guard.get(&app).copied().unwrap_or(false);
                        let is_flashing = flash_guard.get(&app).copied().unwrap_or(false);

                        let side_color = if app == curr_app {
                            COLOR_GREEN_BRIGHT
                        } else if is_flashing {
                            if toggle { COLOR_YELLOW_BRIGHT } else { COLOR_OFF }
                        } else if is_running {
                            COLOR_CYAN
                        } else {
                            COLOR_GREY_DIM
                        };

                        lp_guard.set_led(note, side_color, false);
                    }
                }
                thread::sleep(Duration::from_millis(500));
            }
        });
    }

    pub fn animate_operation(&self, anim_type: &str, transition: &str, duration_secs: f32) {
        self.animate_operation_with_text(anim_type, transition, duration_secs, "HOPE CODE");
    }

    pub fn animate_operation_with_text(&self, anim_type: &str, transition: &str, duration_secs: f32, custom_text: &str) {
        let lp_clone = Arc::clone(&self.lp);
        let app_tasks_clone = Arc::clone(&self.app_tasks);
        let active_app_clone = Arc::clone(&self.active_app);
        let animating_clone = Arc::clone(&self.animating);

        let anim_str = anim_type.to_string();
        let trans_str = transition.to_string();
        let text_str = custom_text.to_string();

        thread::spawn(move || {
            {
                let mut anim_guard = animating_clone.lock().unwrap();
                if *anim_guard {
                    return;
                }
                *anim_guard = true;
            }

            // Intro transition
            Self::play_transition_intro(&lp_clone, &trans_str);

            // Main animation
            let start = Instant::now();
            let dur = Duration::from_secs_f32(duration_secs);

            match anim_str.as_str() {
                "cpu_ram_meter" => Self::anim_cpu_ram_meter(&lp_clone, start, dur),
                "pomodoro_timer" => Self::anim_pomodoro_timer(&lp_clone, start, dur),
                "git_sentinel" => Self::anim_git_sentinel(&lp_clone, start, dur),
                "text_banner" => Self::anim_scrolling_text(&lp_clone, &text_str, COLOR_RED_BRIGHT, dur),
                "vortex_whirl" => Self::anim_vortex_whirl(&lp_clone, start, dur),
                "color_comb" => Self::anim_color_comb(&lp_clone, start, dur),
                "hypnotic_rings" => Self::anim_hypnotic_rings(&lp_clone, start, dur),
                "pulsar_burst" => Self::anim_pulsar_burst(&lp_clone, start, dur),
                "spinner" => Self::anim_spinner(&lp_clone, start, dur),
                "scan" => Self::anim_scan(&lp_clone, start, dur),
                "success_ripple" => Self::anim_ripple(&lp_clone, COLOR_GREEN_BRIGHT, dur),
                "error_flash" => Self::anim_flash(&lp_clone, COLOR_RED_BRIGHT, dur),
                "rainbow_wave" => Self::anim_rainbow_wave(&lp_clone, start, dur),
                "matrix_rain" => Self::anim_matrix_rain(&lp_clone, start, dur),
                "fireworks" => Self::anim_fireworks(&lp_clone, start, dur),
                "galaxy_spiral" => Self::anim_galaxy_spiral(&lp_clone, start, dur),
                "plasma_wave" => Self::anim_plasma_wave(&lp_clone, start, dur),
                "equalizer_bars" => Self::anim_equalizer_bars(&lp_clone, start, dur),
                "strobe_pulse" => Self::anim_strobe_pulse(&lp_clone, start, dur),
                _ => Self::anim_spinner(&lp_clone, start, dur),
            }

            // Outro transition back to grid
            let curr_app = active_app_clone.lock().unwrap().clone();
            Self::play_transition_outro(&lp_clone, &app_tasks_clone, &curr_app, &trans_str);

            {
                let mut anim_guard = animating_clone.lock().unwrap();
                *anim_guard = false;
            }

            // Re-render grid state
            let all_tasks_guard = app_tasks_clone.lock().unwrap();
            let tasks_guard = &all_tasks_guard[&curr_app];
            let mut lp_guard = lp_clone.lock().unwrap();
            for (&task_id, state) in tasks_guard.iter() {
                let row = (task_id / 8) as u8;
                let col = (task_id % 8) as u8;
                let color = Self::get_color_for_state(state, true);
                lp_guard.set_grid_pad(row, col, color);
            }
        });
    }

    // --- Jules Idea #1: Live CPU & RAM Performance Meter ---
    fn anim_cpu_ram_meter(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut rng = rand::thread_rng();
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();

                // Top 2 rows: CPU Load (Green -> Yellow -> Red)
                let cpu_val = rng.gen_range(2..=8);
                for col in 0..cpu_val {
                    let color = if col >= 6 { COLOR_RED_BRIGHT } else if col >= 4 { COLOR_YELLOW_BRIGHT } else { COLOR_GREEN_BRIGHT };
                    lp_guard.set_grid_pad(0, col as u8, color);
                    lp_guard.set_grid_pad(1, col as u8, color);
                }

                // Bottom 2 rows: RAM Usage (Cyan -> Blue)
                let ram_val = rng.gen_range(3..=8);
                for col in 0..ram_val {
                    let color = if col >= 6 { COLOR_PURPLE } else { COLOR_CYAN };
                    lp_guard.set_grid_pad(6, col as u8, color);
                    lp_guard.set_grid_pad(7, col as u8, color);
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    // --- Jules Idea #2: Pomodoro Focus Clock Ring ---
    fn anim_pomodoro_timer(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let outer_ring = [
            (0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7),
            (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7),
            (7, 6), (7, 5), (7, 4), (7, 3), (7, 2), (7, 1), (7, 0),
            (6, 0), (5, 0), (4, 0), (3, 0), (2, 0), (1, 0)
        ];
        let mut progress = 0;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for i in 0..outer_ring.len() {
                    let (r, c) = outer_ring[i];
                    let color = if i <= progress % outer_ring.len() { COLOR_ORANGE } else { COLOR_GREY_DIM };
                    lp_guard.set_grid_pad(r, c, color);
                }
            }
            progress += 1;
            thread::sleep(Duration::from_millis(60));
        }
    }

    // --- Jules Idea #3: Git Commit & AI Proof Sentinel ---
    fn anim_git_sentinel(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let rings = [
            vec![(3, 3), (3, 4), (4, 3), (4, 4)],
            vec![(2, 2), (2, 5), (5, 2), (5, 5)],
            vec![(1, 1), (1, 6), (6, 1), (6, 6)],
            vec![(0, 0), (0, 7), (7, 0), (7, 7)],
        ];
        while start.elapsed() < dur {
            for ring in &rings {
                {
                    let mut lp_guard = lp.lock().unwrap();
                    lp_guard.clear();
                    for &(r, c) in ring {
                        lp_guard.set_grid_pad(r, c, COLOR_GREEN_BRIGHT);
                    }
                }
                thread::sleep(Duration::from_millis(80));
            }
        }
    }

    // --- Scrolling Text Banner Animation ---

    fn anim_scrolling_text(lp: &Arc<Mutex<LaunchpadMiniMK3>>, text: &str, color: u8, dur: Duration) {
        let text_chars: Vec<char> = text.chars().collect();
        if text_chars.is_empty() { return; }

        let mut text_buffer: Vec<[u8; 8]> = Vec::new();
        for &ch in &text_chars {
            text_buffer.push(get_char_bitmap(ch));
        }

        let total_cols = text_buffer.len() * 8 + 8;
        let start = Instant::now();
        let mut offset = 0;

        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();

                for r in 0..8 {
                    for c in 0..8 {
                        let global_col = (offset + c) % total_cols;
                        let char_idx = global_col / 8;
                        let col_in_char = global_col % 8;

                        if char_idx < text_buffer.len() {
                            let char_map = text_buffer[char_idx];
                            let row_bits = char_map[r];
                            let pixel_on = (row_bits & (1 << (7 - col_in_char))) != 0;
                            if pixel_on {
                                lp_guard.set_grid_pad(r as u8, c as u8, color);
                            }
                        }
                    }
                }
            }
            offset += 1;
            thread::sleep(Duration::from_millis(70));
        }
    }

    // --- Transition Routines ---

    fn play_transition_intro(lp: &Arc<Mutex<LaunchpadMiniMK3>>, trans: &str) {
        match trans {
            "wipe_right" => {
                for col in 0..8 {
                    {
                        let mut lp_guard = lp.lock().unwrap();
                        for row in 0..8 {
                            lp_guard.set_grid_pad(row, col, COLOR_WHITE);
                        }
                    }
                    thread::sleep(Duration::from_millis(15));
                }
            }
            "zoom_iris" => {
                let rings = [
                    [(0, 0), (0, 7), (7, 0), (7, 7)],
                    [(1, 1), (1, 6), (6, 1), (6, 6)],
                    [(2, 2), (2, 5), (5, 2), (5, 5)],
                    [(3, 3), (3, 4), (4, 3), (4, 4)],
                ];
                for ring in rings {
                    {
                        let mut lp_guard = lp.lock().unwrap();
                        for (r, c) in ring {
                            lp_guard.set_grid_pad(r, c, COLOR_MAGENTA);
                        }
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
            "dissolve" | _ => {
                let mut rng = rand::thread_rng();
                for _ in 0..20 {
                    let r = rng.gen_range(0..8);
                    let c = rng.gen_range(0..8);
                    {
                        let mut lp_guard = lp.lock().unwrap();
                        lp_guard.set_grid_pad(r, c, COLOR_CYAN);
                    }
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }

    fn play_transition_outro(lp: &Arc<Mutex<LaunchpadMiniMK3>>, app_tasks: &Arc<Mutex<HashMap<AppTarget, HashMap<usize, TaskState>>>>, curr_app: &AppTarget, trans: &str) {
        let all_tasks_guard = app_tasks.lock().unwrap();
        let tasks_guard = &all_tasks_guard[curr_app];

        match trans {
            "wipe_right" => {
                for col in 0..8 {
                    {
                        let mut lp_guard = lp.lock().unwrap();
                        for row in 0..8 {
                            let tid = (row * 8 + col) as usize;
                            let color = Self::get_color_for_state(&tasks_guard[&tid], true);
                            lp_guard.set_grid_pad(row, col, color);
                        }
                    }
                    thread::sleep(Duration::from_millis(15));
                }
            }
            _ => {
                let mut lp_guard = lp.lock().unwrap();
                for (&tid, state) in tasks_guard.iter() {
                    let row = (tid / 8) as u8;
                    let col = (tid % 8) as u8;
                    let color = Self::get_color_for_state(state, true);
                    lp_guard.set_grid_pad(row, col, color);
                }
            }
        }
    }

    // --- Visual Animations ---

    fn anim_vortex_whirl(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut angle: f32 = 0.0;
        let colors = [COLOR_CYAN, COLOR_MAGENTA, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT];
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let dx = c as f32 - 3.5;
                        let dy = r as f32 - 3.5;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let a = dy.atan2(dx) + angle;
                        if (a.sin() * dist).abs() < 1.2 {
                            let color = colors[(dist as usize) % colors.len()];
                            lp_guard.set_grid_pad(r as u8, c as u8, color);
                        }
                    }
                }
            }
            angle += 0.3;
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn anim_color_comb(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut step = 0;
        let colors = [COLOR_RED_BRIGHT, COLOR_ORANGE, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT, COLOR_CYAN, COLOR_BLUE, COLOR_PURPLE, COLOR_MAGENTA];
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let idx = (r * 2 + c + step) % colors.len();
                        lp_guard.set_grid_pad(r as u8, c as u8, colors[idx]);
                    }
                }
            }
            step += 1;
            thread::sleep(Duration::from_millis(60));
        }
    }

    fn anim_hypnotic_rings(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut step = 0;
        let colors = [COLOR_WHITE, COLOR_CYAN, COLOR_BLUE, COLOR_MAGENTA];
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let dx = c as f32 - 3.5;
                        let dy = r as f32 - 3.5;
                        let ring_idx = ((dx * dx + dy * dy).sqrt() as usize + step) % colors.len();
                        lp_guard.set_grid_pad(r as u8, c as u8, colors[ring_idx]);
                    }
                }
            }
            step += 1;
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn anim_pulsar_burst(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut pulse = 0;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                let color = if pulse % 2 == 0 { COLOR_RED_BRIGHT } else { COLOR_YELLOW_BRIGHT };
                for r in 0..8 {
                    for c in 0..8 {
                        if (r == 3 || r == 4) || (c == 3 || c == 4) {
                            lp_guard.set_grid_pad(r as u8, c as u8, color);
                        }
                    }
                }
            }
            pulse += 1;
            thread::sleep(Duration::from_millis(70));
        }
    }

    fn anim_spinner(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let coords = [
            (2, 2), (2, 3), (2, 4), (2, 5),
            (3, 5), (4, 5), (5, 5),
            (5, 4), (5, 3), (5, 2),
            (4, 2), (3, 2),
        ];
        let mut idx = 0;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                let (r, c) = coords[idx % coords.len()];
                lp_guard.set_grid_pad(r, c, COLOR_CYAN);
            }
            idx += 1;
            thread::sleep(Duration::from_millis(60));
        }
    }

    fn anim_scan(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut row: i8 = 0;
        let mut dir: i8 = 1;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for col in 0..8 {
                    lp_guard.set_grid_pad(row as u8, col, COLOR_WHITE);
                }
            }
            row += dir;
            if row > 7 || row < 0 {
                dir *= -1;
                row += dir * 2;
            }
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn anim_ripple(lp: &Arc<Mutex<LaunchpadMiniMK3>>, color: u8, dur: Duration) {
        let rings = [
            vec![(3, 3), (3, 4), (4, 3), (4, 4)],
            vec![(2, 2), (2, 5), (5, 2), (5, 5)],
            vec![(1, 1), (1, 6), (6, 1), (6, 6)],
            vec![(0, 0), (0, 7), (7, 0), (7, 7)],
        ];
        let step_delay = dur / rings.len() as u32;
        for ring in rings {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for (r, c) in ring {
                    lp_guard.set_grid_pad(r, c, color);
                }
            }
            thread::sleep(step_delay);
        }
    }

    fn anim_flash(lp: &Arc<Mutex<LaunchpadMiniMK3>>, color: u8, dur: Duration) {
        let delay = dur / 6;
        for _ in 0..3 {
            {
                let mut lp_guard = lp.lock().unwrap();
                for r in 0..8 {
                    for c in 0..8 {
                        lp_guard.set_grid_pad(r, c, color);
                    }
                }
            }
            thread::sleep(delay);
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
            }
            thread::sleep(delay);
        }
    }

    fn anim_rainbow_wave(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let colors = [COLOR_RED_BRIGHT, COLOR_ORANGE, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT, COLOR_CYAN, COLOR_BLUE, COLOR_MAGENTA, COLOR_PURPLE];
        let mut shift = 0;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let idx = (r + c + shift) % colors.len();
                        lp_guard.set_grid_pad(r as u8, c as u8, colors[idx]);
                    }
                }
            }
            shift += 1;
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn anim_matrix_rain(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut rng = rand::thread_rng();
        let mut drops: Vec<i8> = (0..8).map(|_| rng.gen_range(-8..0)).collect();
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for col in 0..8 {
                    let head = drops[col];
                    if (0..8).contains(&head) {
                        lp_guard.set_grid_pad(head as u8, col as u8, COLOR_WHITE);
                    }
                    if (0..8).contains(&(head - 1)) {
                        lp_guard.set_grid_pad((head - 1) as u8, col as u8, COLOR_GREEN_BRIGHT);
                    }
                    drops[col] += 1;
                    if drops[col] > 10 {
                        drops[col] = rng.gen_range(-4..0);
                    }
                }
            }
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn anim_fireworks(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut rng = rand::thread_rng();
        let colors = [COLOR_CYAN, COLOR_MAGENTA, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT, COLOR_ORANGE];
        while start.elapsed() < dur {
            let center_r = rng.gen_range(1..7);
            let center_c = rng.gen_range(1..7);
            let color = colors[rng.gen_range(0..colors.len())];

            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                lp_guard.set_grid_pad(center_r, center_c, COLOR_WHITE);
            }
            thread::sleep(Duration::from_millis(50));

            {
                let mut lp_guard = lp.lock().unwrap();
                for dr in [-1, 0, 1] {
                    for dc in [-1, 0, 1] {
                        let nr = center_r as i8 + dr;
                        let nc = center_c as i8 + dc;
                        if (0..8).contains(&nr) && (0..8).contains(&nc) {
                            lp_guard.set_grid_pad(nr as u8, nc as u8, color);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(120));
        }
    }

    fn anim_galaxy_spiral(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut angle_offset = 0.0f32;
        let colors = [COLOR_CYAN, COLOR_BLUE, COLOR_MAGENTA, COLOR_PURPLE];
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let dx = c as f32 - 3.5;
                        let dy = r as f32 - 3.5;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let angle = dy.atan2(dx);
                        let spiral_val = (dist - angle * 2.0 + angle_offset).sin();
                        if spiral_val > 0.4 {
                            let color = colors[(dist as usize) % colors.len()];
                            lp_guard.set_grid_pad(r as u8, c as u8, color);
                        }
                    }
                }
            }
            angle_offset += 0.4;
            thread::sleep(Duration::from_millis(60));
        }
    }

    fn anim_plasma_wave(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut t = 0.0f32;
        let colors = [COLOR_RED_BRIGHT, COLOR_ORANGE, COLOR_YELLOW_BRIGHT, COLOR_GREEN_BRIGHT, COLOR_CYAN, COLOR_MAGENTA];
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        let v1 = (c as f32 * 0.8 + t).sin();
                        let v2 = (r as f32 * 0.8 + t * 1.2).sin();
                        let v = (v1 + v2 + 2.0) / 4.0;
                        let color_idx = ((v * (colors.len() - 1) as f32) as usize).min(colors.len() - 1);
                        lp_guard.set_grid_pad(r as u8, c as u8, colors[color_idx]);
                    }
                }
            }
            t += 0.3;
            thread::sleep(Duration::from_millis(60));
        }
    }

    fn anim_equalizer_bars(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut rng = rand::thread_rng();
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for col in 0..8 {
                    let height = rng.gen_range(1..=8);
                    for r in 0..height {
                        let row = 7 - r;
                        let color = if r >= 6 { COLOR_RED_BRIGHT } else if r >= 4 { COLOR_YELLOW_BRIGHT } else { COLOR_GREEN_BRIGHT };
                        lp_guard.set_grid_pad(row as u8, col as u8, color);
                    }
                }
            }
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn anim_strobe_pulse(lp: &Arc<Mutex<LaunchpadMiniMK3>>, start: Instant, dur: Duration) {
        let mut step = 0;
        while start.elapsed() < dur {
            {
                let mut lp_guard = lp.lock().unwrap();
                lp_guard.clear();
                for r in 0..8 {
                    for c in 0..8 {
                        if (r + c) % 2 == (step % 2) {
                            lp_guard.set_grid_pad(r as u8, c as u8, COLOR_WHITE);
                        } else {
                            lp_guard.set_grid_pad(r as u8, c as u8, COLOR_BLUE);
                        }
                    }
                }
            }
            step += 1;
            thread::sleep(Duration::from_millis(80));
        }
    }
}
