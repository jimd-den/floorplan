//! Street lamps: regular spacing, both sidewalks, arms over the road,
//! and nothing standing on a turnaround bulb.

use floorplan::domain::geometry::Rect;
use floorplan::domain::light::{LightKind, LightState};
use floorplan::domain::neighborhood::Street;
use floorplan::domain::street_furniture::LAMP_REACH;
use floorplan::use_cases::street_lights::{
    LAMP_FIRST, LAMP_OFFSET, LAMP_SPACING, lamps_along,
};
use floorplan::use_cases::world_streets::streets_in;

fn streets(seed: u32) -> Vec<Street> {
    streets_in(seed, Rect::new(0.0, 0.0, 140.0, 140.0))
}

/// The arc length of the sample nearest `p` on `street`.
fn nearest_s(street: &Street, p: floorplan::domain::curve::Pt) -> f64 {
    let mut best = (f64::MAX, 0.0);
    for &s in &street.samples.lengths {
        let d = street.samples.point_at(s).dist(p);
        if d < best.0 {
            best = (d, s);
        }
    }
    best.1
}

/// Signed lateral offset of `p` from the centerline: + on the
/// street's normal side.
fn lateral(street: &Street, p: floorplan::domain::curve::Pt) -> f64 {
    let s = nearest_s(street, p);
    let n = street.samples.normal_at(s);
    p.sub(street.samples.point_at(s)).dot(n)
}

#[test]
fn lamps_along_a_street_are_deterministic() {
    let s = &streets(42)[0];
    assert_eq!(lamps_along(s), lamps_along(s));
    assert!(!lamps_along(s).is_empty(), "a 150 m street has no lamps");
}

#[test]
fn every_lamp_stands_on_the_sidewalk_and_reaches_over_the_road() {
    for seed in 0..3u32 {
        for s in streets(seed) {
            for lamp in lamps_along(&s) {
                let pole = lateral(&s, lamp.position).abs();
                assert!(
                    (pole - LAMP_OFFSET).abs() < 0.6,
                    "seed {seed}: pole {pole:.2} m from the centerline"
                );
                let head = lateral(&s, lamp.head());
                assert!(
                    head.abs() < pole - 1.0,
                    "seed {seed}: the arm does not reach the road ({head:.2} vs {pole:.2})"
                );
                assert!(
                    (lamp.head().dist(lamp.position) - LAMP_REACH).abs() < 1e-9,
                    "seed {seed}: the arm is not {LAMP_REACH} m"
                );
            }
        }
    }
}

#[test]
fn lamps_alternate_sides_so_both_sidewalks_are_lit() {
    let mut left = 0;
    let mut right = 0;
    for seed in 0..3u32 {
        for s in streets(seed) {
            for lamp in lamps_along(&s) {
                if lateral(&s, lamp.position) > 0.0 {
                    right += 1;
                } else {
                    left += 1;
                }
            }
        }
    }
    assert!(left > 0 && right > 0, "one sidewalk never gets a lamp ({left}/{right})");
}

#[test]
fn lamps_are_spaced_along_the_street() {
    let s = &streets(42)[0];
    let mut arc: Vec<f64> = lamps_along(s).iter().map(|l| nearest_s(s, l.position)).collect();
    arc.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(arc.len() >= 4, "only {} lamps", arc.len());
    assert!((arc[0] - LAMP_FIRST).abs() < 0.6, "first lamp at {:.1} m", arc[0]);
    for w in arc.windows(2) {
        let gap = w[1] - w[0];
        assert!((gap - LAMP_SPACING).abs() < 1.0, "lamp gap {gap:.1} m");
    }
}

#[test]
fn no_lamp_stands_on_a_turnaround_bulb() {
    let mut checked = 0;
    for seed in 0..12u32 {
        for s in streets(seed) {
            let Some(bulb_at) = s.bulb_at else { continue };
            for lamp in lamps_along(&s) {
                let at = nearest_s(&s, lamp.position);
                assert!(at < bulb_at, "seed {seed}: a lamp stands on the bulb ({at:.1} of {bulb_at:.1})");
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "no street with a bulb carried a lamp");
}

#[test]
fn every_lamp_carries_a_lamp_light_at_its_head() {
    let s = &streets(42)[0];
    for (i, lamp) in lamps_along(s).iter().enumerate() {
        let light = lamp.light(i as u32);
        assert_eq!(light.kind, LightKind::Lamp);
        assert_eq!(light.state, LightState::On);
        assert!(light.radius > 0.0);
        assert!((light.x - lamp.head().x).abs() < 1e-9);
    }
}
