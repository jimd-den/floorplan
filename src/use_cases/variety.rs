//! Plan hashing: an FNV-1a hash over the seed-free plan JSON, used
//! by the variety test to count distinct houses. The seed is
//! excluded on purpose: hashing it would make every hash distinct
//! by construction and prove nothing about the plans.

use crate::adapters::json::render_plan_json;
use crate::adapters::neighborhood_json::render_neighborhood_plan_json;
use crate::domain::model::House;
use crate::domain::neighborhood::Neighborhood;

/// FNV-1a over a string.
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Hashes a plan. Same plan, same hash; the seed plays no part.
pub fn plan_hash(house: &House) -> u64 {
    fnv1a(&render_plan_json(house))
}

/// Hashes a neighborhood (seed-free). Same neighborhood, same hash.
pub fn neighborhood_hash(n: &Neighborhood) -> u64 {
    fnv1a(&render_neighborhood_plan_json(n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::style::Style;
    use crate::use_cases::generate::generate_house;

    #[test]
    fn same_house_gives_the_same_hash() {
        let house = generate_house(1, Style::Ranch);
        assert_eq!(plan_hash(&house), plan_hash(&house));
    }

    #[test]
    fn different_houses_give_different_hashes() {
        assert_ne!(
            plan_hash(&generate_house(1, Style::Ranch)),
            plan_hash(&generate_house(2, Style::Ranch))
        );
        assert_ne!(
            plan_hash(&generate_house(1, Style::Ranch)),
            plan_hash(&generate_house(1, Style::Modern))
        );
    }

    #[test]
    fn the_hash_is_never_zero_for_real_input() {
        assert_ne!(plan_hash(&generate_house(42, Style::Ranch)), 0);
    }

    #[test]
    fn same_neighborhood_gives_the_same_hash() {
        let n = crate::use_cases::neighborhood::generate_neighborhood(1, (2, 2));
        assert_eq!(neighborhood_hash(&n), neighborhood_hash(&n));
    }

    #[test]
    fn different_neighborhoods_give_different_hashes() {
        let a = crate::use_cases::neighborhood::generate_neighborhood(1, (2, 2));
        let b = crate::use_cases::neighborhood::generate_neighborhood(2, (2, 2));
        assert_ne!(neighborhood_hash(&a), neighborhood_hash(&b));
    }
}
