//! Interaction commands: the one command model (spec §3, §5).
//! A command names a door, a lamp, or a piece of furniture by id;
//! `use_cases::interact` applies it to the house and the writers
//! reflect the new state. The renderer only shows it.

use crate::domain::geometry::Rect;

/// A command against the house state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interaction {
    /// Open a door (hinged; the renderer animates the swing).
    OpenDoor { door_id: u32 },
    /// Close a door.
    CloseDoor { door_id: u32 },
    /// Flip a lamp on or off.
    ToggleLamp { furniture_id: u32 },
    /// Flip any light: a ceiling, lamp, porch, or the flashlight.
    ToggleLight { light_id: u32 },
    /// Move a chair or table to a new rect (north-west corner).
    MoveFurniture { furniture_id: u32, to: Rect },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_carry_their_ids() {
        let c = Interaction::ToggleLamp { furniture_id: 42 };
        assert_eq!(c, Interaction::ToggleLamp { furniture_id: 42 });
        assert_ne!(c, Interaction::ToggleLamp { furniture_id: 43 });
    }
}
