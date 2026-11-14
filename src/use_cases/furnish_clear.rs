//! Clearance checks for furniture placement: a candidate rect
//! must sit inside its room, overlap no placed piece, and block
//! no door swing and no window. Split out of `furnish` so both
//! files stay under the 200-line ceiling.

use crate::domain::furniture::{Furniture, FurnitureKind};
use crate::domain::geometry::{Axis, Rect};
use crate::domain::model::{Opening, Room};

/// A candidate rect is valid when it sits inside the room, overlaps
/// no placed piece, blocks no door swing and no window. Only tall
/// pieces can block a window; low pieces sit below the sill.
pub(crate) fn fits(
    room: &Room,
    kind: FurnitureKind,
    rect: &Rect,
    doors: &[&Opening],
    windows: &[&Opening],
    placed: &[Furniture],
) -> bool {
    if rect.x < room.rect.x - 1e-9
        || rect.y < room.rect.y - 1e-9
        || rect.right() > room.rect.right() + 1e-9
        || rect.bottom() > room.rect.bottom() + 1e-9
    {
        return false;
    }
    if placed.iter().any(|p| p.rect.overlaps(rect)) {
        return false;
    }
    if doors.iter().any(|o| blocks_door(room, rect, o)) {
        return false;
    }
    if kind.is_tall() && windows.iter().any(|o| blocks_window(rect, o)) {
        return false;
    }
    true
}

/// A piece blocks a door when it stands inside the door's swing:
/// a rectangle the door's width along the wall (plus a 0.1 m
/// margin) and the door's width into the room — on the room's
/// side of the wall only. A door swings into one room, not both.
pub fn blocks_door(room: &Room, rect: &Rect, o: &Opening) -> bool {
    rect.overlaps(&door_zone(room, o))
}

/// The door's swing zone on the room's side of the wall.
pub fn door_zone(room: &Room, o: &Opening) -> Rect {
    let w = o.width;
    let m = 0.1;
    match o.axis {
        Axis::Horizontal => {
            let x = o.x - w / 2.0 - m;
            let span = w + 2.0 * m;
            if (o.y - room.rect.y).abs() < 1e-9 {
                Rect::new(x, o.y, span, w)
            } else {
                Rect::new(x, o.y - w, span, w)
            }
        }
        Axis::Vertical => {
            let y = o.y - w / 2.0 - m;
            let span = w + 2.0 * m;
            if (o.x - room.rect.x).abs() < 1e-9 {
                Rect::new(o.x, y, w, span)
            } else {
                Rect::new(o.x - w, y, w, span)
            }
        }
    }
}

/// A piece blocks a window when it stands against the window's
/// wall and its span covers the window's span.
pub fn blocks_window(rect: &Rect, o: &Opening) -> bool {
    match o.axis {
        Axis::Horizontal => {
            let on_wall = (rect.y - o.y).abs() < 1e-9 || (rect.bottom() - o.y).abs() < 1e-9;
            on_wall && rect.x < o.x + o.width / 2.0 && rect.right() > o.x - o.width / 2.0
        }
        Axis::Vertical => {
            let on_wall = (rect.x - o.x).abs() < 1e-9 || (rect.right() - o.x).abs() < 1e-9;
            on_wall && rect.y < o.y + o.width / 2.0 && rect.bottom() > o.y - o.width / 2.0
        }
    }
}

/// True when a person can walk from every door of the room into its
/// main open area. A 0.2 m grid with a 0.15 m body clearance around
/// each piece; the target is the free space's centroid, not the
/// geometric centre (a bed can legitimately cover the centre).
pub fn walkable(room: &Room, doors: &[&Opening], placed: &[Furniture]) -> bool {
    if doors.is_empty() {
        return true;
    }
    let cell = 0.2;
    let nx = ((room.rect.w / cell).ceil() as usize).max(1);
    let nz = ((room.rect.d / cell).ceil() as usize).max(1);
    let mut blocked = vec![false; nx * nz];
    for j in 0..nz {
        for i in 0..nx {
            let x = room.rect.x + (i as f64 + 0.5) * cell;
            let z = room.rect.y + (j as f64 + 0.5) * cell;
            blocked[j * nx + i] = placed.iter().any(|p| {
                x >= p.rect.x - 0.15
                    && x <= p.rect.right() + 0.15
                    && z >= p.rect.y - 0.15
                    && z <= p.rect.bottom() + 0.15
            });
        }
    }
    let free: Vec<(usize, usize)> = (0..nz)
        .flat_map(|j| (0..nx).map(move |i| (i, j)))
        .filter(|&(i, j)| !blocked[j * nx + i])
        .collect();
    if free.is_empty() {
        return false;
    }
    let (sum_i, sum_j) = free.iter().fold((0usize, 0usize), |(si, sj), &(i, j)| (si + i, sj + j));
    let (ci, cj) = (sum_i as f64 / free.len() as f64, sum_j as f64 / free.len() as f64);
    let target = free
        .iter()
        .min_by(|&&(i1, j1), &&(i2, j2)| {
            let d1 = (i1 as f64 - ci).powi(2) + (j1 as f64 - cj).powi(2);
            let d2 = (i2 as f64 - ci).powi(2) + (j2 as f64 - cj).powi(2);
            d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
        .unwrap();
    doors.iter().all(|o| {
        let (dx, dz) = door_inner_point(room, o);
        let si = ((dx - room.rect.x) / cell).floor() as usize;
        let sj = ((dz - room.rect.y) / cell).floor() as usize;
        if si >= nx || sj >= nz {
            return true;
        }
        if blocked[sj * nx + si] {
            return false;
        }
        reachable(&blocked, nx, nz, si, sj, target.0, target.1)
    })
}

/// The door's centre, just inside the room.
fn door_inner_point(room: &Room, o: &Opening) -> (f64, f64) {
    match o.axis {
        Axis::Horizontal => {
            if (o.y - room.rect.y).abs() < 1e-9 {
                (o.x, o.y + 0.3)
            } else {
                (o.x, o.y - 0.3)
            }
        }
        Axis::Vertical => {
            if (o.x - room.rect.x).abs() < 1e-9 {
                (o.x + 0.3, o.y)
            } else {
                (o.x - 0.3, o.y)
            }
        }
    }
}

/// Breadth-first reachability on the grid.
fn reachable(blocked: &[bool], nx: usize, nz: usize, si: usize, sj: usize, ti: usize, tj: usize) -> bool {
    if blocked[sj * nx + si] {
        return false;
    }
    let mut seen = vec![false; nx * nz];
    let mut stack = vec![(si, sj)];
    seen[sj * nx + si] = true;
    while let Some((i, j)) = stack.pop() {
        if i == ti && j == tj {
            return true;
        }
        for (di, dj) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let ni = i as i32 + di;
            let nj = j as i32 + dj;
            if ni < 0 || nj < 0 || ni >= nx as i32 || nj >= nz as i32 {
                continue;
            }
            let (ni, nj) = (ni as usize, nj as usize);
            if !seen[nj * nx + ni] && !blocked[nj * nx + ni] {
                seen[nj * nx + ni] = true;
                stack.push((ni, nj));
            }
        }
    }
    false
}
