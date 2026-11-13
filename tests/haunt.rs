//! Fear engine: dread, the dial, tasks, and wrong details.

use floorplan::domain::haunt::{DreadRung, Event, EventKind, TaskTarget};
use floorplan::domain::style::Style;
use floorplan::frameworks_drivers::rng::Rng;
use floorplan::use_cases::generate::generate_house;
use floorplan::use_cases::haunt::{allowed_rung, complete_task, new_dread, step, Systems};
use floorplan::use_cases::haunt_events::wrong_details;
use floorplan::use_cases::haunt_tasks::generate_tasks;

fn dark_step() -> Systems {
    Systems { darkness: 1.0, dark_minutes: 60.0, tasks_done: 0, light_relief: 0.0 }
}

#[test]
fn dial_zero_never_raises_dread() {
    let mut state = new_dread(0);
    step(&mut state, &dark_step(), &[]);
    assert_eq!(state.level, 0, "dial 0 must freeze dread");
    assert_eq!(state.rung, DreadRung::Calm);
}

#[test]
fn light_relief_lowers_dread() {
    let mut state = new_dread(50);
    let systems = Systems { darkness: 1.0, dark_minutes: 0.0, tasks_done: 0, light_relief: 1.0 };
    step(&mut state, &systems, &[]);
    assert_eq!(state.level, 2, "light relief must ground the player");
}

#[test]
fn time_in_the_dark_raises_dread() {
    let mut state = new_dread(50);
    let systems = Systems { darkness: 1.0, dark_minutes: 10.0, tasks_done: 0, light_relief: 0.0 };
    step(&mut state, &systems, &[]);
    assert_eq!(state.level, 15, "ten dark minutes must add five");
}

#[test]
fn completed_tasks_lower_dread() {
    let mut state = new_dread(50);
    let systems = Systems { darkness: 1.0, dark_minutes: 0.0, tasks_done: 1, light_relief: 0.0 };
    step(&mut state, &systems, &[]);
    assert_eq!(state.level, 4, "a completed task must ground the player");
}

#[test]
fn the_dial_caps_the_rung_it_will_spend() {
    assert_eq!(allowed_rung(0), DreadRung::Calm);
    assert_eq!(allowed_rung(1), DreadRung::Unease);
    assert_eq!(allowed_rung(34), DreadRung::Wrong);
    assert_eq!(allowed_rung(67), DreadRung::Event);
    assert_eq!(allowed_rung(100), DreadRung::Event);
}

#[test]
fn wrong_details_raise_dread_and_are_logged() {
    let mut state = new_dread(100);
    let systems = Systems { darkness: 1.0, dark_minutes: 0.0, tasks_done: 0, light_relief: 0.0 };
    let candidates = vec![Event { kind: EventKind::DoorAjar, target: 7 }];
    step(&mut state, &systems, &candidates);
    assert_eq!(state.event_log, candidates, "an allowed detail must be logged");
    assert_eq!(state.level, 13, "a wrong detail must add three");
}

#[test]
fn wrong_details_are_refused_above_the_dials_rung() {
    let mut state = new_dread(1);
    let dark = Systems { darkness: 1.0, dark_minutes: 0.0, tasks_done: 0, light_relief: 0.0 };
    for _ in 0..8 {
        step(&mut state, &dark, &[]);
    }
    assert_eq!(state.rung, DreadRung::Event);
    let candidates = vec![Event { kind: EventKind::LampOn, target: 9 }];
    step(&mut state, &dark, &candidates);
    assert!(state.event_log.is_empty(), "dial 1 must not spend event-rung details");
}

#[test]
fn completing_a_task_grounds_the_player() {
    let house = generate_house(42, Style::Modern);
    let mut rng = Rng::new(42);
    let mut tasks = generate_tasks(&house, &mut rng);
    let id = tasks[0].id;
    let mut state = new_dread(50);
    step(&mut state, &dark_step(), &[]);
    let before = state.level;

    assert!(complete_task(&mut state, &mut tasks, id));
    assert!(tasks[0].done, "a completed task must be marked done");
    assert!(state.level < before, "a completed task must lower dread");
    assert!(!complete_task(&mut state, &mut tasks, id), "a task completes once");
}

#[test]
fn dial_zero_stays_silent_over_a_long_session() {
    for seed in 0..10u32 {
        let house = generate_house(seed, Style::Modern);
        let mut rng = Rng::new(seed);
        let candidates = wrong_details(&house, &mut rng, 100);
        let mut state = new_dread(0);
        for action in 0..100u32 {
            let systems = Systems {
                darkness: 1.0,
                dark_minutes: 5.0,
                tasks_done: action % 3,
                light_relief: 0.0,
            };
            step(&mut state, &systems, &candidates);
        }
        assert_eq!(state.level, 0, "seed {seed}: dial 0 must stay calm");
        assert!(state.event_log.is_empty(), "seed {seed}: dial 0 must log nothing");
        assert_eq!(state.rung, DreadRung::Calm);
    }
}

#[test]
fn wrong_details_are_seed_deterministic_and_silent_at_dial_zero() {
    let house = generate_house(42, Style::Modern);
    let mut a = Rng::new(7);
    let mut b = Rng::new(7);
    let first = wrong_details(&house, &mut a, 80);
    let second = wrong_details(&house, &mut b, 80);
    assert_eq!(first, second, "same seed must reproduce the same details");
    assert!(!first.is_empty(), "a high dial must offer wrong details");

    let mut c = Rng::new(7);
    assert!(wrong_details(&house, &mut c, 0).is_empty(), "dial 0 is silent");
}

#[test]
fn tasks_name_real_rooms_and_objects() {
    let house = generate_house(42, Style::Modern);
    let mut rng = Rng::new(42);
    let tasks = generate_tasks(&house, &mut rng);
    assert!(!tasks.is_empty(), "a house must offer tasks");
    for task in &tasks {
        match task.target {
            TaskTarget::Room(id) => assert!(
                house.floors.iter().any(|f| f.rooms.iter().any(|r| r.id == id)),
                "task names missing room {id}"
            ),
            TaskTarget::Furniture(id) => assert!(
                house.floors.iter().any(|f| f.furniture.iter().any(|p| p.id == id)),
                "task names missing piece {id}"
            ),
        }
    }
}
