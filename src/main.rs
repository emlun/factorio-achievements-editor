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

#[cfg(debug_assertions)]
use std::io::Read;

use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::io::NoSeek;
use clap::Parser;
use clap::Subcommand;
use factorio_achievements_editor::AchievementsDat;

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

fn main() -> BinResult<()> {
    let cli = Cli::parse();
    let mut stdin = std::io::stdin();

    let data = AchievementsDat::read_le(&mut NoSeek::new(&mut stdin))?;

    match cli.command {
        None | Some(Command::Dump) => {
            dbg!(data);
        }

        Some(Command::Delete { id }) => {
            let data = data.delete(id.as_bytes());
            data.write_le(&mut NoSeek::new(&mut std::io::stdout()))?;
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
