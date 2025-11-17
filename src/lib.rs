// Factorio achievements editor
// Copyright (C) 2025  Emil Lundberg
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod data_types;

use std::collections::BTreeSet;
use std::fmt::Debug;

use binrw::binrw;
use binrw::helpers::until_eof;
use data_types::SizedVec;
use data_types::SpaceOptimizedString;

#[binrw]
#[derive(Debug)]
pub struct AchievementsDat {
    version: [u16; 4],
    unused: [u8; 1],
    headers: SizedVec<u16, AchievementHeader>,
    contents: SizedVec<u32, AchievementContent>,
    #[br(parse_with = until_eof)]
    tracked: Vec<u16>,
}

impl AchievementsDat {
    pub fn delete(mut self, id: &str) -> Self {
        self.contents
            .iter_mut()
            .filter(|content| content.id.as_ref() == id)
            .for_each(|content| {
                content.progress.reset();
            });
        self
    }

    pub fn list(&self) -> BTreeSet<&SpaceOptimizedString> {
        self.contents.iter().map(|item| &item.id).collect()
    }
}

#[binrw]
#[derive(Debug)]
pub struct AchievementHeader {
    typ: SpaceOptimizedString,
    subobjects: SizedVec<u16, HeaderSubobject>,
}

#[binrw]
#[derive(Debug)]
pub struct HeaderSubobject {
    id: SpaceOptimizedString,
    index: u16,
}

#[binrw]
#[derive(Debug)]
pub struct AchievementContent {
    typ: SpaceOptimizedString,
    id: SpaceOptimizedString,
    #[br(args(typ.as_bytes()))]
    progress: AchievementProgress,
}

#[binrw]
#[derive(Debug)]
#[br(import(typ: &[u8]))]
pub enum AchievementProgress {
    #[br(pre_assert(typ == b"achievement"))]
    Achievement,
    #[br(pre_assert(typ == b"build-entity-achievement"))]
    BuildEntity([u8; 4]),
    #[br(pre_assert(typ == b"change-surface-achievement"))]
    ChangeSurface([u8; 1]),
    #[br(pre_assert(typ == b"combat-robot-count-achievement"))]
    CombatRobotCount(u32),
    #[br(pre_assert(typ == b"complete-objective-achievement"))]
    CompleteObjective,
    #[br(pre_assert(typ == b"construct-with-robots-achievement"))]
    ConstructWithRobots { constructed: u32, unknown: [u8; 4] },
    #[br(pre_assert(typ == b"create-platform-achievement"))]
    CreatePlatform([u8; 4]),
    #[br(pre_assert(typ == b"deconstruct-with-robots-achievement"))]
    DeconstructWithRobots { deconstructed: u32 },
    #[br(pre_assert(typ == b"deliver-by-robots-achievement"))]
    DeliverByRobots([u8; 4]),
    #[br(pre_assert(typ == b"deplete-resource-achievement"))]
    DepleteResource([u8; 4]),
    #[br(pre_assert(typ == b"destroy-cliff-achievement"))]
    DestroyCliff([u8; 4]),
    #[br(pre_assert(typ == b"dont-build-entity-achievement"))]
    DontBuildEntity([u8; 5]),
    #[br(pre_assert(typ == b"dont-craft-manually-achievement"))]
    DontCraftManually([u8; 4]),
    /// Unknown format
    #[br(pre_assert(typ == b"dont-kill-manually-achievement"))]
    DontKillManually([u8; 0]),
    /// Unknown format
    #[br(pre_assert(typ == b"dont-research-before-researching-achievement"))]
    DontResearchBeforeResearching([u8; 0]),
    #[br(pre_assert(typ == b"dont-use-entity-in-energy-production-achievement"))]
    DontUseEntityInEnergyProduction { max_j_per_h: f64 },
    #[br(pre_assert(typ == b"equip-armor-achievement"))]
    EquipArmor([u8; 4]),
    #[br(pre_assert(typ == b"finish-the-game-achievement"))]
    FinishTheGame([u8; 4]),
    #[br(pre_assert(typ == b"group-attack-achievement"))]
    GroupAttack([u8; 4]),
    #[br(pre_assert(typ == b"kill-achievement"))]
    Kill { max_killed: f64 },
    #[br(pre_assert(typ == b"module-transfer-achievement"))]
    ModuleTransfer([u8; 4]),
    #[br(pre_assert(typ == b"place-equipment-achievement"))]
    PlaceEquipment([u8; 4]),
    #[br(pre_assert(typ == b"player-damaged-achievement"))]
    PlayerDamaged { max_damage: f32, survived: u8 },
    #[br(pre_assert(typ == b"produce-achievement"))]
    Produce { produced: f64 },
    #[br(pre_assert(typ == b"produce-per-hour-achievement"))]
    ProducePerHour { max_per_h: f64 },
    #[br(pre_assert(typ == b"research-achievement"))]
    Research,
    #[br(pre_assert(typ == b"research-with-science-pack-achievement"))]
    ResearchWithSciencePack([u8; 4]),
    #[br(pre_assert(typ == b"shoot-achievement"))]
    Shoot([u8; 4]),
    #[br(pre_assert(typ == b"space-connection-distance-traveled-achievement"))]
    SpaceConnectionDistanceTraveled([u8; 4]),
    #[br(pre_assert(typ == b"train-path-achievement"))]
    TrainPath { longest_path: f64 },
    #[br(pre_assert(typ == b"use-entity-in-energy-production-achievement"))]
    UseEntityInEnergyProduction([u8; 5]),
    #[br(pre_assert(typ == b"use-item-achievement"))]
    UseItem([u8; 4]),
}

impl AchievementProgress {
    fn reset(&mut self) {
        use AchievementProgress::*;
        *self = match self {
            Achievement => Achievement,
            BuildEntity(..) => BuildEntity(Default::default()),
            ChangeSurface(..) => ChangeSurface(Default::default()),
            CombatRobotCount(..) => CombatRobotCount(Default::default()),
            CompleteObjective => CompleteObjective,
            ConstructWithRobots { .. } => ConstructWithRobots {
                constructed: Default::default(),
                unknown: Default::default(),
            },
            CreatePlatform(..) => CreatePlatform(Default::default()),
            DeconstructWithRobots { .. } => DeconstructWithRobots {
                deconstructed: Default::default(),
            },
            DeliverByRobots(..) => DeliverByRobots(Default::default()),
            DepleteResource(..) => DepleteResource(Default::default()),
            DestroyCliff(..) => DestroyCliff(Default::default()),
            DontBuildEntity(..) => DontBuildEntity(Default::default()),
            DontCraftManually(..) => DontCraftManually(Default::default()),
            DontKillManually(..) => todo!(),
            DontResearchBeforeResearching(..) => todo!(),
            DontUseEntityInEnergyProduction { .. } => DontUseEntityInEnergyProduction {
                max_j_per_h: Default::default(),
            },
            EquipArmor(..) => EquipArmor(Default::default()),
            FinishTheGame(..) => FinishTheGame(Default::default()),
            GroupAttack(..) => GroupAttack(Default::default()),
            Kill { .. } => Kill {
                max_killed: Default::default(),
            },
            ModuleTransfer(..) => ModuleTransfer(Default::default()),
            PlaceEquipment(..) => PlaceEquipment(Default::default()),
            PlayerDamaged { .. } => PlayerDamaged {
                max_damage: Default::default(),
                survived: Default::default(),
            },
            Produce { .. } => Produce {
                produced: Default::default(),
            },
            ProducePerHour { .. } => ProducePerHour {
                max_per_h: Default::default(),
            },
            Research => Research,
            ResearchWithSciencePack(..) => ResearchWithSciencePack(Default::default()),
            Shoot(..) => Shoot(Default::default()),
            SpaceConnectionDistanceTraveled(..) => {
                SpaceConnectionDistanceTraveled(Default::default())
            }
            TrainPath { .. } => TrainPath {
                longest_path: Default::default(),
            },
            UseEntityInEnergyProduction(..) => UseEntityInEnergyProduction(Default::default()),
            UseItem(..) => UseItem(Default::default()),
        };
    }
}
