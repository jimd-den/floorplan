//! Build a floor's skeleton from its rooms: every shared-wall link,
//! marked backbone when it crosses a BSP split or a footprint join.
//! The backbone is the connectivity skeleton — one door per backbone
//! link joins the room graph — and the extras are the other links.

use crate::domain::geometry::Segment;
use crate::domain::model::Room;
use crate::domain::skeleton::{Skeleton, adjacency};
use crate::use_cases::bsp::Split;

/// Builds the skeleton for a floor: all adjacencies, backbone
/// where a link crosses a split or a join, entry as given.
pub fn build_skeleton(
    rooms: &[Room],
    splits: &[Split],
    joins: &[Segment],
    entry: u32,
) -> Skeleton {
    let links = adjacency(rooms)
        .into_iter()
        .map(|mut l| {
            l.backbone = splits.iter().any(|s| l.segment.overlap(&s.line).is_some())
                || joins.iter().any(|j| l.segment.overlap(j).is_some());
            l
        })
        .collect();
    Skeleton { entry, links }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry::Rect;
    use crate::domain::model::Program;
    use crate::use_cases::bsp::subdivide;
    use crate::frameworks_drivers::rng::Rng;

    fn room(id: u32, program: Program, rect: Rect) -> Room {
        Room { id, program, rect }
    }

    #[test]
    fn a_split_produces_one_backbone_link() {
        let mut rng = Rng::new(4);
        let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 10.0, 6.0), 2);
        let rooms = vec![
            room(0, Program::Living, sub.leaves[0]),
            room(1, Program::Kitchen, sub.leaves[1]),
        ];
        let skeleton = build_skeleton(&rooms, &sub.splits, &[], 0);
        assert_eq!(skeleton.links.len(), 1, "two rooms share one wall");
        assert!(skeleton.links[0].backbone, "a split link is backbone");
        assert_eq!(skeleton.entry, 0);
    }

    #[test]
    fn every_split_has_a_backbone_link() {
        for seed in 0..30u32 {
            let mut rng = Rng::new(seed);
            let sub = subdivide(&mut rng, Rect::new(0.0, 0.0, 12.0, 9.0), 6);
            let programs = crate::use_cases::programs::assign_programs(&sub.leaves, 0, false);
            let rooms: Vec<Room> = sub
                .leaves
                .iter()
                .enumerate()
                .map(|(i, r)| room(i as u32, programs[i], *r))
                .collect();
            let skeleton = build_skeleton(&rooms, &sub.splits, &[], 0);
            for split in &sub.splits {
                assert!(
                    skeleton.links.iter().any(|l| l.backbone && l.segment.overlap(&split.line).is_some()),
                    "seed {seed}: split at {} has no backbone link",
                    split.at
                );
            }
            let ids: Vec<u32> = rooms.iter().map(|r| r.id).collect();
            assert!(skeleton.is_connected(&ids), "seed {seed}: skeleton is disconnected");
            assert_eq!(skeleton.spine().len(), rooms.len() - 1, "seed {seed}: spine is not a tree");
        }
    }

    #[test]
    fn a_join_produces_a_backbone_link() {
        let rooms = vec![
            room(0, Program::Living, Rect::new(0.0, 0.0, 6.0, 5.0)),
            room(1, Program::Kitchen, Rect::new(6.0, 0.0, 6.0, 5.0)),
        ];
        let joins = vec![Segment::new(6.0, 0.0, 6.0, 5.0)];
        let skeleton = build_skeleton(&rooms, &[], &joins, 0);
        assert_eq!(skeleton.links.len(), 1, "two rooms share one wall");
        assert!(skeleton.links[0].backbone, "a join link is backbone");
    }
}
