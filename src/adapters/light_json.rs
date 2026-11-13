//! JSON writer for light sources and the daylight cycle.
//!
//! Fixed field order and two-decimal floats, so the same lights always
//! render byte-identical JSON. The shell renders this; it never
//! computes light or darkness itself.

use crate::domain::light::{Daylight, LightSource};

/// Renders a house's lights as a single-line JSON array. `room` is
/// null for a carried light.
pub fn render_lights_json(lights: &[LightSource]) -> String {
    let mut s = String::from("[");
    for (i, light) in lights.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let room = match light.room_id {
            Some(id) => id.to_string(),
            None => "null".to_string(),
        };
        s.push_str(&format!(
            "{{\"id\":{},\"kind\":\"{}\",\"room\":{},\"x\":{:.2},\"y\":{:.2},\"radius\":{:.2},\"state\":\"{}\"}}",
            light.id,
            light.kind.id(),
            room,
            light.x,
            light.y,
            light.radius,
            light.state.id()
        ));
    }
    s.push(']');
    s
}

/// Renders the daylight cycle: the clock, whether it is held, the sun
/// elevation, and the named phase.
pub fn render_daylight_json(daylight: &Daylight) -> String {
    format!(
        "{{\"minutes\":{},\"held\":{},\"elevation\":{:.2},\"phase\":\"{}\"}}",
        daylight.minutes,
        daylight.held,
        daylight.elevation(),
        daylight.phase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::light::{LightKind, LightState};

    fn a_light() -> LightSource {
        LightSource {
            id: 1000,
            kind: LightKind::Ceiling,
            room_id: Some(4),
            x: 1.0,
            y: 2.0,
            radius: 6.0,
            state: LightState::On,
        }
    }

    #[test]
    fn lights_json_carries_kind_room_and_state() {
        let json = render_lights_json(&[a_light()]);
        assert!(json.contains("\"kind\":\"ceiling\""));
        assert!(json.contains("\"room\":4"));
        assert!(json.contains("\"state\":\"on\""));
    }

    #[test]
    fn a_carried_light_has_a_null_room() {
        let mut light = a_light();
        light.kind = LightKind::Flashlight;
        light.room_id = None;
        assert!(render_lights_json(&[light]).contains("\"room\":null"));
    }

    #[test]
    fn daylight_json_names_the_phase() {
        assert!(render_daylight_json(&Daylight::at(720)).contains("\"phase\":\"day\""));
        assert!(render_daylight_json(&Daylight::at(0)).contains("\"phase\":\"night\""));
        assert!(render_daylight_json(&Daylight::held_at(720)).contains("\"held\":true"));
    }
}
