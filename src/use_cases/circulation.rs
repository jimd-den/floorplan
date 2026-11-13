//! Circulation extras: extra doors between living, kitchen and
//! dining where they share a wall but the split tree gave them none.

use crate::domain::model::{Opening, Program, Room};
use crate::frameworks_drivers::rng::Rng;
use crate::use_cases::doors::{Adjacency, make_door};

/// Circulation extras between living / kitchen / dining where they
/// share a wall but have no door yet — at most two, seeded.
pub(crate) fn extras(
    rng: &mut Rng,
    rooms: &[Room],
    adj: &[Adjacency],
    openings: &[Opening],
    next_id: &mut u32,
) -> Vec<Opening> {
    let wanted = [Program::Living, Program::Kitchen, Program::Dining];
    let ids: Vec<u32> = rooms
        .iter()
        .filter(|r| wanted.contains(&r.program))
        .map(|r| r.id)
        .collect();
    let mut extra = vec![];
    for (i, &a) in ids.iter().enumerate() {
        for &b in ids.iter().skip(i + 1) {
            if extra.len() >= 2 {
                break;
            }
            let has = openings
                .iter()
                .chain(extra.iter())
                .any(|o| (o.from == a && o.to == Some(b)) || (o.from == b && o.to == Some(a)));
            if has {
                continue;
            }
            let pair = adj
                .iter()
                .find(|x| (x.a == a && x.b == b) || (x.a == b && x.b == a));
            // A door is 0.6 m at its narrowest: a shared wall shorter
            // than that cannot hold one, and a wider door would spill
            // across the neighbouring walls.
            if let Some(pair) = pair
                && pair.segment.len() >= 0.6
                && rng.chance(0.5)
            {
                extra.push(make_door(rng, pair, next_id));
            }
        }
    }
    extra
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn no_extra_door_on_a_wall_shorter_than_a_door() {
        // Living and kitchen share a 0.1 m sliver of wall: no door
        // fits there, and a 0.6 m door would spill across the
        // neighbouring walls.
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 4.0, 4.0)),
            room(1, Program::Kitchen, Rect::new(4.0, 0.0, 4.0, 4.0)),
            room(2, Program::Dining, Rect::new(4.0, 3.9, 4.0, 4.0)),
        ];
        let adj = crate::use_cases::doors::adjacency(&rooms);
        let mut rng = Rng::new(1);
        let mut next_id = 0;
        let extra = extras(&mut rng, &rooms, &adj, &[], &mut next_id);
        // Living and kitchen share a 4 m wall, so they may get a
        // door; living and dining share 0.1 m and must not.
        for d in &extra {
            assert!(
                !((d.from == 0 && d.to == Some(2)) || (d.from == 2 && d.to == Some(0))),
                "a door was placed on the 0.1 m living/dining wall"
            );
        }
    }
}
