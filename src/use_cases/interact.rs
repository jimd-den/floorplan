//! Interact: apply an `Interaction` command to the house. The
//! core owns the state — a door's open/closed, a lamp's on/off, a
//! moved piece's position — and the writers (OBJ, glTF, JSON)
//! reflect it. A command that names nothing, or a move that would
//! break a placement rule, is refused and returns false.
//! See ADR 0003 and spec §3, §4.2, §5.

use crate::domain::furniture::{FurnitureKind, FurnitureState};
use crate::domain::geometry::Rect;
use crate::domain::interaction::Interaction;
use crate::domain::model::{House, OpeningKind};
use crate::use_cases::furnish_clear::fits;

/// Applies one command. Returns true when it changed the house.
pub fn apply(house: &mut House, command: Interaction) -> bool {
    match command {
        Interaction::OpenDoor { door_id } => set_door(house, door_id, true),
        Interaction::CloseDoor { door_id } => set_door(house, door_id, false),
        Interaction::ToggleLamp { furniture_id } => toggle_lamp(house, furniture_id),
        Interaction::ToggleLight { light_id } => {
            if !crate::use_cases::light::toggle(&mut house.lights, light_id) {
                return false;
            }
            crate::use_cases::light::sync_lamp(house, light_id);
            true
        }
        Interaction::MoveFurniture { furniture_id, to } => move_furniture(house, furniture_id, to),
    }
}

/// Opens (or closes) a door by id. A window is not a door.
fn set_door(house: &mut House, door_id: u32, open: bool) -> bool {
    for floor in &mut house.floors {
        if !floor.openings.iter().any(|o| {
            o.id == door_id && matches!(o.kind, OpeningKind::Door | OpeningKind::EntryDoor)
        }) {
            continue;
        }
        if open {
            if !floor.open_doors.contains(&door_id) {
                floor.open_doors.push(door_id);
            }
        } else {
            floor.open_doors.retain(|&id| id != door_id);
        }
        return true;
    }
    false
}

/// Flips a lamp on or off. Anything that is not a lamp is refused.
fn toggle_lamp(house: &mut House, furniture_id: u32) -> bool {
    for floor in &mut house.floors {
        for piece in &mut floor.furniture {
            if piece.id != furniture_id || piece.kind != FurnitureKind::Lamp {
                continue;
            }
            piece.state = match piece.state {
                FurnitureState::On => FurnitureState::Off,
                _ => FurnitureState::On,
            };
            return true;
        }
    }
    false
}

/// Moves a chair or table to a new rect. The destination must be
/// a valid placement: inside the room, overlapping nothing,
/// blocking no door swing and no (tall-piece) window. The piece's
/// stored position follows the move.
fn move_furniture(house: &mut House, furniture_id: u32, to: Rect) -> bool {
    for floor in &mut house.floors {
        let Some(idx) = floor.furniture.iter().position(|p| p.id == furniture_id) else {
            continue;
        };
        let piece = &floor.furniture[idx];
        if !piece.kind.is_movable() {
            return false;
        }
        let room_id = piece.room_id;
        let kind = piece.kind;
        let Some(room) = floor.rooms.iter().find(|r| r.id == room_id) else {
            return false;
        };
        let doors: Vec<_> = floor
            .openings
            .iter()
            .filter(|o| o.kind != OpeningKind::Window && (o.from == room_id || o.to == Some(room_id)))
            .collect();
        let windows: Vec<_> = floor
            .openings
            .iter()
            .filter(|o| o.kind == OpeningKind::Window && o.from == room_id)
            .collect();
        let others: Vec<_> = floor
            .furniture
            .iter()
            .filter(|p| p.room_id == room_id && p.id != furniture_id)
            .cloned()
            .collect();
        let moved = Rect::new(to.x, to.y, piece.rect.w, piece.rect.d);
        if !fits(room, kind, &moved, &doors, &windows, &others) {
            return false;
        }
        let piece = &mut floor.furniture[idx];
        piece.rect = moved;
        piece.state = FurnitureState::Moved { x: moved.x, y: moved.y };
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    fn house42() -> House {
        generate_house(42, Style::Modern)
    }

    fn first_door(house: &House) -> u32 {
        house.floors[0]
            .openings
            .iter()
            .find(|o| o.kind == OpeningKind::Door)
            .expect("no door")
            .id
    }

    fn first_lamp(house: &House) -> u32 {
        house.floors[0]
            .furniture
            .iter()
            .find(|p| p.kind == FurnitureKind::Lamp)
            .expect("no lamp")
            .id
    }

    #[test]
    fn opening_a_door_marks_it_open() {
        let mut house = house42();
        let id = first_door(&house);
        assert!(apply(&mut house, Interaction::OpenDoor { door_id: id }));
        assert!(house.floors[0].open_doors.contains(&id));
        assert!(apply(&mut house, Interaction::CloseDoor { door_id: id }));
        assert!(!house.floors[0].open_doors.contains(&id));
    }

    #[test]
    fn opening_a_missing_door_is_refused() {
        let mut house = house42();
        assert!(!apply(&mut house, Interaction::OpenDoor { door_id: 99999 }));
    }

    #[test]
    fn toggling_a_lamp_flips_it_on_and_off() {
        let mut house = house42();
        let id = first_lamp(&house);
        assert!(apply(&mut house, Interaction::ToggleLamp { furniture_id: id }));
        let lamp = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
        assert_eq!(lamp.state, FurnitureState::On);
        assert!(apply(&mut house, Interaction::ToggleLamp { furniture_id: id }));
        let lamp = house.floors[0].furniture.iter().find(|p| p.id == id).unwrap();
        assert_eq!(lamp.state, FurnitureState::Off);
    }

    #[test]
    fn toggling_a_bed_is_refused() {
        let mut house = house42();
        let bed = house.floors[0]
            .furniture
            .iter()
            .find(|p| p.kind == FurnitureKind::Bed)
            .expect("no bed")
            .id;
        assert!(!apply(&mut house, Interaction::ToggleLamp { furniture_id: bed }));
    }

    #[test]
    fn moving_a_bed_is_refused() {
        let mut house = house42();
        let bed = house.floors[0]
            .furniture
            .iter()
            .find(|p| p.kind == FurnitureKind::Bed)
            .expect("no bed")
            .id;
        let to = Rect::new(1.0, 1.0, 1.6, 2.1);
        assert!(!apply(&mut house, Interaction::MoveFurniture { furniture_id: bed, to }));
    }
}
