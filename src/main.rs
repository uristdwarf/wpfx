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
    env::current_dir,
    io::{stderr, stdout},
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{self, Command},
};
use wpfx::config::*;
use wpfx::errors::*;

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
            let pfx = get_absolute_path(&config.prefix);
            let mut command = match config.gamescope.enabled {
                false => Command::new(&config.runner),
                true => gamescope_command(&config),
            };

            if config.dxvk {
                command.env("WINEDLLOVERRIDES", "dxgi,d3d11,d3d10core,d3d9=n,b");
            }

            let result = command
                .env("WINEPREFIX", pfx)
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

fn gamescope_command(config: &App) -> Command {
    let mut gamescope = Command::new("gamescope");
    gamescope
        // TODO: Prefer explicitly set env variables
        .env("WINEPREFIX", &config.prefix)
        .arg("-W")
        .arg(&config.gamescope.output_width)
        .arg("-H")
        .arg(&config.gamescope.output_height)
        .arg("-w")
        .arg(&config.gamescope.game_width)
        .arg("-h")
        .arg(&config.gamescope.game_height);

    if config.gamescope.relative_mouse {
        gamescope.arg("--force-grab-cursor");
    }
    if config.gamescope.fullscreen {
        gamescope.arg("--fullscreen");
    }
    gamescope.arg("--").arg(&config.runner);

    gamescope
}

fn get_absolute_path(path: &String) -> String {
    if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        let mut dir = current_dir().expect("could not get current working directory");
        dir.push(path);
        return dir.to_str().unwrap().to_string();
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
