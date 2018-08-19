extern crate hound;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;
extern crate gtk;
extern crate gdk;
extern crate gio;

#[cfg(test)] extern crate rgsl;

mod noaa_apt;
mod dsp;
mod wav;
mod misc;
mod gui;
mod err;

fn main() {

    let mut input_filename: Option<String> = None;
    let mut debug = false;
    let mut quiet = false;
    let mut output_filename: Option<String> = None;
    let mut resample_output: Option<u32> = None;
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Decode NOAA APT images from WAV files. Run \
                               without arguments to launch the GUI");
        parser.refer(&mut input_filename)
            .add_argument("input_filename", argparse::StoreOption,
            "Input WAV file.");
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

    if debug {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
    } else if quiet {
        simple_logger::init_with_level(log::Level::Warn).unwrap();
    } else {
        simple_logger::init_with_level(log::Level::Info).unwrap();
    }

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
                    noaa_apt::resample_wav(input_filename.as_str(), output.as_str(), rate);
                }

                // Decode WAV file
                None => {
                    let output = match output_filename {
                        Some(filename) => filename,
                        None => String::from("./output.png"),
                    };
                    noaa_apt::decode(input_filename.as_str(), output.as_str());
                }
            }
        }

        // Input filename not set, launch GUI
        None => {
            gui::main();
        }
    }
}
