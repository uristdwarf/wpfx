use std::{
    env::{self, current_dir, VarError},
    path::Path,
    process::Command,
};

use crate::config;

pub fn create_command(app: &config::App) -> Command {
    let mut command = match app.gamescope.enabled {
        false => Command::new(&app.runner),
        true => gamescope_command(app),
    };
    set_command_env_variables(&mut command, app);
    command
}

pub fn set_command_env_variables(command: &mut Command, app: &config::App) {
    set_command_env(command, "WINEPREFIX", &get_absolute_path(&app.prefix));
    if app.dxvk {
        set_command_env(command, "WINEDLLOVERRIDES", "dxgi,d3d11,d3d10core,d3d9=n,b");
    }
}

fn set_command_env(command: &mut Command, key: &str, value: &str) {
    match env::var(key) {
        Ok(v) => command.env(key, v),
        Err(VarError::NotPresent) => command.env(key, value),
        Err(e) => panic!("Failed to get environment variable: {}", e),
    };
}

fn gamescope_command(config: &config::App) -> Command {
    let mut gamescope = Command::new("gamescope");
    gamescope
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
