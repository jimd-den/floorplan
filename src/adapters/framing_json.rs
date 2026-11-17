//! framing_json: the structural frame as a JSON fragment, spliced
//! into the house document by `adapters::json`. Fixed field order,
//! two-decimal floats, no whitespace — the same house always
//! renders byte-identical JSON. Split out of `json.rs` when the
//! frame joined the document and the writer passed the line ceiling.

use crate::adapters::json::axis_name;
use crate::domain::model::House;

/// The framing as JSON: wall frames (segment, stud offsets, framed
/// openings), floor frames (rect, joist offsets, direction), and
/// the roof frame (ridge, rafter offsets, span).
pub(crate) fn framing_json(house: &House) -> String {
    let f = &house.framing;
    let mut s = String::from("{\"walls\":[");
    for (i, wf) in f.wall_frames.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"seg\":{{\"x1\":{:.2},\"y1\":{:.2},\"x2\":{:.2},\"y2\":{:.2}}},\"base\":{:.2},\"studs\":[",
            wf.segment.x1, wf.segment.y1, wf.segment.x2, wf.segment.y2, wf.base
        ));
        for (j, st) in wf.studs.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!("{st:.2}"));
        }
        s.push_str("],\"openings\":[");
        for (j, o) in wf.openings.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!(
                "{{\"id\":{},\"lo\":{:.2},\"hi\":{:.2},\"header\":{:.2}}}",
                o.opening_id, o.lo, o.hi, o.header_y
            ));
        }
        s.push_str("]}");
    }
    s.push_str("],\"floors\":[");
    for (i, ff) in f.floor_frames.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"rect\":{{\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"d\":{:.2}}},\"base\":{:.2},\"joists\":[",
            ff.rect.x, ff.rect.y, ff.rect.w, ff.rect.d, ff.base
        ));
        for (j, jo) in ff.joists.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!("{jo:.2}"));
        }
        s.push_str(&format!("],\"direction\":\"{}\"}}", axis_name(ff.direction)));
    }
    let roof = &f.roof_frame;
    s.push_str(&format!(
        "],\"roof\":{{\"ridge\":{{\"x1\":{:.2},\"y1\":{:.2},\"x2\":{:.2},\"y2\":{:.2}}},\"rafters\":[",
        roof.ridge.x1, roof.ridge.y1, roof.ridge.x2, roof.ridge.y2
    ));
    for (i, ra) in roof.rafters.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("{ra:.2}"));
    }
    s.push_str(&format!("],\"span\":{:.2}}}}}", roof.span));
    s
}
