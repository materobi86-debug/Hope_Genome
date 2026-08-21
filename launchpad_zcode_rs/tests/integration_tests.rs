#[cfg(test)]
mod tests {
    use launchpad_zcode_rs::midi::LaunchpadMiniMK3;
    use launchpad_zcode_rs::engine::{LaunchpadEngine, TaskState};

    #[test]
    fn test_pad_note_conversion() {
        assert_eq!(LaunchpadMiniMK3::pad_to_note(0, 0), 81);
        assert_eq!(LaunchpadMiniMK3::pad_to_note(0, 7), 88);
        assert_eq!(LaunchpadMiniMK3::pad_to_note(7, 0), 11);
        assert_eq!(LaunchpadMiniMK3::pad_to_note(7, 7), 18);

        assert_eq!(LaunchpadMiniMK3::note_to_pad(81), Some((0, 0)));
        assert_eq!(LaunchpadMiniMK3::note_to_pad(18), Some((7, 7)));
    }

    #[test]
    fn test_task_state_grid() {
        let lp = LaunchpadMiniMK3::new(true);
        let engine = LaunchpadEngine::new(lp);

        engine.set_task_state(0, TaskState::Success);
        engine.set_task_state(1, TaskState::Error);

        engine.stop();
    }
}
