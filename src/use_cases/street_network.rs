//! Street network: a curving spine with cul-de-sac branches.
//!
//! One spine runs across the area; branches curve off it and end
//! in turnaround bulbs (a 300° arc, radius 14 m). The network is
//! a tree, so it is connected by construction.
//! Streets are Catmull-Rom centerlines sampled about every 2 m.

use crate::domain::curve::{Polyline, Pt};
use crate::domain::neighborhood::Street;
use crate::domain::spline::sample_catmull_rom;
use crate::frameworks_drivers::rng::Rng;

/// Nominal street-cell size, in metres (the old block was 76–96 m).
pub const CELL: f64 = 85.0;
/// Margin around the network, in metres.
pub const MARGIN: f64 = 20.0;
/// Spline sample spacing, in metres.
pub const SAMPLE_STEP: f64 = 2.0;
/// Turnaround bulb radius, in metres.
pub const BULB_RADIUS: f64 = 14.0;
/// No lots within this arc length of a junction, in metres.
pub const JUNCTION_CLEAR: f64 = 12.0;

/// Plans the street network for a neighborhood of `blocks`
/// (columns, rows) of street cells. Street 0 is the spine; the
/// rest are branches, each starting on the spine.
pub fn plan_network(rng: &mut Rng, blocks: (u32, u32)) -> Vec<Street> {
    let (cols, rows) = (blocks.0.max(1), blocks.1.max(1));
    let w = cols as f64 * CELL + 2.0 * MARGIN;
    let h = rows as f64 * CELL + 2.0 * MARGIN;
    let mut streets = vec![spine_street(rng, cols, w, h)];
    let n_branches = 1 + rng.range_usize(cols.min(rows) as usize);
    let spine_len = streets[0].samples.len();
    let start_north = rng.chance(0.5);
    for k in 0..n_branches {
        let want = spine_len * (k as f64 + 1.0) / (n_branches as f64 + 1.0)
            + rng.range_f64(-15.0, 15.0);
        let (start, s) = nearest_sample(&streets[0].samples, want);
        let north = (k % 2 == 0) == start_north;
        let side = if north { -1 } else { 1 };
        let branch = branch_street(rng, &streets[0], start, s, side, rows);
        streets[0].junctions.push(s);
        streets.push(branch);
    }
    streets
}

/// The spine: a gently meandering east-west spline across the
/// area, entering and leaving at mid-height.
fn spine_street(rng: &mut Rng, cols: u32, w: f64, h: f64) -> Street {
    let n_points = cols as usize + 2;
    let mut control = Vec::new();
    for i in 0..n_points {
        let x = i as f64 * w / (n_points - 1) as f64;
        let y = if i == 0 || i == n_points - 1 {
            h / 2.0
        } else {
            h / 2.0 + rng.range_f64(-22.0, 22.0)
        };
        control.push(Pt::new(x, y));
    }
    Street {
        control: control.clone(),
        samples: Polyline::new(sample_catmull_rom(&control, SAMPLE_STEP)),
        bulb_at: None,
        bulb_center: None,
        junctions: Vec::new(),
    }
}

/// A branch: a spline leaving the spine at `start` (a spine
/// sample at arc length `s`), curving to `side` (+1 is the
/// spine's +normal side, -1 the other) and ending in a bulb.
fn branch_street(rng: &mut Rng, spine: &Street, start: Pt, s: f64, side: i8, rows: u32) -> Street {
    let tangent = spine.samples.tangent_at(s);
    let dir = spine.samples.normal_at(s).scale(side as f64);
    // Long enough that even the shortest branch carries lots:
    // a 50 m approach leaves under one frontage after the
    // junction clearance, so short branches were nearly empty.
    let len = rng.range_f64(70.0, 70.0 + 20.0 * rows as f64);
    let mid = start.add(dir.scale(len * 0.5)).add(tangent.scale(rng.range_f64(-10.0, 10.0)));
    let end = start.add(dir.scale(len)).add(tangent.scale(rng.range_f64(-18.0, 18.0)));
    let control = vec![start, mid, end];
    let mut samples = sample_catmull_rom(&control, SAMPLE_STEP);
    let approach = Polyline::new(samples.clone());
    let bulb_at = approach.len();
    let t_end = approach.tangent_at(bulb_at);
    let left = rng.chance(0.5);
    let n = if left { t_end.perp() } else { t_end.perp().scale(-1.0) };
    let centre = end.add(n.scale(BULB_RADIUS));
    samples.extend(bulb_arc(end, centre, left));
    Street {
        control,
        samples: Polyline::new(samples),
        bulb_at: Some(bulb_at),
        bulb_center: Some(centre),
        junctions: Vec::new(),
    }
}

/// The 300° turnaround arc around `centre`, starting at `end`
/// (which is on the circle) and continuing the approach tangent.
fn bulb_arc(end: Pt, centre: Pt, left: bool) -> Vec<Pt> {
    let radius = end.sub(centre).len();
    let start_angle = (end.y - centre.y).atan2(end.x - centre.x);
    let sweep = 5.0 * std::f64::consts::PI / 3.0;
    let dir = if left { 1.0 } else { -1.0 };
    let n = (radius * sweep / SAMPLE_STEP).ceil().max(1.0) as usize;
    let mut out = Vec::new();
    for k in 1..=n {
        let a = start_angle + dir * sweep * k as f64 / n as f64;
        out.push(Pt::new(centre.x + radius * a.cos(), centre.y + radius * a.sin()));
    }
    out
}

/// The spine sample nearest arc length `want`, and its arc length.
fn nearest_sample(samples: &Polyline, want: f64) -> (Pt, f64) {
    let mut best = (samples.points[0], samples.lengths[0]);
    let mut best_d = f64::MAX;
    for (i, p) in samples.points.iter().enumerate() {
        let d = (samples.lengths[i] - want).abs();
        if d < best_d {
            best_d = d;
            best = (*p, samples.lengths[i]);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(seed: u32) -> Vec<Street> {
        plan_network(&mut Rng::new(seed), (2, 2))
    }

    #[test]
    fn the_network_is_deterministic() {
        assert_eq!(network(42), network(42));
    }

    #[test]
    fn different_seeds_give_different_networks() {
        assert_ne!(network(1), network(2));
    }

    #[test]
    fn the_spine_spans_the_area_and_branches_join_it() {
        let streets = network(42);
        assert!(!streets.is_empty(), "no streets");
        let spine = &streets[0];
        assert!(spine.samples.len() >= 2.0 * CELL, "spine too short");
        for b in streets.iter().skip(1) {
            let start = b.control[0];
            let near_spine = spine
                .samples
                .points
                .iter()
                .any(|p| p.dist(start) < 0.5);
            assert!(near_spine, "branch starts off the spine");
        }
    }

    #[test]
    fn every_branch_ends_in_a_bulb() {
        for seed in 0..20u32 {
            for b in network(seed).iter().skip(1) {
                assert!(b.bulb_at.is_some(), "seed {seed}: branch has no bulb");
                assert!(b.bulb_center.is_some(), "seed {seed}: bulb has no centre");
            }
        }
    }

    #[test]
    fn streets_are_smooth() {
        for seed in 0..20u32 {
            for s in network(seed) {
                let mut worst = 0.0f64;
                for w in s.samples.points.windows(3) {
                    let a = w[1].sub(w[0]).unit();
                    let b = w[2].sub(w[1]).unit();
                    let turn = a.dot(b).clamp(-1.0, 1.0).acos().to_degrees();
                    worst = worst.max(turn);
                }
                assert!(worst < 30.0, "seed {seed}: sharpest turn {worst:.1}°");
            }
        }
    }

    #[test]
    fn every_street_is_long_enough_to_carry_lots() {
        for seed in 0..20u32 {
            for s in network(seed) {
                assert!(s.samples.len() >= 40.0, "seed {seed}: street only {:.1} m", s.samples.len());
            }
        }
    }
}
