//! A neighborhood's layout family: the street/block pattern.
//!
//! The family is chosen deterministically from the seed, so the
//! same seed always builds the same pattern. Grid is the original
//! uniform street grid; Offset staggers alternate block rows by
//! half a block; CulDeSac turns one or two blocks of an otherwise
//! regular grid into dead-end courts.

/// A street/block pattern family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    Grid,
    Offset,
    CulDeSac,
}

impl Layout {
    /// All families, in selection order.
    pub fn all() -> [Layout; 3] {
        [Layout::Grid, Layout::Offset, Layout::CulDeSac]
    }

    /// The family for a seed, deterministically. Every third seed
    /// is the same family, so each family covers a third of the
    /// seed space.
    pub fn from_seed(seed: u32) -> Layout {
        match seed % 3 {
            0 => Layout::Grid,
            1 => Layout::Offset,
            _ => Layout::CulDeSac,
        }
    }

    /// The machine id, used in JSON.
    pub fn id(&self) -> &'static str {
        match self {
            Layout::Grid => "grid",
            Layout::Offset => "offset",
            Layout::CulDeSac => "cul-de-sac",
        }
    }

    /// The human name.
    pub fn name(&self) -> &'static str {
        match self {
            Layout::Grid => "Grid",
            Layout::Offset => "Offset",
            Layout::CulDeSac => "Cul-de-sac",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_seed_is_deterministic() {
        assert_eq!(Layout::from_seed(42), Layout::from_seed(42));
    }

    #[test]
    fn every_family_appears_over_consecutive_seeds() {
        let seen: Vec<Layout> = (0..30u32).map(Layout::from_seed).collect();
        for layout in Layout::all() {
            assert!(seen.contains(&layout), "{layout:?} never appears");
        }
    }

    #[test]
    fn the_family_ids_are_distinct() {
        let ids: Vec<&str> = Layout::all().iter().map(|l| l.id()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);
    }
}
