extern crate num;
extern crate hound;
extern crate rustfft;
#[cfg_attr(test, macro_use)] extern crate approx;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;
extern crate gtk;
extern crate gdk;
extern crate gio;

mod noaa_apt;
mod dsp;
mod frequency;
mod wav;
mod misc;
mod gui;
mod err;
mod filters;

use dsp::Rate;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> err::Result<()> {

    let mut input_filename: Option<String> = None;
    let mut debug = false;
    let mut quiet = false;
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

    if print_version {
        println!("noaa-apt image decoder version {}", VERSION);
        std::process::exit(0);
    }

    if debug {
        simple_logger::init_with_level(log::Level::Debug)?;
    } else if quiet {
        simple_logger::init_with_level(log::Level::Warn)?;
    } else {
        simple_logger::init_with_level(log::Level::Info)?;
    }

    info!("noaa-apt image decoder version {}", VERSION);

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

                    match noaa_apt::resample_wav(
                            input_filename.as_str(), output.as_str(), Rate::hz(rate)) {
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

                    match noaa_apt::decode(
                            input_filename.as_str(), output.as_str()) {
                        Ok(_) => (),
                        Err(e) => error!("{}", e),
                    };
                }
            }
        }

        // Input filename not set, launch GUI
        None => {
            gui::main();
        }
    }

    Ok(())
}
