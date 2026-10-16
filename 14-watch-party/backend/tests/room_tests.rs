use watch_party::error::RoomError;
use watch_party::playback::PlaybackState;
use watch_party::room::{InMemoryRooms, RoomRegistry};

fn playing(position: f64, at: f64) -> PlaybackState {
    PlaybackState { playing: true, position, rate: 1.0, at }
}

#[test]
fn first_member_becomes_leader_because_someone_must_own_the_timeline() {
    let mut rooms = InMemoryRooms::new();
    let (alice, view) = rooms.join("movie", "alice", 0.0).unwrap();
    assert_eq!(view.leader, alice);
    let (_, view) = rooms.join("movie", "bob", 10.0).unwrap();
    assert_eq!(view.leader, alice);
    assert_eq!(view.members.len(), 2);
}

#[test]
fn only_the_leader_can_drive_playback_so_followers_cannot_fight_over_it() {
    let mut rooms = InMemoryRooms::new();
    let (alice, _) = rooms.join("movie", "alice", 0.0).unwrap();
    let (bob, _) = rooms.join("movie", "bob", 0.0).unwrap();
    assert_eq!(rooms.set_state("movie", bob, playing(5.0, 0.0), 0.0), Err(RoomError::NotLeader));
    let view = rooms.set_state("movie", alice, playing(5.0, 0.0), 0.0).unwrap();
    assert_eq!(view.state.position, 5.0);
}

#[test]
fn leader_leaving_hands_off_to_the_oldest_member_and_keeps_the_timeline() {
    let mut rooms = InMemoryRooms::new();
    let (alice, _) = rooms.join("movie", "alice", 0.0).unwrap();
    let (bob, _) = rooms.join("movie", "bob", 1.0).unwrap();
    let (carol, _) = rooms.join("movie", "carol", 2.0).unwrap();
    rooms.set_state("movie", alice, playing(10.0, 1_000.0), 1_000.0).unwrap();

    let view = rooms.leave("movie", alice, 4_000.0).unwrap().unwrap();

    assert_eq!(view.leader, bob);
    assert!(view.members.iter().all(|m| m.id != alice));
    assert!(view.state.playing);
    assert!((view.state.position - 13.0).abs() < 1e-9);
    assert_eq!(view.state.at, 4_000.0);
    assert_eq!(rooms.set_state("movie", carol, playing(0.0, 4_000.0), 4_000.0), Err(RoomError::NotLeader));
    assert!(rooms.set_state("movie", bob, playing(13.0, 4_000.0), 4_000.0).is_ok());
}

#[test]
fn follower_leaving_does_not_touch_leader_or_state() {
    let mut rooms = InMemoryRooms::new();
    let (alice, _) = rooms.join("movie", "alice", 0.0).unwrap();
    let (bob, _) = rooms.join("movie", "bob", 0.0).unwrap();
    let before = rooms.set_state("movie", alice, playing(7.0, 100.0), 100.0).unwrap().state;
    let view = rooms.leave("movie", bob, 9_000.0).unwrap().unwrap();
    assert_eq!(view.leader, alice);
    assert_eq!(view.state, before);
}

#[test]
fn last_member_leaving_closes_the_room_so_a_new_party_starts_fresh() {
    let mut rooms = InMemoryRooms::new();
    let (alice, _) = rooms.join("movie", "alice", 0.0).unwrap();
    rooms.set_state("movie", alice, playing(50.0, 0.0), 0.0).unwrap();
    assert_eq!(rooms.leave("movie", alice, 1.0).unwrap(), None);
    assert!(matches!(rooms.view("movie"), Err(RoomError::UnknownRoom(_))));
    let (dave, view) = rooms.join("movie", "dave", 2.0).unwrap();
    assert_eq!(view.leader, dave);
    assert_eq!(view.state.position, 0.0);
    assert!(!view.state.playing);
}

#[test]
fn rooms_are_isolated_from_each_other() {
    let mut rooms = InMemoryRooms::new();
    let (alice, _) = rooms.join("a", "alice", 0.0).unwrap();
    let (bob, _) = rooms.join("b", "bob", 0.0).unwrap();
    rooms.set_state("a", alice, playing(30.0, 0.0), 0.0).unwrap();
    assert_eq!(rooms.view("b").unwrap().state.position, 0.0);
    assert_eq!(rooms.view("b").unwrap().leader, bob);
    assert_eq!(rooms.set_state("b", alice, playing(1.0, 0.0), 0.0), Err(RoomError::UnknownMember(alice)));
}

#[test]
fn empty_names_are_rejected() {
    let mut rooms = InMemoryRooms::new();
    assert!(matches!(rooms.join("movie", "  ", 0.0), Err(RoomError::InvalidJoin(_))));
    assert!(matches!(rooms.join("", "alice", 0.0), Err(RoomError::InvalidJoin(_))));
}
