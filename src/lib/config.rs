use std::fs::{self, File};
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::process::{self, Command};

use crate::errors::{exit_code, exit_err, Errors};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct App {
    // Path to executable
    pub executable: Option<String>,
    pub runner: String,
    pub prefix: String,
    pub gamescope: Gamescope,
    // Whether to enable DXVK or not (requires manually adding the dll's, see
    // https://github.com/doitsujin/dxvk for more info
    // Will set the envoirenment variable WINEDLLOVERRIDES to "dxgi,d3d11,d3d10core,d3d9=n"
    // TODO: Allow setting specific DLL overrides
    pub dxvk: bool,
}

impl Default for App {
    fn default() -> Self {
        App {
            executable: None,
            runner: String::from("wine"),
            prefix: String::from("pfx"),
            gamescope: Gamescope::default(),
            dxvk: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Gamescope {
    pub enabled: bool,
    pub output_width: String,
    pub output_height: String,
    pub game_width: String,
    pub game_height: String,
    pub fullscreen: bool,
    pub relative_mouse: bool,
}

// TODO: Look for ways on wayland systems
fn get_resolution() -> String {
    let shell_command = "xrandr | grep ' connected' | grep -oP '\\d+x\\d+' | sort -nr | head -n 1";

    match Command::new("sh").arg("-c").arg(shell_command).output() {
        Ok(out) => {
            if !out.status.success() || out.stdout.is_empty() {
                // Special case
                eprintln!(
                    "xrandr did not exit successfully: {}",
                    String::from_utf8(out.stderr).unwrap().trim()
                );
                eprintln!("defaulting to 1920x1080");
                return String::from("1920x1080");
            };
            String::from_utf8(out.stdout).unwrap().trim().to_owned()
        }
        Err(e) => {
            eprintln!("could not execute xrandr: {}", e);
            eprintln!("defaulting to 1920x1080");
            String::from("1920x1080")
        }
    }
}

impl Default for Gamescope {
    fn default() -> Self {
        let res = get_resolution();
        println!("{}", res);
        let width = res.split_once('x').unwrap().0.to_owned();
        let height = res.split_once('x').unwrap().1.to_owned();
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
// TODO: Add global configuration
pub fn read_or_init_config(path: &str) -> App {
    let toml_config = fs::read_to_string(path).unwrap_or_else(|err| match err.kind() {
        ErrorKind::NotFound => {
            eprintln!("wpfx.toml file not found, creating it");
            // TODO: Add comments to configuration file
            let default = toml::to_string::<App>(&App::default()).unwrap();
            let mut file = File::create_new(path)
                .unwrap_or_else(|err| exit_err(err, Errors::CreatingConfigFile));
            file.write_all(default.as_bytes())
                .unwrap_or_else(|err| exit_err(err, Errors::WritingConfigFile));
            default
        }
        _ => {
            exit_err(err, Errors::ReadingConfigFile);
        }
    });
    toml::from_str::<App>(&toml_config)
        .map_err(|err| exit_err(err, Errors::ParsingConfigFile))
        .unwrap()
}

pub fn init_config(path: &str) {
    if Path::new(path).exists() {
        exit_code(Errors::ConfigAlreadyExists);
    }
    let default_config = App::default();
    let toml_config = toml::to_string(&default_config)
        .unwrap_or_else(|err| exit_err(err, Errors::ParsingConfigFile));
    fs::write(path, toml_config).unwrap_or_else(|err| exit_err(err, Errors::ReadingConfigFile));

    println!("Successfully created a default wpfx.toml file.");
    println!("Please edit the file as needed.");
    match fs::create_dir(default_config.prefix) {
        Ok(_) => (),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            println!("Prefix already exists, skipping creation...")
        }
        Err(err) => exit_err(err, Errors::CouldNotCreatePrefix),
    }
    process::exit(0);
}
