//! Dungeon feature types, ported from dungeon-feature-type.h.
//! Replaces the simplified Feature enum with the full DCSS feature set.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DungeonFeatureType {
    Unseen = 0,
    // Doors
    ClosedDoor,
    BrokenDoor,
    ClosedClearDoor,
    BrokenClearDoor,
    RunedDoor,
    RunedClearDoor,
    SealedDoor,
    SealedClearDoor,
    // Trees
    Tree,
    Mangrove,
    DemonicTree,
    PetrifiedTree,
    // Walls
    MetalWall,
    CrystalWall,
    RockWall,
    SlimyWall,
    StoneWall,
    PermarockWall,
    ClearRockWall,
    ClearStoneWall,
    ClearPermarockWall,
    FrigidWall,
    // Barriers
    Grate,
    OpenSea,
    LavaSea,
    EndlessSalt,
    // Statues
    OrcishIdol,
    MetalStatue,
    ZotStatue,
    GraniteStatue,
    MalignGateway,
    // Liquids
    Lava,
    DeepWater,
    ShallowWater,
    Mud,
    ToxicBog,
    BindingSigil,
    // Floor
    Floor,
    OpenDoor,
    OpenClearDoor,
    // Traps
    TrapTeleport,
    TrapTeleportPermanent,
    TrapShaft,
    TrapWeb,
    TrapTyrant,
    TrapArchmage,
    TrapHarlequin,
    TrapDevourer,
    TrapAlarm,
    TrapZot,
    TrapDispersal,
    TrapDispersalInactive,
    PassageOfGolubria,
    TrapNet,
    TrapPlate,
    SpikeLauncher,
    OrbDais,
    // Shops
    EnterShop,
    AbandonedShop,
    // Stairs
    StoneStairsDownI,
    StoneStairsDownII,
    StoneStairsDownIII,
    EscapeHatchDown,
    StoneStairsUpI,
    StoneStairsUpII,
    StoneStairsUpIII,
    EscapeHatchUp,
    // Transporters
    Transporter,
    TransporterLanding,
    // Branch entrances/exits (hell)
    EnterDis,
    EnterGehenna,
    EnterCocytus,
    EnterTartarus,
    ExitDis,
    ExitGehenna,
    ExitCocytus,
    ExitTartarus,
    // Abyss/Pandemonium
    EnterAbyss,
    ExitAbyss,
    StoneArch,
    EnterPandemonium,
    ExitPandemonium,
    TransitPandemonium,
    // Dungeon exits
    ExitDungeon,
    ExitThroughAbyss,
    ExitHell,
    EnterHell,
    ExpiredPortal,
    // Branch entrances
    EnterOrc,
    EnterLair,
    EnterSlime,
    EnterVaults,
    EnterCrypt,
    EnterZot,
    EnterTemple,
    EnterSnake,
    EnterElf,
    EnterTomb,
    EnterSwamp,
    EnterShoals,
    EnterSpider,
    EnterDepths,
    // Branch exits
    ExitOrc,
    ExitLair,
    ExitSlime,
    ExitVaults,
    ExitCrypt,
    ExitZot,
    ExitTemple,
    ExitSnake,
    ExitElf,
    ExitTomb,
    ExitSwamp,
    ExitShoals,
    ExitSpider,
    ExitDepths,
    // Altars
    AltarZin,
    AltarShiningOne,
    AltarKikubaaqudgha,
    AltarYredelemnul,
    AltarXom,
    AltarVehumet,
    AltarOkawaru,
    AltarMakhleb,
    AltarSifMuna,
    AltarTrog,
    AltarNemelexxobeh,
    AltarElyvilon,
    AltarLugonu,
    AltarBeogh,
    AltarJiyva,
    AltarFedhas,
    AltarCheibriados,
    AltarAshenzari,
    AltarDithmenos,
    AltarGozag,
    AltarQazlal,
    AltarRu,
    AltarUskayaw,
    AltarHepliaklqana,
    AltarWuJian,
    AltarIgnis,
    AltarEcumenical,
    // Fountains
    FountainBlue,
    FountainSparkling,
    FountainBlood,
    FountainEyes,
    DryFountain,
    // Caches
    CacheOfBakedGoods,
    CacheOfFruit,
    CacheOfMeat,
    // Decorative
    DecorativeFloor,
    Runelight,
    // Meta (not rendered)
    ExploreHorizon,
    TravelTrail,
    UnknownAltar,
    UnknownPortal,
    AbyssalStair,
    SealedStairsUp,
    SealedStairsDown,
    // Portal vaults
    EnterZiggurat,
    EnterBazaar,
    EnterTrove,
    EnterNecropolis,
    EnterSewer,
    EnterOssuary,
    EnterBailey,
    EnterGauntlet,
    EnterIceCave,
    EnterVolcano,
    EnterWizlab,
    EnterDesolation,
    EnterArena,
    EnterCrucible,
    ExitZiggurat,
    ExitBazaar,
    ExitTrove,
    ExitNecropolis,
    ExitSewer,
    ExitOssuary,
    ExitBailey,
    ExitGauntlet,
    ExitIceCave,
    ExitVolcano,
    ExitWizlab,
    ExitDesolation,
    ExitArena,
    ExitCrucible,
}

impl DungeonFeatureType {
    pub fn is_wall(self) -> bool {
        use DungeonFeatureType::*;
        matches!(self, MetalWall | CrystalWall | RockWall | SlimyWall | StoneWall
            | PermarockWall | ClearRockWall | ClearStoneWall | ClearPermarockWall
            | FrigidWall | Tree | Mangrove | DemonicTree | PetrifiedTree
            | OrcishIdol | MetalStatue | ZotStatue | GraniteStatue | Grate
            | OpenSea | LavaSea | EndlessSalt)
    }

    pub fn is_passable(self) -> bool {
        use DungeonFeatureType::*;
        matches!(self, Floor | OpenDoor | OpenClearDoor | ShallowWater | Mud
            | StoneStairsDownI | StoneStairsDownII | StoneStairsDownIII | EscapeHatchDown
            | StoneStairsUpI | StoneStairsUpII | StoneStairsUpIII | EscapeHatchUp
            | Transporter | TransporterLanding | EnterShop | TrapTeleport
            | TrapShaft | TrapWeb | TrapAlarm | TrapNet | TrapPlate | SpikeLauncher
            | PassageOfGolubria | DecorativeFloor | BindingSigil)
    }

    pub fn is_door(self) -> bool {
        use DungeonFeatureType::*;
        matches!(self, ClosedDoor | BrokenDoor | ClosedClearDoor | BrokenClearDoor
            | RunedDoor | RunedClearDoor | SealedDoor | SealedClearDoor
            | OpenDoor | OpenClearDoor)
    }

    pub fn is_stairs_down(self) -> bool {
        use DungeonFeatureType::*;
        matches!(self, StoneStairsDownI | StoneStairsDownII | StoneStairsDownIII | EscapeHatchDown)
    }
}
