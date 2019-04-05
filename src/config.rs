//! Manages configurations from commandline and settings file

use directories;
use toml;
use serde::Deserialize;

// For std::fs::File.read_to_string()
use std::io::prelude::*;

use noaa_apt::Contrast;
use err;

/// How to launch the program.
#[derive(Clone, Debug)]
pub enum Mode {
    /// Open GUI.
    Gui(GuiSettings),

    /// Show version and quit.
    Version,

    /// Decode image from commandline.
    Decode(DecodeSettings),

    /// Resample image from commandline.
    Resample(ResampleSettings),
}

/// Settings for decoding
#[derive(Clone, Debug)]
pub struct DecodeSettings {
    /// Input filename.
    pub input_filename: String,

    /// Output filename.
    pub output_filename: String,

    /// Whether to sync frames.
    pub sync: bool,

    /// Contrast adjustment method.
    pub contrast_adjustment: Contrast,

    /// If we are exporting steps to WAV.
    pub export_wav: bool,

    /// If we are exporting the filtered signal on resample. When using
    /// `fast_resampling()` this step is VERY slow and RAM heavy (gigabytes!),
    /// so that function checks if this variable is set before doing extra work.
    pub export_resample_filtered: bool,

    /// Sample rate in Hz to use for intermediate processing.
    pub work_rate: u32,

    /// Attenuation in positive dB for the resampling filter.
    pub resample_atten: f32,

    /// Transition band width in Hz for the resampling filter.
    pub resample_delta_freq: f32,

    /// Cutout frequency in Hz of the resampling filter.
    pub resample_cutout: f32,

    /// Attenuation in positive dB for the demodulation filter.
    pub demodulation_atten: f32,
}

/// Settings for resampling
#[derive(Clone, Debug)]
pub struct ResampleSettings {
    /// Input filename.
    pub input_filename: String,

    /// Output filename.
    pub output_filename: String,

    /// If we are exporting steps to WAV.
    pub export_wav: bool,

    /// If we are exporting the filtered signal on resample. When using
    /// `fast_resampling()` this step is VERY slow and RAM heavy (gigabytes!),
    /// so that function checks if this variable is set before doing extra work.
    pub export_resample_filtered: bool,

    /// Sample rate in Hz to output.
    pub output_rate: u32,

    /// Attenuation in positive dB for the resampling filter.
    pub wav_resample_atten: f32,

    /// Transition band width in fractions of pi radians per second for the
    /// resampling filter.
    pub wav_resample_delta_freq: f32,
}

/// Settings for GUI decoding/resampling
#[derive(Clone, Debug)]
pub struct GuiSettings {
    /// Sample rate to use for intermediate processing when decoding.
    pub work_rate: u32,

    /// Attenuation in positive dB for the resampling filter used when decoding.
    pub resample_atten: f32,

    /// Transition band width in Hz for the resampling filter used when decoding.
    pub resample_delta_freq: f32,

    /// Cutout frequency in Hz of the resampling filter used when decoding.
    pub resample_cutout: f32,

    /// Attenuation in positive dB for the demodulation filter used when
    /// decoding.
    pub demodulation_atten: f32,

    /// Attenuation in positive dB for the resampling filter used when
    /// resampling WAV files.
    pub wav_resample_atten: f32,

    /// Transition band width in fractions of pi radians per second for the
    /// resampling filter used when resampling WAV files.
    pub wav_resample_delta_freq: f32,
}

/// Holds the deserialized raw parsed settings file.
#[derive(Deserialize)]
struct DeSettings {
    check_updates: bool,
    profiles: DeProfiles,
}

/// Holds the deserialized raw parsed profiles table
#[derive(Deserialize)]
struct DeProfiles {
    default_profile: String,
    standard: DeProfile,
    fast: DeProfile,
    slow: DeProfile,
}

/// Holds each deserialized raw parsed profile subtable
#[derive(Deserialize)]
struct DeProfile {
    work_rate: i64,
    resample_atten: i64,
    resample_delta_freq: i64,
    resample_cutout: i64,
    demodulation_atten: i64,
    wav_resample_atten: i64,
    wav_resample_delta_freq: f64,
}

/// Parse `DeSettings` from file
fn parse_from_file(filename: &std::path::PathBuf) -> err::Result<DeSettings> {
    let mut file = std::fs::File::open(filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(toml::from_str(text.as_str())?)
}

/// Load `DeSettings` from settings file.
///
/// Tries to create the settings file if it's not available and loads the
/// default settings.
fn load_de_settings() -> DeSettings {

    let default_settings_str = include_str!("default_settings.toml");

    if let Some(proj_dirs) = directories::ProjectDirs::from("ar.com.mbernardi", "", "noaa-apt") {

        let filename = proj_dirs.config_dir().join("settings.toml");

        if let Ok(de_settings) = parse_from_file(&filename) {

            return de_settings

        } else {

            let _result = std::fs::create_dir(proj_dirs.config_dir());
            if let Ok(mut file) = std::fs::File::create(&filename) {
                println!(
                    "Missing or corrupted settings file, created default \
                    settings file on {:?}",
                    &filename,
                    );
                file.write_all(default_settings_str.as_bytes())
                    .expect("Could not write to file");
            } else {
                println!(
                    "Could not open or create settings file ({:?}), using default settings",
                    &filename,
                );
            }
            return toml::from_str(default_settings_str).expect(
                "Failed to parse default settings"
            )
        }
    } else {
        println!("Could not get system settings directory, using default settings");
        return toml::from_str(default_settings_str).expect(
            "Failed to parse default settings"
        )
    }
}

/// Read commandline arguments and load settings to decide the settings to
/// return.
///
/// Returns if we should check for updates, the verbosity and the mode including
/// the settings.
pub fn get_config() -> (bool, log::Level, Mode) {

    // Parse commandline

    let mut input_filename: Option<String> = None;
    let mut debug = false;
    let mut quiet = false;
    let mut wav_steps = false;
    let mut export_resample_filtered = false;
    let mut sync = true;
    let mut contrast_adjustment: Option<String> = None;
    let mut profile: Option<String> = None;
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
        parser.refer(&mut profile)
            .add_option(&["-p", "--profile"], argparse::StoreOption,
            "Profile to use, values loaded from settings file. Possible values: \
            \"standard\", \"fast\" or \"slow\".");
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

    // Open settings file
    let de_settings = load_de_settings();

    // Decide and merge commandline arguments and settings

    // Select commandline profile, otherwise load default
    let profile: String = profile.unwrap_or(de_settings.profiles.default_profile);
    // Translate string to struct
    let profile: DeProfile = match profile.as_str() {
        "standard" => de_settings.profiles.standard,
        "fast" => de_settings.profiles.fast,
        "slow" => de_settings.profiles.slow,
        string => {
            println!("Invalid profile \"{}\", using standard profile", string);
            de_settings.profiles.standard
        },
    };

    let check_updates = de_settings.check_updates;

    let verbosity = if debug {
        log::Level::Debug
    } else if quiet {
        log::Level::Warn
    } else {
        log::Level::Info
    };

    if print_version {
        return (check_updates, verbosity, Mode::Version);
    }

    // If set, then the program will be used as a command-line one, otherwise we
    // open the GUI
    if let Some(input_filename) = input_filename {

        // If set, we are resampling, otherwise we are decoding
        if let Some(rate) = resample_output {

            let settings = ResampleSettings {
                input_filename,
                output_filename: output_filename.unwrap_or("./output.png".to_string()),
                export_wav: wav_steps,
                export_resample_filtered,
                output_rate: rate,
                wav_resample_atten: profile.wav_resample_atten as f32,
                wav_resample_delta_freq: profile.wav_resample_delta_freq as f32,
            };

            return (check_updates, verbosity, Mode::Resample(settings));

        // resample_output option not set, decode WAV file
        } else {

            // See https://stackoverflow.com/questions/48034119/rust-matching-a-optionstring
            let contrast_adjustment: Contrast = match contrast_adjustment
                .as_ref()
                .map(|s| s.as_str())
            {
                Some("telemetry") => Contrast::Telemetry,
                Some("disable") => Contrast::MinMax,
                Some("98_percent") | None => Contrast::Percent(0.98),
                Some(_) => {
                    println!("Invalid contrast adjustment argument");
                    std::process::exit(0);
                },
            };

            let settings = DecodeSettings {
                input_filename,
                output_filename: output_filename.unwrap_or("./output.png".to_string()),
                export_wav: wav_steps,
                export_resample_filtered,
                sync,
                contrast_adjustment,
                work_rate: profile.work_rate as u32,
                resample_atten: profile.resample_atten as f32,
                resample_delta_freq: profile.resample_delta_freq as f32,
                resample_cutout: profile.resample_cutout as f32,
                demodulation_atten: profile.demodulation_atten as f32,
            };

            return (check_updates, verbosity, Mode::Decode(settings));
        }

    // Input filename not set, launch GUI
    } else {

        let settings = GuiSettings {
            work_rate: profile.work_rate as u32,
            resample_atten: profile.resample_atten as f32,
            resample_delta_freq: profile.resample_delta_freq as f32,
            resample_cutout: profile.resample_cutout as f32,
            demodulation_atten: profile.demodulation_atten as f32,
            wav_resample_atten: profile.wav_resample_atten as f32,
            wav_resample_delta_freq: profile.wav_resample_delta_freq as f32,
        };

        return (check_updates, verbosity, Mode::Gui(settings));

    }

}
