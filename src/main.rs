//! NOAA APT image decoder

extern crate num;
extern crate hound;
extern crate rustfft;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;
extern crate reqwest;
extern crate directories;
extern crate toml;
extern crate serde;
#[cfg_attr(test, macro_use)] extern crate approx;
#[cfg(feature = "gui")] extern crate gtk;
#[cfg(feature = "gui")] extern crate gdk;
#[cfg(feature = "gui")] extern crate gio;
#[cfg(feature = "gui")] extern crate glib;

mod noaa_apt;
mod dsp;
mod frequency;
mod wav;
mod misc;
mod err;
mod filters;
mod context;
mod telemetry;
mod config;
#[cfg(feature = "gui")] mod gui;

use dsp::{Freq, Rate};
use context::Context;
use noaa_apt::Contrast;


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
                settings.wav_steps.expect("No wav_steps on settings"),
                settings.export_resample_filtered.expect("No export_resample_filtered on settings"),
            );

            match noaa_apt::decode(
                context,
                settings.input_filename.expect("No input_filename in settings").as_str(),
                settings.output_filename.expect("No output_filename in settings").as_str(),
                settings.contrast_adjustment.expect("No contrast_adjustment on settings"),
                settings.sync.expect("No sync on Settings"),
                Rate::hz(settings.work_rate),
                settings.resample_atten,
                settings.resample_delta_freq,
                settings.resample_cutout,
                settings.demodulation_atten,
            ) {
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
                settings.wav_steps.expect("No wav_steps on settings"),
                settings.export_resample_filtered.expect("No export_resample_filtered on settings"),
            );

            let work_rate = Rate::hz(settings.work_rate);

            match noaa_apt::resample_wav(
                context,
                settings.input_filename.expect("No input_filename in settings").as_str(),
                settings.output_filename.expect("No output_filename in settings").as_str(),
                Rate::hz(settings.resample_rate.expect("No resample_rate on settings")),
                settings.wav_resample_atten,
                Freq::pi_rad(settings.wav_resample_delta_freq),
            ) {
                Ok(_) => (),
                Err(e) => error!("{}", e),
            };

        },
    };

    Ok(())
}
