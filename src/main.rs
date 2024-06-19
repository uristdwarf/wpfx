/*
    wpfx - Wine gaming done the Unix way
    Copyright (C) 2024 Urmas Rist

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{Parser, Subcommand};
use std::{
    io::{stderr, stdout},
    os::unix::process::ExitStatusExt,
    process::{self},
};
use wpfx::errors::*;
use wpfx::{commands, config::*};

const CONFIG_PATH: &str = "wpfx.toml";

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_config(CONFIG_PATH),
        Commands::Run { exe } => {
            let config = read_or_init_config(CONFIG_PATH);
            let exe = exe.unwrap_or_else(|| {
                config
                    .executable
                    .as_ref()
                    .unwrap_or_else(|| exit_code(Errors::NoExeProvided))
                    .to_string()
            });
            let mut command = commands::create_command(&config);

            let result = command
                .arg(exe)
                .stdout(stdout())
                .stderr(stderr())
                .output()
                .unwrap_or_else(|err| exit_err(err, Errors::CouldNotExecuteWine));

            process::exit(
                result
                    .status
                    .code()
                    .unwrap_or_else(|| result.status.signal().unwrap()),
            );
        }
        Commands::Install => {
            println!("Installing app...");
            // TODO: Implementation of the install command
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, PartialEq, Eq)]
enum Commands {
    /// Run application
    Run {
        // Executable to run
        exe: Option<String>,
    },
    /// Install application by creating a .desktop file and placing it in the correct places
    Install,
    /// Initialize application by creating an empty .toml file to contain configuration.
    /// This also creates an empty prefix if one doesn't exist.
    Init,
}
