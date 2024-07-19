mod gist;

use clap::Parser;
use fastcmp::Compare;
use gist::Gist;
use homedir::my_home;
use std::{
    fs::{self, File},
    io::Write,
};

const GIST_NAME: &str = "zed-config.json";

#[derive(clap::ValueEnum, Clone, Default)]
enum Mode {
    #[default]
    Sink,
    Source,
}

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t, value_enum)]
    mode: Mode,
}

fn main() {
    let args = Args::parse();

    let home_path = my_home().unwrap().expect("error getting home path");
    let config_file_path = format!("{}/.config/zed/settings.json", home_path.to_str().unwrap());

    let config_file = fs::read(config_file_path.clone()).expect("error reading Zed config file");

    let mut gist = Gist::new_with_name(GIST_NAME);

    match gist.get_hash() {
        Some(_) => match args.mode {
            Mode::Sink => sink(&mut gist, config_file_path),
            Mode::Source => {
                gist.edit(config_file_path)
                    .expect("error persisting new config content");
            }
        },
        None => {
            println!(
                "No gist named '{}' found in your GitHub account. Creating with current config...",
                GIST_NAME
            );

            Gist::new_with_name(GIST_NAME)
                .create(config_file)
                .expect("error creating gist {}");
        }
    }
}

fn sink(gist: &mut Gist, config_path: String) {
    let config_file = fs::read(config_path.as_str()).expect("error reading Zed config file");
    let gist_content = gist.content().expect("error getting gist content");

    let are_equal = config_file.feq(gist_content.as_slice());

    if !are_equal {
        let mut file = File::options()
            .write(true)
            .open(config_path.as_str())
            .expect("unable to open config file");

        file.write_all(gist_content.as_slice())
            .expect("unable to write config file update");

        let mut bak_file = File::options()
            .create(true)
            .write(true)
            .open(
                config_path
                    .as_str()
                    .replace("settings.json", "settings.bak.json"),
            )
            .expect("unable to open config bak file");

        bak_file
            .write_all(config_file.as_slice())
            .expect("unable to write config file update");

        println!("Config file updated successfully!");
    }
}
