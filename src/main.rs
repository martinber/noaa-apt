//! NOAA APT image decoder

extern crate num;
extern crate hound;
extern crate rustfft;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;
extern crate reqwest;
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
#[cfg(feature = "gui")] mod gui;

use dsp::Rate;
use context::Context;
use noaa_apt::Contrast;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application entry point
fn main() -> err::Result<()> {

    let mut input_filename: Option<String> = None;
    let mut debug = false;
    let mut quiet = false;
    let mut wav_steps = false;
    let mut export_resample_filtered = false;
    let mut sync = true;
    let mut contrast_adjustment: Option<String> = None;
    let mut print_version = false;
    let mut output_filename: Option<String> = None;
    let mut resample_output: Option<u32> = None;
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Decode NOAA APT images from WAV files. Run \
                               without arguments to launch the GUI");
        parser.refer(&mut input_filename)
            .add_argument("input_filename", argparse::StoreOption,
            "Input WAV file.");
        parser.refer(&mut print_version)
            .add_option(&["-v", "--version"], argparse::StoreTrue,
            "Show version and quit.");
        parser.refer(&mut debug)
            .add_option(&["-d", "--debug"], argparse::StoreTrue,
            "Print debugging messages.");
        parser.refer(&mut quiet)
            .add_option(&["-q", "--quiet"], argparse::StoreTrue,
            "Don't print info messages.");
        parser.refer(&mut wav_steps)
            .add_option(&["--wav-steps"], argparse::StoreTrue,
            "Export a WAV for every step of the decoding process for debugging, \
            the files will be located on the current folder, named \
            {number}_{description}.wav");
        parser.refer(&mut export_resample_filtered)
            .add_option(&["--export-resample-filtered"], argparse::StoreTrue,
            "Export a WAV for the expanded and filtered signal on the resampling
            step. Very expensive operation, can take several GiB of both RAM and
            disk. --wav-steps should be set.");
        parser.refer(&mut sync)
            .add_option(&["--no-sync"], argparse::StoreFalse,
            "Disable syncing, useful when the sync frames are noisy and the \
            syncing attempts do more harm than good.");
        parser.refer(&mut contrast_adjustment)
            .add_option(&["-c", "--contrast"], argparse::StoreOption,
            "Contrast adjustment method for decode. Possible values: \
            \"98_percent\", \"telemetry\" or \"disable\". 98 Percent used by \
            default.");
        parser.refer(&mut output_filename)
            .add_option(&["-o", "--output"], argparse::StoreOption,
            "Set output path. When decoding images the default is \
            './output.png', when resampling the default is './output.wav'.")
            .metavar("FILENAME");
        parser.refer(&mut resample_output)
            .add_option(&["-r", "--resample"], argparse::StoreOption,
            "Resample WAV file to a given sample rate, no APT image will be \
            decoded.")
            .metavar("SAMPLE_RATE");
        parser.parse_args_or_exit();
    }

    if debug {
        simple_logger::init_with_level(log::Level::Debug)?;
    } else if quiet {
        simple_logger::init_with_level(log::Level::Warn)?;
    } else {
        simple_logger::init_with_level(log::Level::Info)?;
    }

    if print_version {
        println!("noaa-apt image decoder version {}", VERSION);
        match misc::check_updates(VERSION) {
            Some((false, _latest)) => println!("You have the latest version available"),
            Some((true, latest)) => println!("Version \"{}\" available for download!", latest),
            None => println!("Could not retrieve latest version available"),
        }
        std::process::exit(0);
    }

    info!("noaa-apt image decoder version {}", VERSION);

    // See https://stackoverflow.com/questions/48034119/rust-matching-a-optionstring
    let contrast_adjustment: Contrast = match contrast_adjustment
        .as_ref()
        .map(|s| s.as_str())
    {
        Some("98_percent") => Contrast::Percent(0.98),
        Some("telemetry") => Contrast::Telemetry,
        Some("disable") => Contrast::MinMax,
        Some(_) => {
            println!("Invalid contrast adjustment argument");
            std::process::exit(0);
        },
        None => Contrast::Percent(0.98),
    };

    match input_filename {

        // Input filename set, command line
        Some(input_filename) => {
            match resample_output {

                // Just resample the WAV file
                Some(rate) => {
                    let output = match output_filename {
                        Some(filename) => filename,
                        None => String::from("./output.wav"),
                    };

                    let context = Context::resample(
                        wav_steps,
                        export_resample_filtered
                    );

                    match noaa_apt::resample_wav(
                        context,
                        input_filename.as_str(),
                        output.as_str(),
                        Rate::hz(rate),
                    ) {
                        Ok(_) => (),
                        Err(e) => error!("{}", e),
                    };
                }

                // Decode WAV file
                None => {
                    let output = match output_filename {
                        Some(filename) => filename,
                        None => String::from("./output.png"),
                    };

                    let context = Context::decode(
                        Rate::hz(noaa_apt::WORK_RATE),
                        Rate::hz(noaa_apt::FINAL_RATE),
                        wav_steps,
                        export_resample_filtered,
                    );

                    match noaa_apt::decode(
                        context,
                        input_filename.as_str(),
                        output.as_str(),
                        contrast_adjustment,
                        sync,
                    ) {
                        Ok(_) => (),
                        Err(e) => error!("{}", e),
                    };
                }
            }
        }

        // Input filename not set, launch GUI
        None => {
            #[cfg(feature = "gui")]
            {
                gui::main();
            }
            #[cfg(not(feature = "gui"))]
            {
                error!("Program compiled without gui support, please download \
                    the gui version of this program.");
            }
        }
    }

    Ok(())
}
