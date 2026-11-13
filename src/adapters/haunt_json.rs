//! Fear-engine and interactive sandbox payloads for the WASM adapter.
//!
//! Serializes routine home-fixing tasks and coupled M3->M5 dread steps
//! as deterministic JSON, and builds updated `FPM1` house meshes after
//! applying a player interaction command (`OpenDoor`, `CloseDoor`,
//! `ToggleLamp`, `ToggleLight`, `MoveFurniture`).

use crate::adapters::mesh_payload::mesh_bytes;
use crate::adapters::wasm::style_for_index;
use crate::domain::geometry::Rect;
use crate::domain::haunt::{DreadRung, DreadState, Event, EventKind, Task, TaskKind, TaskTarget};
use crate::domain::interaction::Interaction;
use crate::domain::light::Daylight;
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::extrude::extrude;
use crate::use_cases::generate::generate_house;
use crate::use_cases::haunt::step;
use crate::use_cases::haunt_events::wrong_details;
use crate::use_cases::haunt_tasks::generate_tasks;
use crate::use_cases::interact::apply;
use crate::use_cases::light_dread::{Lighting, PlayerLight, flicker_event, systems_for};

/// Renders a list of home-fixing tasks as a JSON array.
pub fn render_tasks_json(tasks: &[Task]) -> String {
    let mut s = String::from("[");
    for (i, t) in tasks.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let (target_kind, target_id) = target_fields(t.target);
        s.push_str(&format!(
            "{{\"id\":{},\"kind\":\"{}\",\"target_kind\":\"{}\",\"target\":{},\"done\":{}}}",
            t.id,
            task_kind_name(t.kind),
            target_kind,
            target_id,
            t.done
        ));
    }
    s.push(']');
    s
}

/// The home-fixing tasks for a house (seed and style index) as JSON.
pub fn house_tasks_json(seed: u32, style_index: u32) -> Result<Vec<u8>, String> {
    let Some(style) = style_for_index(style_index) else {
        return Err(format!("style index {style_index} out of range (0..6)"));
    };
    let house = generate_house(seed, style);
    let mut rng = Rng::new(seed);
    let tasks = generate_tasks(&house, &mut rng);
    Ok(render_tasks_json(&tasks).into_bytes())
}

/// Parameters for one coupled light + fear-engine step across WASM.
#[derive(Debug, Clone, Copy)]
pub struct HauntStepInput {
    pub seed: u32,
    pub style_index: u32,
    pub dial: u8,
    pub level: u8,
    pub room_id: u32,
    pub carrying_flashlight: bool,
    pub daylight_minutes: u32,
    pub tick: u64,
    pub dark_minutes: f64,
    pub tasks_done: u32,
}

/// Runs one coupled M3->M5 step and returns the resulting dread state,
/// light levels, and spent wrong-detail events as JSON.
pub fn haunt_step_json(input: HauntStepInput) -> Result<Vec<u8>, String> {
    let Some(style) = style_for_index(input.style_index) else {
        return Err(format!("style index {} out of range (0..6)", input.style_index));
    };
    let house = generate_house(input.seed, style);
    let lighting = Lighting {
        lights: &house.lights,
        daylight: Daylight::at(input.daylight_minutes),
        tick: input.tick,
    };
    let player = PlayerLight {
        room_id: input.room_id,
        carrying_flashlight: input.carrying_flashlight,
        dark_minutes: input.dark_minutes,
        tasks_done: input.tasks_done,
    };
    let systems = systems_for(&player, &lighting);
    let mut rng = Rng::new(input.seed ^ (input.tick as u32).wrapping_mul(0x9E37_79B9));
    let mut candidates = wrong_details(&house, &mut rng, input.dial);
    if let Some(ev) = flicker_event(input.room_id, &house.lights) {
        candidates.push(ev);
    }
    let mut state = DreadState {
        level: input.level.min(100),
        dial: input.dial.min(100),
        rung: DreadRung::from_level(input.level.min(100)),
        event_log: Vec::new(),
    };
    step(&mut state, &systems, &candidates);
    let events: Vec<String> = state.event_log.iter().map(format_event).collect();
    let json = format!(
        "{{\"level\":{},\"dial\":{},\"rung\":\"{}\",\"darkness\":{:.2},\"light_relief\":{:.2},\"events\":[{}]}}",
        state.level,
        state.dial,
        rung_name(state.rung),
        systems.darkness,
        systems.light_relief,
        events.join(",")
    );
    Ok(json.into_bytes())
}

/// Applies an interaction command (`0` open door, `1` close door,
/// `2` toggle lamp, `3` toggle light, `4` move furniture to centimetre
/// coordinates) and returns the updated `FPM1` house mesh.
pub fn house_interact_mesh_bytes(
    seed: u32,
    style_index: u32,
    cmd_kind: u32,
    target_id: u32,
    x_cm: i32,
    y_cm: i32,
) -> Result<Vec<u8>, String> {
    let Some(style) = style_for_index(style_index) else {
        return Err(format!("style index {style_index} out of range (0..6)"));
    };
    let command = match cmd_kind {
        0 => Interaction::OpenDoor { door_id: target_id },
        1 => Interaction::CloseDoor { door_id: target_id },
        2 => Interaction::ToggleLamp { furniture_id: target_id },
        3 => Interaction::ToggleLight { light_id: target_id },
        4 => Interaction::MoveFurniture {
            furniture_id: target_id,
            to: Rect::new(x_cm as f64 / 100.0, y_cm as f64 / 100.0, 0.0, 0.0),
        },
        _ => return Err(format!("unknown interaction command kind {cmd_kind}")),
    };
    let mut house = generate_house(seed, style);
    if !apply(&mut house, command) {
        return Err(format!("interaction {cmd_kind} on target {target_id} was refused"));
    }
    Ok(mesh_bytes(&extrude(&house)))
}

fn rung_name(rung: DreadRung) -> &'static str {
    match rung {
        DreadRung::Calm => "calm",
        DreadRung::Unease => "unease",
        DreadRung::Wrong => "wrong",
        DreadRung::Event => "event",
    }
}

fn task_kind_name(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::CheckLights => "check_lights",
        TaskKind::RestorePower => "restore_power",
        TaskKind::Catalog => "catalog",
        TaskKind::CloseDoors => "close_doors",
    }
}

fn target_fields(target: TaskTarget) -> (&'static str, u32) {
    match target {
        TaskTarget::Room(id) => ("room", id),
        TaskTarget::Furniture(id) => ("furniture", id),
    }
}

fn event_kind_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::DoorAjar => "door_ajar",
        EventKind::ChairMoved => "chair_moved",
        EventKind::LampOn => "lamp_on",
        EventKind::LightFlicker => "light_flicker",
    }
}

fn format_event(e: &Event) -> String {
    format!("{{\"kind\":\"{}\",\"target\":{}}}", event_kind_name(e.kind), e.target)
}
