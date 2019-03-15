#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate toolup_macros;
#[macro_use]
extern crate kopy_common_lib;
extern crate directories;
extern crate json;
extern crate indicatif;
extern crate chrono;
extern crate http;
extern crate tar;
extern crate zip;
extern crate atty;
#[macro_use]
extern crate lazy_static;
#[cfg(target_family = "unix")]
extern crate nix;
extern crate regex;

mod storage;
mod common;
mod commands;
mod config;

use config::{ConfigContainer, parse_config, initialize_configs};

use std::sync::RwLock;
use std::default::Default;

use directories::ProjectDirs;
use clap::{App};

lazy_static! {
    pub static ref CONFIG_DIR: String = {
        let project_dirs = ProjectDirs::from("io", "ehdev", "toolup").expect("To create project dirs");
        s!(project_dirs.config_dir().to_str().unwrap())
    };

    pub static ref CACHE_DIR: String = {
        let project_dirs = ProjectDirs::from("io", "ehdev", "toolup").expect("To create project dirs");
        s!(project_dirs.cache_dir().to_str().unwrap())
    };

    pub static ref CONFIG_DATA: RwLock<Box<ConfigContainer>> = RwLock::new(Box::new(ConfigContainer::default()));
}

fn main() {
    let yml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yml)
        .version(&*format!("v{}", crate_version!()))
        .get_matches();

    kopy_common_lib::configure_logging(
        matches.occurrences_of("debug") as i32,
        matches.is_present("warn"),
        matches.is_present("quite"),
    );

    initialize_configs(&matches);
   
    match matches.subcommand_name() {
        Some("path") | Some("init") | None => {},
        _ => {
            if let Err(err) = parse_config(&matches) {
                eprintln!("Error while running toolup: {}", err);
                std::process::exit(err.into())
            }
        }
    }

    let command = match matches.subcommand() {
        ("path", Some(_)) => { println!("{}", s!(CONFIG_DIR)); Ok(0) }
        ("show-version", Some(cmd_match)) => commands::run_show_version(cmd_match),
        ("lock-tool", Some(cmd_match)) => commands::run_lock_tool(cmd_match),
        ("unlock-tool", Some(cmd_match)) => commands::run_unlock_tool(cmd_match),
        ("status", Some(cmd_match)) => commands::run_status(cmd_match),
        ("update", Some(cmd_match)) => commands::run_update(cmd_match),
        ("run", Some(cmd_match)) => commands::run_exec(cmd_match),
        ("init", Some(cmd_match)) => commands::manage::init(cmd_match),
        ("manage", Some(cmd_match)) => {
            match cmd_match.subcommand() {
                ("add-tool", Some(cmd_match)) => commands::manage::add_tool(cmd_match),
                ("delete-tool", Some(cmd_match)) => commands::manage::delete_tool(cmd_match),
                _ => { panic!("This is a bug, please report the command wasn't found.")}
            }
        }
        _ => { panic!("This is a bug, please report the command wasn't found.")}
    };

    match command {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("Error while running toolup: {}", err);
            std::process::exit(err.into())
        }
    }
}
