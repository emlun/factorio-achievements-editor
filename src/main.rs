use factorio_achievements_editor::AchievementsDat;
use factorio_achievements_editor::AchievementsModdedDat;
use factorio_achievements_editor::Parse;

fn main() -> std::io::Result<()> {
    // let data = AchievementsDat::parse(&mut std::io::stdin(), &())?;
    let data = AchievementsModdedDat::parse(&mut std::io::stdin(), &())?;
    dbg!(data);
    Ok(())
}
