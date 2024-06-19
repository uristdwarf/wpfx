use std::error::Error;
use std::process;

pub enum Errors {
    // Success = 0,
    CreatingConfigFile = 1,
    ReadingConfigFile = 2,
    WritingConfigFile = 3,
    ParsingConfigFile = 4,
    ConfigAlreadyExists = 5,
    CouldNotExecuteWine = 6,
    CouldNotCreatePrefix = 7,
    NoExeProvided = 8,
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
            Self::NoExeProvided => "No executable was provided, either in wpfx.toml (as 'name') or as an argument to run",
        }
    }
}

pub fn exit_err<E: Error>(err: E, code: Errors) -> ! {
    eprintln!("{}: {err}", code.message());
    process::exit(code as i32)
}

pub fn exit_code(code: Errors) -> ! {
    eprintln!("{}", code.message());
    process::exit(code as i32)
}
