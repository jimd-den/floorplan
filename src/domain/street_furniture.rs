//! Street furniture: lamps and signs placed along a street.
//!
//! Pure data plus the two derived values the mesh and the shell both
//! need: where a lamp's head hangs, and the light it casts. A sign
//! carries its plate text in the document, never in the mesh — a mesh
//! has no font, so the shell textures the face. `rotation` uses the
//! house convention (degrees, clockwise from south on a map), so the
//! shell rotates furniture and houses the same way.

use crate::domain::curve::Pt;
use crate::domain::light::{LightKind, LightSource, LightState};
use crate::domain::placement::front_for;

/// How far a lamp arm reaches from its pole, in metres.
pub const LAMP_REACH: f64 = 2.4;
/// How far a lamp's light reaches, in metres.
pub const LAMP_LIGHT_RADIUS: f64 = 14.0;
/// The height of a lamp head, in metres.
pub const LAMP_HEIGHT: f64 = 4.7;

/// A street lamp: a pole on the sidewalk whose arm reaches over the
/// road, with the light hanging at the arm's end.
#[derive(Debug, Clone, PartialEq)]
pub struct StreetLamp {
    /// The pole's base, on the sidewalk.
    pub position: Pt,
    /// Degrees, clockwise from south: the arm faces the road.
    pub rotation: f64,
}

impl StreetLamp {
    /// The lamp head: `LAMP_REACH` metres toward the road from the
    /// pole, at `LAMP_HEIGHT`.
    pub fn head(&self) -> Pt {
        self.position.add(front_for(self.rotation).scale(LAMP_REACH))
    }

    /// The light this lamp casts. The caller assigns the id, because
    /// ids are per document, not per street.
    pub fn light(&self, id: u32) -> LightSource {
        let head = self.head();
        LightSource {
            id,
            kind: LightKind::Lamp,
            room_id: None,
            x: head.x,
            y: head.y,
            radius: LAMP_LIGHT_RADIUS,
            state: LightState::On,
        }
    }
}

/// What a sign says. The text is in `RoadSign`, not in the mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignKind {
    Stop,
    StreetName,
    SpeedLimit,
}

impl SignKind {
    /// Lower-case name for the JSON writer.
    pub fn id(&self) -> &'static str {
        match self {
            SignKind::Stop => "stop",
            SignKind::StreetName => "street_name",
            SignKind::SpeedLimit => "speed_limit",
        }
    }
}

/// A road sign: a post with a plate, facing the driver who must
/// read it.
#[derive(Debug, Clone, PartialEq)]
pub struct RoadSign {
    /// The post's base, on the sidewalk.
    pub position: Pt,
    /// Degrees, clockwise from south: the plate faces the driver.
    pub rotation: f64,
    pub kind: SignKind,
    /// The plate's text: `STOP`, a street name, or a speed limit.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lamp_head_hangs_over_the_road_from_its_pole() {
        // Rotation -90 faces east (see domain::placement), so the arm
        // reaches 2.4 m east of the pole.
        let lamp = StreetLamp { position: Pt::new(10.0, 5.0), rotation: -90.0 };
        let head = lamp.head();
        assert!((head.x - 12.4).abs() < 1e-9, "head at x {:.2}", head.x);
        assert!((head.y - 5.0).abs() < 1e-9, "head moved along the street");
    }

    #[test]
    fn a_lamp_casts_an_on_lamp_light_at_its_head() {
        let lamp = StreetLamp { position: Pt::new(0.0, 0.0), rotation: 0.0 };
        let light = lamp.light(7);
        assert_eq!(light.id, 7);
        assert_eq!(light.kind, LightKind::Lamp);
        assert_eq!(light.state, LightState::On);
        assert_eq!(light.room_id, None, "a street lamp is not in a room");
        assert!((light.x - lamp.head().x).abs() < 1e-9);
        assert!((light.y - lamp.head().y).abs() < 1e-9);
        assert_eq!(light.radius, LAMP_LIGHT_RADIUS);
    }

    #[test]
    fn sign_kinds_have_stable_names() {
        assert_eq!(SignKind::Stop.id(), "stop");
        assert_eq!(SignKind::StreetName.id(), "street_name");
        assert_eq!(SignKind::SpeedLimit.id(), "speed_limit");
    }
}
