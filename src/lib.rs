use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Read;
use std::marker::PhantomData;

fn read_exact<const LEN: usize, R: Read>(read: &mut R) -> std::io::Result<[u8; LEN]> {
    let mut buf = [0; LEN];
    read.read_exact(&mut buf)?;
    Ok(buf)
}

pub trait Parse
where
    Self: Sized,
{
    type Ctx;
    fn parse<R: Read>(read: &mut R, ctx: &Self::Ctx) -> std::io::Result<Self>;
}

pub struct SpaceOptimizedString {
    value: Box<[u8]>,
}

impl Debug for SpaceOptimizedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&String::from_utf8(self.value.to_vec()).map_err(|_| std::fmt::Error)?)
    }
}

impl Parse for SpaceOptimizedString {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        let head: [u8; 1] = read_exact(read)?;
        let len: u32 = if head == [255] {
            u32::from_le_bytes(read_exact(read)?)
        } else {
            head[0].into()
        };
        let mut buf: Vec<u8> = vec![0; len.try_into().expect("Failed to parse length as usize")];
        read.read_exact(buf.as_mut_slice())?;
        Ok(Self {
            value: buf.into_boxed_slice(),
        })
    }
}

trait FromLeBytes<const LEN: usize> {
    fn from_le_bytes(bytes: &[u8; LEN]) -> Self;
}

impl FromLeBytes<2> for i16 {
    fn from_le_bytes(bytes: &[u8; 2]) -> Self {
        Self::from_le_bytes(*bytes)
    }
}
impl FromLeBytes<4> for i32 {
    fn from_le_bytes(bytes: &[u8; 4]) -> Self {
        Self::from_le_bytes(*bytes)
    }
}

#[derive(Debug)]
pub struct Array<const LEN: usize, L, T> {
    len: PhantomData<([u8; LEN], L)>,
    items: Vec<T>,
}

impl<const LEN: usize, L, T> Parse for Array<LEN, L, T>
where
    L: FromLeBytes<LEN>,
    usize: TryFrom<L>,
    <usize as TryFrom<L>>::Error: Debug,
    T: Parse,
{
    type Ctx = <T as Parse>::Ctx;
    fn parse<R: Read>(read: &mut R, ctx: &Self::Ctx) -> std::io::Result<Self> {
        let raw_len = read_exact(read)?;
        let len = usize::try_from(L::from_le_bytes(&raw_len))
            .expect(&format!("Invalid length: {:?}", raw_len));
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(T::parse(read, ctx)?);
        }
        Ok(Self {
            len: PhantomData,
            items,
        })
    }
}

#[derive(Debug)]
pub struct AchievementsDat {
    version: [i16; 4],
    const_false: bool,
    headers: Array<2, i16, AchievementHeader>,
    contents: Array<4, i32, AchievementContent>,
    tracked: Vec<i16>,
}

impl Parse for AchievementsDat {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        let version = [
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
        ];
        let const_false = read_exact::<1, _>(read)? == [1];
        let headers = Array::parse(read, &())?;
        let contents = Array::parse(read, &())?;
        Ok(Self {
            version,
            const_false,
            headers,
            contents,
            tracked: {
                let mut buf = Vec::new();
                while let Ok(next) = read_exact(read) {
                    buf.push(i16::from_le_bytes(next));
                }
                buf
            },
        })
    }
}

#[derive(Debug)]
pub struct AchievementsModdedDat {
    version: [i16; 4],
    const_false: bool,
    headers: Array<2, i16, AchievementHeaderModded>,
    contents: Array<4, i32, AchievementContentModded>,
    tracked: Vec<i16>,
}

impl Parse for AchievementsModdedDat {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        Ok(Self {
            version: [
                i16::from_le_bytes(read_exact(read)?),
                i16::from_le_bytes(read_exact(read)?),
                i16::from_le_bytes(read_exact(read)?),
                i16::from_le_bytes(read_exact(read)?),
            ],
            const_false: read_exact::<1, _>(read)? != [1],
            headers: Array::parse(read, &())?,
            contents: Array::parse(read, &())?,
            tracked: {
                let mut buf = Vec::new();
                while let Ok(next) = read_exact(read) {
                    buf.push(i16::from_le_bytes(next));
                }
                buf
            },
        })
    }
}

#[derive(Debug)]
pub struct AchievementHeader {
    typ: SpaceOptimizedString,
    subobjects: Array<2, i16, HeaderSubobject>,
}

impl Parse for AchievementHeader {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        Ok(Self {
            typ: SpaceOptimizedString::parse(read, &())?,
            subobjects: Array::parse(read, &())?,
        })
    }
}

#[derive(Debug)]
pub struct AchievementHeaderModded {
    typ: SpaceOptimizedString,
    subobjects: Array<2, i16, HeaderSubobject>,
}

impl Parse for AchievementHeaderModded {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        Ok(Self {
            typ: SpaceOptimizedString::parse(read, &())?,
            subobjects: Array::parse(read, &())?,
        })
    }
}

#[derive(Debug)]
pub struct HeaderSubobject {
    id: SpaceOptimizedString,
    index: i16,
}

impl Parse for HeaderSubobject {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        Ok(Self {
            id: SpaceOptimizedString::parse(read, &())?,
            index: i16::from_le_bytes(read_exact(read)?),
        })
    }
}

#[derive(Debug)]
pub struct AchievementContent {
    typ: SpaceOptimizedString,
    id: SpaceOptimizedString,
    progress: AchievementProgress,
}

impl Parse for AchievementContent {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &()) -> std::io::Result<Self> {
        let typ = SpaceOptimizedString::parse(read, &())?;
        let id = SpaceOptimizedString::parse(read, &())?;
        let progress = AchievementProgress::parse(&typ.value, read)?;
        Ok(Self { typ, id, progress })
    }
}

#[derive(Debug)]
pub struct AchievementContentModded {
    typ: SpaceOptimizedString,
    id: SpaceOptimizedString,
    progress: AchievementProgress,
}

impl Parse for AchievementContentModded {
    type Ctx = ();
    fn parse<R: Read>(read: &mut R, _: &Self::Ctx) -> std::io::Result<Self> {
        let typ = SpaceOptimizedString::parse(read, &())?;
        let id = SpaceOptimizedString::parse(read, &())?;
        let progress = AchievementProgress::parse(&typ.value, read)?;
        Ok(Self { typ, id, progress })
    }
}

#[derive(Debug)]
pub enum AchievementProgress {
    BuildEntity([u8; 4]),
    CombatRobotCount(i32),
    ConstructWithRobots { constructed: i32, unknown: [u8; 4] },
    DeconstructWithRobots { deconstructed: i32 },
    DeliverByRobots([u8; 4]),
    DontBuildEntity([u8; 5]),
    DontCraftManually([u8; 4]),
    DontUseEntityInEnergyProduction { max_j_per_h: f64 },
    FinishTheGame([u8; 4]),
    GroupAttack([u8; 4]),
    Kill { max_killed: f64 },
    PlayerDamaged { max_damage: f32, survived: bool },
    Produce { produced: f64 },
    ProducePerHour { max_per_h: f64 },
    Research,
    TrainPath { longest_path: f64 },
    Achievement,
    CompleteObjective,
    UseEntityInEnergyProduction([u8; 5]),
    DepleteResource([u8; 4]),
    ResearchWithSciencePack([u8; 4]),
    DestroyCliff([u8; 4]),
    Shoot([u8; 4]),
    CreatePlatform([u8; 4]),
    ChangeSurface([u8; 1]),
    SpaceConnectionDistanceTraveled([u8; 4]),
    ModuleTransfer([u8; 4]),
    EquipArmor([u8; 4]),
    UseItem([u8; 4]),
    PlaceEquipment([u8; 4]),
}

impl AchievementProgress {
    fn parse<R: Read>(typ: &[u8], read: &mut R) -> std::io::Result<Self> {
        use AchievementProgress::*;
        Ok(match typ {
            b"build-entity-achievement" => BuildEntity(read_exact(read)?),
            b"combat-robot-count-achievement" => {
                CombatRobotCount(i32::from_le_bytes(read_exact(read)?))
            }
            b"construct-with-robots-achievement" => ConstructWithRobots {
                constructed: i32::from_le_bytes(read_exact(read)?),
                unknown: read_exact(read)?,
            },
            b"deconstruct-with-robots-achievement" => DeconstructWithRobots {
                deconstructed: i32::from_le_bytes(read_exact(read)?),
            },
            b"deliver-by-robots-achievement" => DeliverByRobots(read_exact(read)?),
            b"dont-build-entity-achievement" => DontBuildEntity(read_exact(read)?),
            b"dont-craft-manually-achievement" => DontCraftManually(read_exact(read)?),
            b"dont-use-entity-in-energy-production-achievement" => {
                DontUseEntityInEnergyProduction {
                    max_j_per_h: f64::from_le_bytes(read_exact(read)?),
                }
            }
            b"finish-the-game-achievement" => FinishTheGame(read_exact(read)?),
            b"group-attack-achievement" => GroupAttack(read_exact(read)?),
            b"kill-achievement" => Kill {
                max_killed: f64::from_le_bytes(read_exact(read)?),
            },
            b"player-damaged-achievement" => PlayerDamaged {
                max_damage: f32::from_le_bytes(read_exact(read)?),
                survived: read_exact(read)? == [1],
            },
            b"produce-achievement" => Produce {
                produced: f64::from_le_bytes(read_exact(read)?),
            },
            b"produce-per-hour-achievement" => ProducePerHour {
                max_per_h: f64::from_le_bytes(read_exact(read)?),
            },
            b"research-achievement" => Research,
            b"train-path-achievement" => TrainPath {
                longest_path: f64::from_le_bytes(read_exact(read)?),
            },
            b"achievement" => Achievement,
            b"complete-objective-achievement" => CompleteObjective,
            b"use-entity-in-energy-production-achievement" => {
                UseEntityInEnergyProduction(read_exact(read)?)
            }
            b"deplete-resource-achievement" => DepleteResource(read_exact(read)?),
            b"research-with-science-pack-achievement" => ResearchWithSciencePack(read_exact(read)?),
            b"destroy-cliff-achievement" => DestroyCliff(read_exact(read)?),
            b"shoot-achievement" => Shoot(read_exact(read)?),
            b"create-platform-achievement" => CreatePlatform(read_exact(read)?),
            b"change-surface-achievement" => ChangeSurface(read_exact(read)?),
            b"space-connection-distance-traveled-achievement" => {
                SpaceConnectionDistanceTraveled(read_exact(read)?)
            }
            b"module-transfer-achievement" => ModuleTransfer(read_exact(read)?),
            b"equip-armor-achievement" => EquipArmor(read_exact(read)?),
            b"use-item-achievement" => UseItem(read_exact(read)?),
            b"place-equipment-achievement" => PlaceEquipment(read_exact(read)?),
            _ => unimplemented!("Unknown achievement type: {}", String::from_utf8_lossy(typ)),
        })
    }
}
