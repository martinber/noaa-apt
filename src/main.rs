//! NOAA APT image decoder

#![cfg_attr(not(feature = "windows_console"), windows_subsystem = "windows")]

mod config;
mod context;
mod dsp;
mod err;
mod filters;
mod frequency;
#[cfg(feature = "gui")] mod gui;
mod misc;
mod noaa_apt;
mod telemetry;
mod wav;

use log::{debug, error, info};

use dsp::Rate;
use context::Context;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application entry point
fn main() -> err::Result<()> {

    let (check_updates, verbosity, mode) = config::get_config();

    simple_logger::init_with_level(verbosity)?;

    debug!("Mode: {:?}", mode);

    match mode {
        config::Mode::Version => {

            println!("noaa-apt image decoder version {}", VERSION);
            match misc::check_updates(VERSION) {
                Some((false, _latest)) => println!("You have the latest version available"),
                Some((true, latest)) => println!("Version \"{}\" available for download!", latest),
                None => println!("Could not retrieve latest version available"),
            }
            std::process::exit(0);

        },
        config::Mode::Gui(settings) => {

            #[cfg(feature = "gui")]
            {
                gui::main(check_updates, settings);
            }
            #[cfg(not(feature = "gui"))]
            {
                error!("Program compiled without gui support, please download \
                    the gui version of this program or use --help to see available \
                    options.");
            }

        },
        config::Mode::Decode(settings) => {

            if check_updates {
                println!("noaa-apt image decoder version {}", VERSION);
            }

            let context = Context::decode(
                |_progress, description| info!("{}", description),
                Rate::hz(settings.work_rate),
                Rate::hz(noaa_apt::FINAL_RATE),
                settings.export_wav,
                settings.export_resample_filtered,
            );

            match noaa_apt::decode(context, settings) {
                Ok(_) => (),
                Err(e) => error!("{}", e),
            };

        },
        config::Mode::Resample(settings) => {

            if check_updates {
                println!("noaa-apt image decoder version {}", VERSION);
            }

            let context = Context::resample(
                |_progress, description| info!("{}", description),
                settings.export_wav,
                settings.export_resample_filtered,
            );

            match noaa_apt::resample_wav(context, settings) {
                Ok(_) => (),
                Err(e) => error!("{}", e),
            };

        },
    };

    Ok(())
}
