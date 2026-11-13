//! JSON writer: the plan model as a hand-rolled JSON document.
//!
//! Fixed field order, two-decimal floats, no whitespace — the same
//! house always renders byte-identical JSON.

use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::{House, OpeningKind};

/// Renders a whole house as a single-line JSON document.
pub fn render_json(house: &House) -> String {
    render(house, true)
}

/// Renders the plan without the seed — the canonical form the
/// variety hash is taken over, so distinct hashes mean distinct
/// plans rather than distinct seeds.
pub fn render_plan_json(house: &House) -> String {
    render(house, false)
}

fn render(house: &House, with_seed: bool) -> String {
    let mut s = String::new();
    if with_seed {
        s.push_str(&format!(
            "{{\"seed\":{},\"style\":\"{}\",\"footprint\":{{",
            house.seed,
            house.style.id()
        ));
    } else {
        s.push_str(&format!("{{\"style\":\"{}\",\"footprint\":{{", house.style.id()));
    }
    s.push_str("\"rects\":[");
    for (i, r) in house.footprint.rects.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"d\":{:.2}}}",
            r.x, r.y, r.w, r.d
        ));
    }
    s.push_str("],\"joins\":[");
    for (i, j) in house.footprint.joins.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"x1\":{:.2},\"y1\":{:.2},\"x2\":{:.2},\"y2\":{:.2}}}",
            j.x1, j.y1, j.x2, j.y2
        ));
    }
    s.push_str("],\"porch\":");
    s.push_str(&opt_rect(house.footprint.porch));
    s.push_str(",\"garage\":");
    s.push_str(&opt_rect(house.footprint.garage));
    s.push_str("},\"floors\":[");
    for (i, f) in house.floors.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{{\"level\":{},\"rooms\":[", f.level));
        for (j, r) in f.rooms.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"id\":{},\"program\":\"{}\",\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"d\":{:.2}}}",
                r.id,
                r.program.name(),
                r.rect.x,
                r.rect.y,
                r.rect.w,
                r.rect.d
            ));
        }
        s.push_str("],\"openings\":[");
        for (j, o) in f.openings.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"id\":{},\"kind\":\"{}\",\"from\":{},\"to\":{},\"x\":{:.2},\"y\":{:.2},\"axis\":\"{}\",\"width\":{:.2}}}",
                o.id,
                kind_name(o.kind),
                o.from,
                opt_id(o.to),
                o.x,
                o.y,
                axis_name(o.axis),
                o.width
            ));
        }
        if with_seed {
            s.push_str("],\"furniture\":[");
            for (j, p) in f.furniture.iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                s.push_str(&format!(
                    "{{\"id\":{},\"kind\":\"{}\",\"room\":{},\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"d\":{:.2},\"facing\":\"{}\",\"state\":\"{}\"}}",
                    p.id,
                    p.kind.name(),
                    p.room_id,
                    p.rect.x,
                    p.rect.y,
                    p.rect.w,
                    p.rect.d,
                    p.facing.name(),
                    state_name(p.state)
                ));
            }
            s.push_str("],\"open_doors\":[");
            for (j, id) in f.open_doors.iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                s.push_str(&id.to_string());
            }
            s.push_str("]}");
        } else {
            s.push_str("]}");
        }
    }
    s.push_str("],\"stairs\":");
    match house.stairs {
        Some((a, b)) => s.push_str(&format!("[{a},{b}]")),
        None => s.push_str("null"),
    }
    s.push('}');
    s
}

fn opt_rect(rect: Option<Rect>) -> String {
    match rect {
        Some(r) => format!("{{\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"d\":{:.2}}}", r.x, r.y, r.w, r.d),
        None => "null".to_string(),
    }
}

fn opt_id(id: Option<u32>) -> String {
    match id {
        Some(i) => i.to_string(),
        None => "null".to_string(),
    }
}

fn kind_name(kind: OpeningKind) -> &'static str {
    match kind {
        OpeningKind::Door => "door",
        OpeningKind::EntryDoor => "entry",
        OpeningKind::Window => "window",
    }
}

fn axis_name(axis: Axis) -> &'static str {
    match axis {
        Axis::Horizontal => "horizontal",
        Axis::Vertical => "vertical",
    }
}

fn state_name(state: crate::domain::furniture::FurnitureState) -> &'static str {
    use crate::domain::furniture::FurnitureState;
    match state {
        FurnitureState::Fixed => "fixed",
        FurnitureState::On => "on",
        FurnitureState::Off => "off",
        FurnitureState::Open => "open",
        FurnitureState::Closed => "closed",
        FurnitureState::Moved { .. } => "moved",
    }
}
