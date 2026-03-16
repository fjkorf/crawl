use rand::Rng;

pub struct MeleeResult {
    pub hit: bool,
    pub damage: i32,
}

/// Simplified melee: roll to hit, then roll damage minus AC.
pub fn resolve_melee(
    attacker_damage: i32,
    attacker_accuracy: i32,
    defender_ac: i32,
    defender_ev: i32,
) -> MeleeResult {
    let mut rng = rand::rng();
    let attack_roll = rng.random_range(0..=attacker_accuracy);
    let ev_roll = rng.random_range(0..=defender_ev);

    if attack_roll < ev_roll {
        return MeleeResult {
            hit: false,
            damage: 0,
        };
    }

    let raw_damage = if attacker_damage > 0 {
        rng.random_range(1..=attacker_damage)
    } else {
        0
    };
    let ac_reduction = if defender_ac > 0 {
        rng.random_range(0..=defender_ac)
    } else {
        0
    };
    let final_damage = (raw_damage - ac_reduction).max(0);

    MeleeResult {
        hit: true,
        damage: final_damage,
    }
}
