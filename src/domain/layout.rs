//! Residential lot and house density for a neighborhood.
//!
//! The old layout families (Grid, Offset, CulDeSac) are retired:
//! streets are splines now (ADR 0010), so there is no family to
//! pick. Density still tiers how many lots a street carries.

/// Residential lot and house density for a neighborhood.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Density {
    Low,
    Medium,
    High,
}

impl Density {
    /// All density tiers, from suburban to urban row.
    pub fn all() -> [Density; 3] {
        [Density::Low, Density::Medium, Density::High]
    }

    /// Deterministic tier from a seed.
    pub fn from_seed(seed: u32) -> Density {
        match (seed / 3) % 3 {
            0 => Density::Low,
            1 => Density::Medium,
            _ => Density::High,
        }
    }

    /// Tier from a 0..3 index.
    pub fn from_index(index: u32) -> Option<Density> {
        Self::all().get(index as usize).copied()
    }

    /// Machine identifier.
    pub fn id(&self) -> &'static str {
        match self {
            Density::Low => "low",
            Density::Medium => "medium",
            Density::High => "high",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_seed_is_deterministic_and_covers_all_tiers() {
        assert_eq!(Density::from_seed(42), Density::from_seed(42));
        let seen: Vec<Density> = (0..30u32).map(Density::from_seed).collect();
        for d in Density::all() {
            assert!(seen.contains(&d), "{d:?} never appears");
        }
    }

    #[test]
    fn the_density_ids_are_distinct() {
        let ids: Vec<&str> = Density::all().iter().map(|d| d.id()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2]);
    }
}
