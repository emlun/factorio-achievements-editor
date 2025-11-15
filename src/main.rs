use std::ffi::OsString;
use std::io::Read;

use clap::Parser;
use clap::Subcommand;
use factorio_achievements_editor::AchievementsDat;
use factorio_achievements_editor::Parse;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Dump,
    Delete {
        #[arg(value_name = "ID")]
        id: OsString,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut stdin = std::io::stdin();

    let data = AchievementsDat::parse(&mut stdin)?;

    match cli.command {
        None | Some(Command::Dump) => {
            dbg!(data);
        }
        _ => unimplemented!(),
    }

    #[cfg(debug_assertions)]
    dbg!({
        let mut buf = Vec::new();
        stdin.read_to_end(&mut buf)?;
        buf
    });

    Ok(())
}
