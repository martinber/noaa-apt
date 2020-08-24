//! NOAA APT image decoder

// Do not show terminal window on MS Windows
#![cfg_attr(not(feature = "windows_console"), windows_subsystem = "windows")]


// I like to use `return` when it makes things clearer
#![allow(clippy::needless_return)]
// Gives a warning when I take `&Signal` (alias of `Vec<f32>`) as arguments
// instead of slices. Writing `&Signal` makes the code clearer
#![allow(clippy::ptr_arg)]
// Makes code clearer, see
// https://doc.rust-lang.org/edition-guide/rust-2018/ownership-and-lifetimes/the-anonymous-lifetime.html
#![warn(elided_lifetimes_in_paths)]


#[macro_use] mod config;
mod context;
mod decode;
mod dsp;
mod err;
mod imageext;
mod filters;
mod frequency;
mod geo;
#[cfg(feature = "gui")] mod gui;
mod map;
mod misc;
mod noaa_apt;
mod processing;
mod resample;
mod telemetry;
mod wav;

use log::{debug, error, info, warn};

use dsp::Rate;
use context::Context;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main function that returns err::Result
fn inner_main() -> err::Result<()> {
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
        },
        config::Mode::Gui { settings } => {

            #[cfg(feature = "gui")]
            {
                gui::main(check_updates, settings);
            }
            #[cfg(not(feature = "gui"))]
            {
                return Err(err::Error::FeatureNotAvailable("Program compiled \
                    without gui support, please download the gui version of \
                    this program or use --help to see available options.".to_string()
                ));
            }

        },
        config::Mode::Decode {
            settings,
            input_filename,
            output_filename,
            sync,
            contrast_adjustment,
            rotate,
            color_settings,
            orbit_settings,
        } => {

            println!("noaa-apt image decoder version {}", VERSION);

            if !sync {
                match contrast_adjustment {
                    noaa_apt::Contrast::Telemetry | noaa_apt::Contrast::Histogram =>
                        warn!("Adjusting contrast without syncing, expect horrible results!"),
                    _ => ()
                }
            }

            let mut context = Context::decode(
                |_progress, description| info!("{}", description),
                Rate::hz(settings.work_rate),
                Rate::hz(noaa_apt::FINAL_RATE),
                settings.export_wav,
                settings.export_resample_filtered,
            );

            let (signal, rate) = noaa_apt::load(&input_filename)?;

            let raw_data = noaa_apt::decode(
                &mut context,
                &settings,
                &signal,
                rate,
                sync
            )?;

            let img = noaa_apt::process(
                &mut context,
                &settings,
                &raw_data,
                contrast_adjustment,
                rotate,
                color_settings,
                orbit_settings,
            )?;

            img.save(&output_filename)?;

        },
        config::Mode::Resample {
            settings,
            input_filename,
            output_filename,
            output_rate,
        } => {

            println!("noaa-apt image decoder version {}", VERSION);

            let mut context = Context::resample(
                |_progress, description| info!("{}", description),
                settings.export_wav,
                settings.export_resample_filtered,
            );

            noaa_apt::resample(
                &mut context,
                settings,
                &input_filename,
                &output_filename,
                output_rate,
            )?;

        },
    };

    Ok(())
}

/// Application entry point.
///
/// Logs errors and exits.
fn main() {

    std::process::exit(match inner_main() {
        Ok(_) => 0,

        Err(err) => {
            error!("{}", err);

            1
        },
    })
}
