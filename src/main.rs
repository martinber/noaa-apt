extern crate hound;
extern crate rgsl;
extern crate png;
#[macro_use] extern crate log;
extern crate simple_logger;
extern crate argparse;

mod noaa_apt;
mod dsp;
mod wav;
mod misc;

fn main() {

    let mut input_filename = String::new();
    let mut debug = false;
    let mut quiet = false;
    let mut output_filename: Option<String> = None;
    let mut resample_output: Option<u32> = None;
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Decode NOAA APT images from WAV files.");
        parser.refer(&mut input_filename)
            .add_argument("input_filename", argparse::Store,
            "Input WAV file.")
            .required();
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

    match resample_output {
        Some(rate) => { // Just resample the WAV file
            let output = match output_filename {
                Some(filename) => filename,
                None => String::from("./output.wav"),
            };
            noaa_apt::resample_wav(input_filename.as_str(), output.as_str(), rate);
        }
        None => { // Decode WAV file
            let output = match output_filename {
                Some(filename) => filename,
                None => String::from("./output.png"),
            };
            noaa_apt::decode(input_filename.as_str(), output.as_str());
        }
    }
}
