//! Fear-engine data: dread, its rungs, tasks, and the wrong-detail log.
//!
//! Pure data plus one classification (`DreadRung::from_level`). The
//! coupling between darkness, tasks, and dread lives in
//! `use_cases::haunt`; the shell supplies the systems as data.

/// How afraid the house currently is. Ordered by severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DreadRung {
    Calm,
    Unease,
    Wrong,
    Event,
}

impl DreadRung {
    /// The rung a dread level falls in. One place, so the thresholds
    /// are not re-derived by callers.
    pub fn from_level(level: u8) -> Self {
        match level {
            0..=24 => DreadRung::Calm,
            25..=49 => DreadRung::Unease,
            50..=74 => DreadRung::Wrong,
            _ => DreadRung::Event,
        }
    }
}

/// A routine, grounding job generated from a real house.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    CheckLights,
    RestorePower,
    Catalog,
    CloseDoors,
}

/// What a task acts on: a real room or a real piece of furniture.
/// A task never names a thing the house does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskTarget {
    Room(u32),
    Furniture(u32),
}

/// One task. `done` starts false.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: u32,
    pub kind: TaskKind,
    pub target: TaskTarget,
    pub done: bool,
}

/// A small, deniable wrong detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    DoorAjar,
    ChairMoved,
    LampOn,
    LightFlicker,
}

/// One logged wrong detail. `target` is a door id, furniture id, or
/// room id, depending on the kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub target: u32,
}

/// The dread of one house: a level, the player's dial, the rung the
/// level falls in, and the log of wrong details.
#[derive(Debug, Clone, PartialEq)]
pub struct DreadState {
    pub level: u8,
    pub dial: u8,
    pub rung: DreadRung,
    pub event_log: Vec<Event>,
}

#[cfg(test)]
mod tests {
    use super::DreadRung;

    #[test]
    fn rungs_rise_at_their_thresholds() {
        assert_eq!(DreadRung::from_level(0), DreadRung::Calm);
        assert_eq!(DreadRung::from_level(24), DreadRung::Calm);
        assert_eq!(DreadRung::from_level(25), DreadRung::Unease);
        assert_eq!(DreadRung::from_level(49), DreadRung::Unease);
        assert_eq!(DreadRung::from_level(50), DreadRung::Wrong);
        assert_eq!(DreadRung::from_level(74), DreadRung::Wrong);
        assert_eq!(DreadRung::from_level(75), DreadRung::Event);
        assert_eq!(DreadRung::from_level(100), DreadRung::Event);
    }
}
