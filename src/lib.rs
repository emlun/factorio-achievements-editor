use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Read;
use std::io::Write;
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
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self>;
}

pub trait Serialize
where
    Self: Sized,
{
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
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
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
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

impl Serialize for SpaceOptimizedString {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        if self.value.len() < 255 {
            w.write_all(&self.value.len().to_le_bytes()[0..1])?;
        } else {
            w.write_all(&[255])?;
            w.write_all(&self.value.len().to_le_bytes()[0..4])?;
        }
        w.write_all(&self.value)
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
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let raw_len = read_exact(read)?;
        let len = usize::try_from(L::from_le_bytes(&raw_len))
            .unwrap_or_else(|_| panic!("Invalid length: {:?}", raw_len));
        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            items.push(T::parse(read)?);
        }
        Ok(Self {
            len: PhantomData,
            items,
        })
    }
}

impl<const LEN: usize, L, T> Serialize for Array<LEN, L, T>
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.items.len().to_le_bytes()[0..LEN])?;
        self.items.iter().try_for_each(|item| item.serialize(w))
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
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let version = [
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
            i16::from_le_bytes(read_exact(read)?),
        ];
        let const_false = read_exact::<1, _>(read)? == [1];
        let headers = Array::parse(read)?;
        let contents = Array::parse(read)?;
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

impl Serialize for AchievementsDat {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.version[0].to_le_bytes())?;
        w.write_all(&self.version[1].to_le_bytes())?;
        w.write_all(&self.version[2].to_le_bytes())?;
        w.write_all(&self.version[3].to_le_bytes())?;
        w.write_all(&[self.const_false.into()])?;
        self.headers.serialize(w)?;
        self.contents.serialize(w)?;
        self.tracked
            .iter()
            .try_for_each(|tracked| w.write_all(&tracked.to_le_bytes()))
    }
}

#[derive(Debug)]
pub struct AchievementHeader {
    typ: SpaceOptimizedString,
    subobjects: Array<2, i16, HeaderSubobject>,
}

impl Parse for AchievementHeader {
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            typ: SpaceOptimizedString::parse(read)?,
            subobjects: Array::parse(read)?,
        })
    }
}

impl Serialize for AchievementHeader {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.typ.serialize(w)?;
        self.subobjects.serialize(w)
    }
}

#[derive(Debug)]
pub struct HeaderSubobject {
    id: SpaceOptimizedString,
    index: i16,
}

impl Parse for HeaderSubobject {
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            id: SpaceOptimizedString::parse(read)?,
            index: i16::from_le_bytes(read_exact(read)?),
        })
    }
}

impl Serialize for HeaderSubobject {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.id.serialize(w)?;
        w.write_all(&self.index.to_le_bytes())
    }
}

#[derive(Debug)]
pub struct AchievementContent {
    typ: SpaceOptimizedString,
    id: SpaceOptimizedString,
    progress: AchievementProgress,
}

impl Parse for AchievementContent {
    fn parse<R: Read>(read: &mut R) -> std::io::Result<Self> {
        let typ = SpaceOptimizedString::parse(read)?;
        let id = SpaceOptimizedString::parse(read)?;
        let progress = AchievementProgress::parse(&typ.value, read)?;
        Ok(Self { typ, id, progress })
    }
}

impl Serialize for AchievementContent {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        self.typ.serialize(w)?;
        self.id.serialize(w)?;
        self.progress.serialize(w)
    }
}

#[derive(Debug)]
pub enum AchievementProgress {
    Achievement,
    BuildEntity([u8; 4]),
    ChangeSurface([u8; 1]),
    CombatRobotCount(i32),
    CompleteObjective,
    ConstructWithRobots { constructed: i32, unknown: [u8; 4] },
    CreatePlatform([u8; 4]),
    DeconstructWithRobots { deconstructed: i32 },
    DeliverByRobots([u8; 4]),
    DepleteResource([u8; 4]),
    DestroyCliff([u8; 4]),
    DontBuildEntity([u8; 5]),
    DontCraftManually([u8; 4]),
    DontUseEntityInEnergyProduction { max_j_per_h: f64 },
    EquipArmor([u8; 4]),
    FinishTheGame([u8; 4]),
    GroupAttack([u8; 4]),
    Kill { max_killed: f64 },
    ModuleTransfer([u8; 4]),
    PlaceEquipment([u8; 4]),
    PlayerDamaged { max_damage: f32, survived: bool },
    Produce { produced: f64 },
    ProducePerHour { max_per_h: f64 },
    Research,
    ResearchWithSciencePack([u8; 4]),
    Shoot([u8; 4]),
    SpaceConnectionDistanceTraveled([u8; 4]),
    TrainPath { longest_path: f64 },
    UseEntityInEnergyProduction([u8; 5]),
    UseItem([u8; 4]),
}

impl AchievementProgress {
    fn parse<R: Read>(typ: &[u8], read: &mut R) -> std::io::Result<Self> {
        use AchievementProgress::*;
        Ok(match typ {
            b"achievement" => Achievement,
            b"build-entity-achievement" => BuildEntity(read_exact(read)?),
            b"change-surface-achievement" => ChangeSurface(read_exact(read)?),
            b"combat-robot-count-achievement" => {
                CombatRobotCount(i32::from_le_bytes(read_exact(read)?))
            }
            b"complete-objective-achievement" => CompleteObjective,
            b"construct-with-robots-achievement" => ConstructWithRobots {
                constructed: i32::from_le_bytes(read_exact(read)?),
                unknown: read_exact(read)?,
            },
            b"create-platform-achievement" => CreatePlatform(read_exact(read)?),
            b"deconstruct-with-robots-achievement" => DeconstructWithRobots {
                deconstructed: i32::from_le_bytes(read_exact(read)?),
            },
            b"deliver-by-robots-achievement" => DeliverByRobots(read_exact(read)?),
            b"deplete-resource-achievement" => DepleteResource(read_exact(read)?),
            b"destroy-cliff-achievement" => DestroyCliff(read_exact(read)?),
            b"dont-build-entity-achievement" => DontBuildEntity(read_exact(read)?),
            b"dont-craft-manually-achievement" => DontCraftManually(read_exact(read)?),
            b"dont-use-entity-in-energy-production-achievement" => {
                DontUseEntityInEnergyProduction {
                    max_j_per_h: f64::from_le_bytes(read_exact(read)?),
                }
            }
            b"equip-armor-achievement" => EquipArmor(read_exact(read)?),
            b"finish-the-game-achievement" => FinishTheGame(read_exact(read)?),
            b"group-attack-achievement" => GroupAttack(read_exact(read)?),
            b"kill-achievement" => Kill {
                max_killed: f64::from_le_bytes(read_exact(read)?),
            },
            b"module-transfer-achievement" => ModuleTransfer(read_exact(read)?),
            b"place-equipment-achievement" => PlaceEquipment(read_exact(read)?),
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
            b"research-with-science-pack-achievement" => ResearchWithSciencePack(read_exact(read)?),
            b"shoot-achievement" => Shoot(read_exact(read)?),
            b"space-connection-distance-traveled-achievement" => {
                SpaceConnectionDistanceTraveled(read_exact(read)?)
            }
            b"train-path-achievement" => TrainPath {
                longest_path: f64::from_le_bytes(read_exact(read)?),
            },
            b"use-entity-in-energy-production-achievement" => {
                UseEntityInEnergyProduction(read_exact(read)?)
            }
            b"use-item-achievement" => UseItem(read_exact(read)?),
            _ => unimplemented!("Unknown achievement type: {}", String::from_utf8_lossy(typ)),
        })
    }
}

impl Serialize for AchievementProgress {
    fn serialize<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        use AchievementProgress::*;
        match self {
            Achievement => Ok(()),
            BuildEntity(data) => w.write_all(data),
            ChangeSurface(data) => w.write_all(data),
            CombatRobotCount(count) => w.write_all(&count.to_le_bytes()),
            CompleteObjective => Ok(()),
            ConstructWithRobots {
                constructed,
                unknown,
            } => {
                w.write_all(&constructed.to_le_bytes())?;
                w.write_all(unknown)
            }
            CreatePlatform(data) => w.write_all(data),
            DeconstructWithRobots { deconstructed } => w.write_all(&deconstructed.to_le_bytes()),
            DeliverByRobots(data) => w.write_all(data),
            DepleteResource(data) => w.write_all(data),
            DestroyCliff(data) => w.write_all(data),
            DontBuildEntity(data) => w.write_all(data),
            DontCraftManually(data) => w.write_all(data),
            DontUseEntityInEnergyProduction { max_j_per_h } => {
                w.write_all(&max_j_per_h.to_le_bytes())
            }
            EquipArmor(data) => w.write_all(data),
            FinishTheGame(data) => w.write_all(data),
            GroupAttack(data) => w.write_all(data),
            Kill { max_killed } => w.write_all(&max_killed.to_le_bytes()),
            ModuleTransfer(data) => w.write_all(data),
            PlaceEquipment(data) => w.write_all(data),
            PlayerDamaged {
                max_damage,
                survived,
            } => {
                w.write_all(&max_damage.to_le_bytes())?;
                w.write_all(&[(*survived).into()])
            }
            Produce { produced } => w.write_all(&produced.to_le_bytes()),
            ProducePerHour { max_per_h } => w.write_all(&max_per_h.to_le_bytes()),
            Research => Ok(()),
            ResearchWithSciencePack(data) => w.write_all(data),
            Shoot(data) => w.write_all(data),
            SpaceConnectionDistanceTraveled(data) => w.write_all(data),
            TrainPath { longest_path } => w.write_all(&longest_path.to_le_bytes()),
            UseEntityInEnergyProduction(data) => w.write_all(data),
            UseItem(data) => w.write_all(data),
        }
    }
}
