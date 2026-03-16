use rand::Rng;
use crate::item::Brand;

pub struct MeleeResult {
    pub hit: bool,
    pub damage: i32,
    pub brand_damage: i32,
    pub brand: Option<Brand>,
    pub total_damage: i32,
}

/// Melee combat: roll to hit, then roll damage minus AC, then apply brand.
pub fn resolve_melee(
    attacker_damage: i32,
    attacker_accuracy: i32,
    defender_ac: i32,
    defender_ev: i32,
    brand: Option<Brand>,
) -> MeleeResult {
    let mut rng = rand::rng();
    let attack_roll = rng.random_range(0..=attacker_accuracy);
    let ev_roll = rng.random_range(0..=defender_ev);

    if attack_roll < ev_roll {
        return MeleeResult { hit: false, damage: 0, brand_damage: 0, brand: None, total_damage: 0 };
    }

    let raw_damage = if attacker_damage > 0 { rng.random_range(1..=attacker_damage) } else { 0 };
    let ac_reduction = if defender_ac > 0 { rng.random_range(0..=defender_ac) } else { 0 };
    let base_damage = (raw_damage - ac_reduction).max(0);

    let brand_damage = match brand {
        Some(Brand::Flaming) | Some(Brand::Freezing) => rng.random_range(2..=4),
        Some(Brand::Draining) => 0, // healing, not extra damage
        Some(Brand::Venom) => 0,    // poison, not extra damage
        None => 0,
    };

    let total = base_damage + brand_damage;

    MeleeResult {
        hit: true,
        damage: base_damage,
        brand_damage,
        brand,
        total_damage: total,
    }
}
