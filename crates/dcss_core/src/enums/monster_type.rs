//! Monster type enum, ported from monster-type.h.
//! Subset of the most common early-game monsters + structural markers.
//! The full 1,075-variant enum can be generated later.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterType {
    ProgramBug,
    // Animals
    Rat,
    Bat,
    FireBat,
    Jackal,
    Hound,
    Wolf,
    Adder,
    // Humanoids
    Goblin,
    Hobgoblin,
    Gnoll,
    Kobold,
    BigKobold,
    Orc,
    OrcWizard,
    OrcPriest,
    OrcWarrior,
    OrcKnight,
    // Undead
    Skeleton,
    Zombie,
    Wight,
    Phantom,
    // Misc early game
    GiantCockroach,
    Scorpion,
    Quokka,
    FrilledLizard,
    Iguana,
    // Tougher
    Ogre,
    Troll,
    Centaur,
    Yak,
    // Boss-tier
    SigmundUnique,
    // Structural
    NoMonster,
}

impl MonsterType {
    /// Map a monster name (from YAML) to a MonsterType.
    pub fn from_name(name: &str) -> Option<MonsterType> {
        match name {
            "rat" => Some(MonsterType::Rat),
            "bat" => Some(MonsterType::Bat),
            "fire bat" => Some(MonsterType::FireBat),
            "jackal" => Some(MonsterType::Jackal),
            "hound" => Some(MonsterType::Hound),
            "wolf" => Some(MonsterType::Wolf),
            "adder" => Some(MonsterType::Adder),
            "goblin" => Some(MonsterType::Goblin),
            "hobgoblin" => Some(MonsterType::Hobgoblin),
            "gnoll" => Some(MonsterType::Gnoll),
            "kobold" => Some(MonsterType::Kobold),
            "big kobold" => Some(MonsterType::BigKobold),
            "orc" => Some(MonsterType::Orc),
            "orc wizard" => Some(MonsterType::OrcWizard),
            "orc priest" => Some(MonsterType::OrcPriest),
            "orc warrior" => Some(MonsterType::OrcWarrior),
            "orc knight" => Some(MonsterType::OrcKnight),
            "skeleton" => Some(MonsterType::Skeleton),
            "zombie" => Some(MonsterType::Zombie),
            "wight" => Some(MonsterType::Wight),
            "phantom" => Some(MonsterType::Phantom),
            "giant cockroach" => Some(MonsterType::GiantCockroach),
            "scorpion" => Some(MonsterType::Scorpion),
            "quokka" => Some(MonsterType::Quokka),
            "frilled lizard" => Some(MonsterType::FrilledLizard),
            "iguana" => Some(MonsterType::Iguana),
            "ogre" => Some(MonsterType::Ogre),
            "troll" => Some(MonsterType::Troll),
            "centaur" => Some(MonsterType::Centaur),
            "yak" => Some(MonsterType::Yak),
            _ => None,
        }
    }
}
