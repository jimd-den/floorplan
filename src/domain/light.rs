//! Light play data: sources, switches, and the daylight cycle.
//!
//! Pure data plus the pure state transitions (`LightState::toggle`,
//! `Daylight::advance`, `Daylight::elevation`). Deriving a house's
//! lights and coupling them to dread lives in `use_cases::light`;
//! the shell only renders and sends commands.

/// What a light is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    Ceiling,
    Lamp,
    Porch,
    Flashlight,
    Window,
}

impl LightKind {
    /// Lower-case name for the JSON writer.
    pub fn id(&self) -> &'static str {
        match self {
            LightKind::Ceiling => "ceiling",
            LightKind::Lamp => "lamp",
            LightKind::Porch => "porch",
            LightKind::Flashlight => "flashlight",
            LightKind::Window => "window",
        }
    }
}

/// A light's state. A broken light cannot be switched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightState {
    On,
    Off,
    Flicker,
    Broken,
}

impl LightState {
    /// Lower-case name for the JSON writer.
    pub fn id(&self) -> &'static str {
        match self {
            LightState::On => "on",
            LightState::Off => "off",
            LightState::Flicker => "flicker",
            LightState::Broken => "broken",
        }
    }

    /// The state a switch produces. Broken stays broken; a flicker
    /// light switches off.
    pub fn toggle(&self) -> LightState {
        match self {
            LightState::On => LightState::Off,
            LightState::Off => LightState::On,
            LightState::Flicker => LightState::Off,
            LightState::Broken => LightState::Broken,
        }
    }

    /// A code the shell can pass back across the boundary.
    pub fn code(&self) -> u32 {
        match self {
            LightState::On => 0,
            LightState::Off => 1,
            LightState::Flicker => 2,
            LightState::Broken => 3,
        }
    }

    /// The state for a code, or `None` when the code is unknown.
    pub fn from_code(code: u32) -> Option<LightState> {
        match code {
            0 => Some(LightState::On),
            1 => Some(LightState::Off),
            2 => Some(LightState::Flicker),
            3 => Some(LightState::Broken),
            _ => None,
        }
    }
}

/// One light: where it is, how far it reaches, and what it is doing.
/// `room_id` is `None` for a carried light (the flashlight).
#[derive(Debug, Clone, PartialEq)]
pub struct LightSource {
    pub id: u32,
    pub kind: LightKind,
    pub room_id: Option<u32>,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub state: LightState,
}

/// The daylight cycle: minutes since midnight, and whether the sandbox
/// has frozen it. Minutes wrap at 1440.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Daylight {
    pub minutes: u32,
    pub held: bool,
}

impl Daylight {
    /// The cycle at a time of day, running.
    pub fn at(minutes: u32) -> Self {
        Self { minutes: minutes % 1440, held: false }
    }

    /// The cycle at a time of day, held there (the sandbox default).
    pub fn held_at(minutes: u32) -> Self {
        Self { minutes: minutes % 1440, held: true }
    }

    /// Advances the clock unless the sandbox holds it.
    pub fn advance(&mut self, minutes: u32) {
        if !self.held {
            self.minutes = (self.minutes + minutes) % 1440;
        }
    }

    /// Sun elevation, -1 (midnight) to 1 (noon). Six o'clock is the
    /// horizon, so windows admit no light at exactly dawn or dusk.
    pub fn elevation(&self) -> f64 {
        let angle = (self.minutes as f64 - 360.0) / 1440.0 * std::f64::consts::TAU;
        angle.sin()
    }

    /// True when the sun is above the horizon.
    pub fn is_day(&self) -> bool {
        self.elevation() > 0.0
    }

    /// The named phase, for the shell and the event log.
    pub fn phase(&self) -> &'static str {
        let elevation = self.elevation();
        if elevation > 0.25 {
            "day"
        } else if elevation > 0.0 {
            if self.minutes < 720 { "dawn" } else { "dusk" }
        } else {
            "night"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_switch_flips_on_and_off_and_never_repairs() {
        assert_eq!(LightState::On.toggle(), LightState::Off);
        assert_eq!(LightState::Off.toggle(), LightState::On);
        assert_eq!(LightState::Flicker.toggle(), LightState::Off);
        assert_eq!(LightState::Broken.toggle(), LightState::Broken);
    }

    #[test]
    fn state_codes_round_trip() {
        for state in [LightState::On, LightState::Off, LightState::Flicker, LightState::Broken] {
            assert_eq!(LightState::from_code(state.code()), Some(state));
        }
        assert_eq!(LightState::from_code(9), None);
    }

    #[test]
    fn noon_is_day_and_midnight_is_night() {
        assert!((Daylight::at(720).elevation() - 1.0).abs() < 1e-9);
        assert!((Daylight::at(0).elevation() + 1.0).abs() < 1e-9);
        assert_eq!(Daylight::at(720).phase(), "day");
        assert_eq!(Daylight::at(0).phase(), "night");
        assert_eq!(Daylight::at(400).phase(), "dawn");
        assert_eq!(Daylight::at(1070).phase(), "dusk");
    }

    #[test]
    fn a_held_cycle_does_not_advance() {
        let mut running = Daylight::at(720);
        running.advance(120);
        assert_eq!(running.minutes, 840);
        let mut held = Daylight::held_at(720);
        held.advance(120);
        assert_eq!(held.minutes, 720);
    }

    #[test]
    fn minutes_wrap_at_a_day() {
        let mut daylight = Daylight::at(1400);
        daylight.advance(100);
        assert_eq!(daylight.minutes, 60);
    }
}
