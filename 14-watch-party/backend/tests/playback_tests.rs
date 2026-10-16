use watch_party::error::RoomError;
use watch_party::playback::PlaybackState;

fn state(playing: bool, position: f64, rate: f64, at: f64) -> PlaybackState {
    PlaybackState { playing, position, rate, at }
}

#[test]
fn playing_state_advances_with_server_time_so_late_joiners_land_on_the_same_frame() {
    let s = state(true, 10.0, 1.0, 1_000.0);
    assert!((s.expected_position(3_500.0) - 12.5).abs() < 1e-9);
}

#[test]
fn rate_scales_elapsed_time_so_double_speed_parties_stay_aligned() {
    let s = state(true, 10.0, 2.0, 0.0);
    assert!((s.expected_position(1_500.0) - 13.0).abs() < 1e-9);
}

#[test]
fn paused_state_never_moves_so_followers_do_not_creep_forward() {
    let s = state(false, 42.0, 1.0, 0.0);
    assert_eq!(s.expected_position(1_000_000.0), 42.0);
}

#[test]
fn server_time_before_state_time_does_not_rewind_the_party() {
    let s = state(true, 5.0, 1.0, 10_000.0);
    assert_eq!(s.expected_position(9_000.0), 5.0);
}

#[test]
fn rebasing_keeps_the_same_timeline_so_handoff_is_invisible_to_viewers() {
    let s = state(true, 3.0, 1.5, 0.0);
    let rebased = s.rebased(2_000.0);
    assert!((rebased.position - 6.0).abs() < 1e-9);
    assert_eq!(rebased.at, 2_000.0);
    for later in [2_000.0, 5_000.0, 60_000.0] {
        assert!((rebased.expected_position(later) - s.expected_position(later)).abs() < 1e-9);
    }
}

#[test]
fn future_timestamps_are_clamped_to_server_now_so_a_bad_leader_clock_cannot_freeze_followers() {
    let s = state(true, 1.0, 1.0, 99_999.0).validated(1_000.0).unwrap();
    assert_eq!(s.at, 1_000.0);
}

#[test]
fn nonsense_states_are_rejected_instead_of_broadcast() {
    assert!(matches!(state(true, -1.0, 1.0, 0.0).validated(0.0), Err(RoomError::InvalidState(_))));
    assert!(matches!(state(true, f64::NAN, 1.0, 0.0).validated(0.0), Err(RoomError::InvalidState(_))));
    assert!(matches!(state(true, 1.0, 0.0, 0.0).validated(0.0), Err(RoomError::InvalidState(_))));
    assert!(matches!(state(true, 1.0, 9.0, 0.0).validated(0.0), Err(RoomError::InvalidState(_))));
}
