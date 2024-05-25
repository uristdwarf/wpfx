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
    error::Error,
    fs::{self, File},
    io::{stderr, stdout, ErrorKind, Write},
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{self, Command},
};

const CONFIG_PATH: &str = "wpfx.toml";

enum Errors {
    // Success = 0,
    CreatingConfigFile = 1,
    ReadingConfigFile = 2,
    WritingConfigFile = 3,
    ParsingConfigFile = 4,
    ConfigAlreadyExists = 5,
    CouldNotExecuteWine = 6,
    CouldNotCreatePrefix = 7,
}

impl Errors {
    fn message(&self) -> &'static str {
        match self {
            // Self::Success => "Operation successful",
            Self::CreatingConfigFile => "Failed creating config file",
            Self::ReadingConfigFile => "Failed reading config file",
            Self::WritingConfigFile => "Failed writing config file",
            Self::ParsingConfigFile => "Failed parsing config file",
            Self::ConfigAlreadyExists => "Configuration file already exists",
            Self::CouldNotExecuteWine => "Failed to execute wine runner",
            Self::CouldNotCreatePrefix => "Could not create prefix directory",
        }
    }
}

fn exit_err<E: Error>(err: E, code: Errors) -> ! {
    eprintln!("{}: {err}", code.message());
    process::exit(code as i32)
}

fn exit_code(code: Errors) -> ! {
    eprintln!("{}", code.message());
    process::exit(code as i32)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_config(CONFIG_PATH),
        Commands::Run { exe } => {
            let config = read_or_init_config(CONFIG_PATH);
            let pfx = get_absolute_path(&config.prefix);
            let mut command = match config.gamescope.enabled {
                false => Command::new(&config.runner),
                true => gamescope_command(&config),
            };

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
        return path.to_string();
    } else {
        let mut dir = current_dir().expect("could not get current working directory");
        dir.push(path);
        return dir.to_str().unwrap().to_string();
    }
}

fn read_or_init_config(path: &str) -> App {
    let toml_config = fs::read_to_string(path).unwrap_or_else(|err| match err.kind() {
        ErrorKind::NotFound => {
            eprintln!("wpfx.toml file not found, creating it");
            let default = toml::to_string::<App>(&App::default()).unwrap();
            let mut file = File::create_new(CONFIG_PATH)
                .unwrap_or_else(|err| exit_err(err, Errors::CreatingConfigFile));
            file.write_all(default.as_bytes())
                .unwrap_or_else(|err| exit_err(err, Errors::WritingConfigFile));
            return default;
        }
        _ => {
            exit_err(err, Errors::ReadingConfigFile);
        }
    });
    toml::from_str::<App>(&toml_config)
        .map_err(|err| exit_err(err, Errors::ParsingConfigFile))
        .unwrap()
}

fn init_config(path: &str) {
    if Path::new(path).exists() {
        exit_code(Errors::ConfigAlreadyExists);
    }
    let default_config = App::default();
    let toml_config = toml::to_string(&default_config)
        .unwrap_or_else(|err| exit_err(err, Errors::ParsingConfigFile));
    fs::write(path, toml_config).unwrap_or_else(|err| exit_err(err, Errors::ReadingConfigFile));

    println!("Successfully created a default wpfx.toml file.");
    println!("Please edit the file as needed.");
    fs::create_dir(default_config.prefix)
        .unwrap_or_else(|err| exit_err(err, Errors::CouldNotCreatePrefix));
    process::exit(0);
}

fn get_resolution() -> String {
    let shell_command = "xrandr | grep ' connected' | grep -oP '\\d+x\\d+' | sort -nr | head -n 1";

    match Command::new("sh").arg("-c").arg(shell_command).output() {
        Ok(out) => String::from_utf8(out.stdout).unwrap().trim().to_owned(),
        Err(e) => {
            eprintln!("could not execute xrandr: {}", e);
            eprintln!("defaulting to 1920x1080");
            String::from("1920x1080")
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct App {
    name: Option<String>,
    runner: String,
    prefix: String,
    gamescope: Gamescope,
}

impl Default for App {
    fn default() -> Self {
        App {
            name: None,
            runner: String::from("wine"),
            prefix: String::from("pfx"),
            gamescope: Gamescope::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Gamescope {
    enabled: bool,
    output_width: String,
    output_height: String,
    game_width: String,
    game_height: String,
    fullscreen: bool,
    relative_mouse: bool,
}

impl Default for Gamescope {
    fn default() -> Self {
        let res = get_resolution();
        let width = res.split_once("x").unwrap().0.to_owned();
        let height = res.split_once("x").unwrap().1.to_owned();
        Gamescope {
            enabled: false,
            output_width: width.clone(),
            output_height: height.clone(),
            game_width: width,
            game_height: height,
            relative_mouse: false,
            fullscreen: true,
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
        exe: String,
    },
    /// Install application by creating a .desktop file and placing it in the correct places
    Install,
    /// Init application by creating an empty .toml file to contain configuration.
    /// This also creates an empty prefix if one doesn't exist.
    Init,
}
