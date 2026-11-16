//! Road signs: stop at junction approaches, street names on corners,
//! speed limits at intervals, all facing the driver who reads them.

use floorplan::domain::curve::{Polyline, Pt};
use floorplan::domain::geometry::Rect;
use floorplan::domain::neighborhood::Street;
use floorplan::domain::placement::front_for;
use floorplan::domain::street_furniture::SignKind;
use floorplan::use_cases::road_signs::{
    EndKind, SIGN_CLEAR, SIGN_OFFSET, SPEED_LIMITS, STREET_NAMES, StreetEnds, signs_along,
};
use floorplan::use_cases::world_streets::streets_in;

/// A straight 120 m street running east from the origin.
fn straight() -> Street {
    let control = vec![Pt::new(0.0, 0.0), Pt::new(40.0, 0.0), Pt::new(80.0, 0.0), Pt::new(120.0, 0.0)];
    let samples = Polyline::new(control.clone());
    Street { control, samples, bulb_at: None, bulb_center: None, junctions: Vec::new() }
}

fn ends(start: EndKind, end: EndKind) -> StreetEnds {
    StreetEnds { start, end }
}

fn nearest_s(street: &Street, p: Pt) -> f64 {
    let mut best = (f64::MAX, 0.0);
    for &s in &street.samples.lengths {
        let d = street.samples.point_at(s).dist(p);
        if d < best.0 {
            best = (d, s);
        }
    }
    best.1
}

fn lateral(street: &Street, p: Pt) -> f64 {
    let s = nearest_s(street, p);
    p.sub(street.samples.point_at(s)).dot(street.samples.normal_at(s))
}

#[test]
fn a_junction_approach_gets_a_stop_sign_facing_the_driver() {
    let s = straight();
    let signs = signs_along(&s, 42, ends(EndKind::Junction, EndKind::DeadEnd));
    let stop = signs.iter().find(|g| g.kind == SignKind::Stop).expect("no stop sign");
    assert_eq!(stop.text, "STOP");
    assert!((stop.position.x - SIGN_CLEAR).abs() < 1e-9, "stop sign at x {:.1}", stop.position.x);
    assert!((stop.position.y + SIGN_OFFSET).abs() < 1e-9, "stop sign on the wrong side");
    // The driver approaches the start travelling west, so the plate
    // faces east: -90° in the house convention.
    assert!((stop.rotation + 90.0).abs() < 1.0, "plate faces {:.0}°", stop.rotation);
    assert!(lateral(&s, stop.position) < 0.0, "stop sign is not on the driver's right");
}

#[test]
fn a_dead_end_gets_no_stop_sign() {
    let s = straight();
    let signs = signs_along(&s, 42, ends(EndKind::DeadEnd, EndKind::DeadEnd));
    assert!(signs.iter().all(|g| g.kind != SignKind::Stop), "a dead end got a stop sign");
}

#[test]
fn every_junction_approach_gets_a_street_name_sign() {
    let s = straight();
    let signs = signs_along(&s, 3, ends(EndKind::Junction, EndKind::Junction));
    let names: Vec<_> = signs.iter().filter(|g| g.kind == SignKind::StreetName).collect();
    assert_eq!(names.len(), 2, "one name sign per junction end");
    for n in names {
        assert!(n.text.ends_with(" ST"), "street name {:?}", n.text);
        assert!(
            STREET_NAMES.iter().any(|w| n.text.starts_with(w)),
            "unknown street name {:?}",
            n.text
        );
    }
}

#[test]
fn speed_limit_signs_carry_one_of_the_limits() {
    let s = straight();
    let signs = signs_along(&s, 42, ends(EndKind::Junction, EndKind::Junction));
    let limits: Vec<_> = signs.iter().filter(|g| g.kind == SignKind::SpeedLimit).collect();
    assert!(!limits.is_empty(), "a 120 m street has no speed-limit sign");
    for l in limits {
        let mph: u32 = l.text.parse().expect("speed limit is not a number");
        assert!(SPEED_LIMITS.contains(&mph), "{mph} is not a posted limit");
    }
}

#[test]
fn every_sign_faces_along_the_street_it_stands_on() {
    for seed in 0..3u32 {
        for s in streets_in(seed, Rect::new(0.0, 0.0, 140.0, 140.0)) {
            for g in signs_along(&s, seed, ends(EndKind::Junction, EndKind::Junction)) {
                let s_at = nearest_s(&s, g.position);
                let along = front_for(g.rotation).dot(s.samples.tangent_at(s_at)).abs();
                assert!(along > 0.98, "seed {seed}: a {:?} plate is turned away", g.kind);
                let off = lateral(&s, g.position).abs();
                assert!((off - SIGN_OFFSET).abs() < 0.7, "seed {seed}: post {off:.2} m off the sidewalk");
            }
        }
    }
}

#[test]
fn signs_are_deterministic_and_seeded() {
    let s = straight();
    let both = ends(EndKind::Junction, EndKind::Junction);
    assert_eq!(signs_along(&s, 42, both), signs_along(&s, 42, both));
    let name = |seed: u32| {
        signs_along(&s, seed, both)
            .into_iter()
            .find(|g| g.kind == SignKind::StreetName)
            .expect("no street-name sign")
            .text
    };
    assert_ne!(name(42), name(43), "the street name ignores the seed");
}

#[test]
fn no_sign_stands_on_a_turnaround_bulb() {
    let mut checked = 0;
    for seed in 0..12u32 {
        for s in streets_in(seed, Rect::new(0.0, 0.0, 140.0, 140.0)) {
            let Some(bulb_at) = s.bulb_at else { continue };
            for g in signs_along(&s, seed, ends(EndKind::Junction, EndKind::DeadEnd)) {
                assert!(nearest_s(&s, g.position) < bulb_at, "seed {seed}: a sign stands on the bulb");
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "no bulb street carried a sign");
}
