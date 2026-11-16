//! Skeleton: the logical structure of one floor — the shared-wall
//! links between its rooms and the entry room. Walls are the
//! skeleton rooms' edges; doors sit on the skeleton's links. The
//! skeleton proves the floor is connected: every room reaches the
//! entry through links, so a house built from it is walkable.

use crate::domain::geometry::{Segment, shared_edge};
use crate::domain::model::Room;

/// One shared wall between two rooms: the rooms it joins, the
/// segment they share, and whether it is backbone — a link across a
/// BSP split or a footprint join, the links the room graph is
/// connected by. Non-backbone links are extra adjacencies.
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonLink {
    pub a: u32,
    pub b: u32,
    pub segment: Segment,
    pub backbone: bool,
}

/// The logical skeleton of one floor: its entry room and every
/// shared-wall link between its rooms.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skeleton {
    pub entry: u32,
    pub links: Vec<SkeletonLink>,
}

impl Skeleton {
    /// True when every room in `rooms` reaches the entry through
    /// links — the floor's circulation graph is one piece.
    pub fn is_connected(&self, rooms: &[u32]) -> bool {
        rooms.iter().all(|r| self.reaches(*r))
    }

    /// True when `room` reaches the entry through links.
    fn reaches(&self, room: u32) -> bool {
        if room == self.entry {
            return true;
        }
        let mut seen = vec![self.entry];
        let mut stack = vec![self.entry];
        while let Some(cur) = stack.pop() {
            for l in &self.links {
                let next = if l.a == cur {
                    Some(l.b)
                } else if l.b == cur {
                    Some(l.a)
                } else {
                    None
                };
                if let Some(n) = next
                    && !seen.contains(&n)
                {
                    if n == room {
                        return true;
                    }
                    seen.push(n);
                    stack.push(n);
                }
            }
        }
        false
    }

    /// The circulation spine: the BFS tree from the entry over
    /// backbone links, the links a door must cross. Every room the
    /// spine reaches is walkable from the entry.
    pub fn spine(&self) -> Vec<SkeletonLink> {
        let mut spine = vec![];
        let mut seen = vec![self.entry];
        let mut stack = vec![self.entry];
        while let Some(cur) = stack.pop() {
            for l in self.links.iter().filter(|l| l.backbone) {
                let next = if l.a == cur {
                    Some(l.b)
                } else if l.b == cur {
                    Some(l.a)
                } else {
                    None
                };
                if let Some(n) = next
                    && !seen.contains(&n)
                {
                    seen.push(n);
                    stack.push(n);
                    spine.push(l.clone());
                }
            }
        }
        spine
    }

    /// The links that carry a door by construction: the backbone.
    pub fn backbone_links(&self) -> Vec<SkeletonLink> {
        self.links.iter().filter(|l| l.backbone).cloned().collect()
    }
}

/// All shared walls between rooms, as unclassified links
/// (`backbone` false — classification is the builder's job).
/// Two rooms that share an edge are adjacent; a sliver of shared
/// wall counts, since the door step decides what can hold one.
pub fn adjacency(rooms: &[Room]) -> Vec<SkeletonLink> {
    let mut out = vec![];
    for (i, a) in rooms.iter().enumerate() {
        for b in rooms.iter().skip(i + 1) {
            if let Some(segment) = shared_edge(&a.rect, &b.rect) {
                out.push(SkeletonLink { a: a.id, b: b.id, segment, backbone: false });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(a: u32, b: u32, backbone: bool) -> SkeletonLink {
        SkeletonLink { a, b, segment: Segment::new(0.0, 0.0, 4.0, 0.0), backbone }
    }

    fn chain() -> Skeleton {
        Skeleton {
            entry: 0,
            links: vec![link(0, 1, true), link(1, 2, true), link(0, 2, false)],
        }
    }

    #[test]
    fn a_linked_chain_is_connected() {
        assert!(chain().is_connected(&[0, 1, 2]), "0-1-2 must be one piece");
    }

    #[test]
    fn an_isolated_room_breaks_the_connection() {
        let s = Skeleton { entry: 0, links: vec![link(0, 1, true)] };
        assert!(!s.is_connected(&[0, 1, 2]), "room 2 has no link");
    }

    #[test]
    fn the_spine_reaches_every_room_over_backbone_links() {
        let spine = chain().spine();
        assert_eq!(spine.len(), 2, "a three-room spine has two links");
        assert!(spine.iter().all(|l| l.backbone), "the spine is backbone only");
        let reached: Vec<u32> = spine.iter().flat_map(|l| [l.a, l.b]).collect();
        assert!(reached.contains(&1) && reached.contains(&2), "spine misses a room");
    }

    #[test]
    fn backbone_links_exclude_extras() {
        let back = chain().backbone_links();
        assert_eq!(back.len(), 2, "two of three links are backbone");
        assert!(back.iter().all(|l| l.backbone));
    }

    #[test]
    fn adjacency_finds_every_shared_wall_unclassified() {
        use crate::domain::geometry::Rect;
        use crate::domain::model::Program;
        let rooms = vec![
            Room { id: 0, program: Program::Living, rect: Rect::new(0.0, 0.0, 4.0, 4.0) },
            Room { id: 1, program: Program::Kitchen, rect: Rect::new(4.0, 0.0, 4.0, 4.0) },
            Room { id: 2, program: Program::Bedroom, rect: Rect::new(0.0, 6.0, 4.0, 4.0) },
        ];
        let links = adjacency(&rooms);
        assert_eq!(links.len(), 1, "only the first two rooms share a wall");
        assert_eq!((links[0].a, links[0].b), (0, 1));
        assert!(!links[0].backbone, "an adjacency starts unclassified");
    }
}
