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

pub struct LaunchpadEngine {
    lp: Arc<Mutex<LaunchpadMiniMK3>>,
    tasks: Arc<Mutex<HashMap<usize, TaskState>>>,
    animating: Arc<Mutex<bool>>,
    running: Arc<Mutex<bool>>,
}

impl LaunchpadEngine {
    pub fn new(lp: LaunchpadMiniMK3) -> Self {
        let mut tasks_map = HashMap::new();
        for i in 0..64 {
            tasks_map.insert(i, TaskState::Pending);
        }

        let engine = Self {
            lp: Arc::new(Mutex::new(lp)),
            tasks: Arc::new(Mutex::new(tasks_map)),
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

    pub fn set_task_state(&self, task_id: usize, state: TaskState) {
        if task_id < 64 {
            if let Ok(mut t) = self.tasks.lock() {
                t.insert(task_id, state);
            }
            self.render_grid(true);
        }
    }

    pub fn reset(&self) {
        if let Ok(mut t) = self.tasks.lock() {
            for i in 0..64 {
                t.insert(i, TaskState::Pending);
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

        let tasks_guard = self.tasks.lock().unwrap();
        let mut lp_guard = self.lp.lock().unwrap();

        for (&task_id, state) in tasks_guard.iter() {
            let row = (task_id / 8) as u8;
            let col = (task_id % 8) as u8;
            let color = Self::get_color_for_state(state, pulse_bright);
            lp_guard.set_grid_pad(row, col, color);
        }
    }

    fn start_pulse_thread(&self) {
        let tasks_clone = Arc::clone(&self.tasks);
        let animating_clone = Arc::clone(&self.animating);
        let running_clone = Arc::clone(&self.running);
        let lp_clone = Arc::clone(&self.lp);

        thread::spawn(move || {
            let mut toggle = false;
            while *running_clone.lock().unwrap() {
                let has_active = {
                    let t = tasks_clone.lock().unwrap();
                    t.values().any(|s| *s == TaskState::Active)
                };

                if has_active && !*animating_clone.lock().unwrap() {
                    toggle = !toggle;
                    let tasks_guard = tasks_clone.lock().unwrap();
                    let mut lp_guard = lp_clone.lock().unwrap();
                    for (&task_id, state) in tasks_guard.iter() {
                        let row = (task_id / 8) as u8;
                        let col = (task_id % 8) as u8;
                        let color = Self::get_color_for_state(state, toggle);
                        lp_guard.set_grid_pad(row, col, color);
                    }
                }
                thread::sleep(Duration::from_millis(500));
            }
        });
    }

    pub fn animate_operation(&self, anim_type: &str, transition: &str, duration_secs: f32) {
        let lp_clone = Arc::clone(&self.lp);
        let tasks_clone = Arc::clone(&self.tasks);
        let animating_clone = Arc::clone(&self.animating);

        let anim_str = anim_type.to_string();
        let trans_str = transition.to_string();

        thread::spawn(move || {
            {
                let mut anim_guard = animating_clone.lock().unwrap();
                if *anim_guard {
                    // Animation already running, skip or interrupt
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
            Self::play_transition_outro(&lp_clone, &tasks_clone, &trans_str);

            {
                let mut anim_guard = animating_clone.lock().unwrap();
                *anim_guard = false;
            }

            // Re-render grid state
            let tasks_guard = tasks_clone.lock().unwrap();
            let mut lp_guard = lp_clone.lock().unwrap();
            for (&task_id, state) in tasks_guard.iter() {
                let row = (task_id / 8) as u8;
                let col = (task_id % 8) as u8;
                let color = Self::get_color_for_state(state, true);
                lp_guard.set_grid_pad(row, col, color);
            }
        });
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

    fn play_transition_outro(lp: &Arc<Mutex<LaunchpadMiniMK3>>, tasks: &Arc<Mutex<HashMap<usize, TaskState>>>, trans: &str) {
        let tasks_guard = tasks.lock().unwrap();

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
            vec![(2, 2), (2, 3), (2, 4), (2, 5), (3, 2), (3, 5), (4, 2), (4, 5), (5, 2), (5, 3), (5, 4), (5, 5)],
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
