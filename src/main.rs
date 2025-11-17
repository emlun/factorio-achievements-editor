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

use std::io::Read;

use clap::Parser;
use clap::Subcommand;
use factorio_achievements_editor::AchievementsDat;
use factorio_achievements_editor::Parse;
use factorio_achievements_editor::Serialize;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// (Default) Parse standard input and dump contents to standard error
    Dump,

    /// Delete the achivement with the given ID, and print the edited file to standard output
    Delete {
        /// The achievement to delete
        #[arg(value_name = "ID")]
        id: String,
    },

    /// List achivement IDs present in standard input
    List,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut stdin = std::io::stdin();
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf)?;

    let data = AchievementsDat::parse(&mut buf.as_slice())?;

    match cli.command {
        None | Some(Command::Dump) => {
            dbg!(data);
        }

        Some(Command::Delete { id }) => {
            let data = data.delete(id.as_bytes());
            data.serialize(&mut std::io::stdout())?;
        }

        Some(Command::List) => {
            dbg!(data.list());
        }
    }

    #[cfg(debug_assertions)]
    dbg!({
        let mut buf = Vec::new();
        stdin.read_to_end(&mut buf)?;
        buf
    });

    Ok(())
}
