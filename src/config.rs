//! Manages configurations from commandline and settings file


use std::fs::File;
use std::io::prelude::*; // For std::fs::File.read_to_string()
use std::path::PathBuf;

use serde::Deserialize;

use crate::err;
use crate::misc;
use crate::noaa_apt::{OrbitSettings, MapSettings, Rotate, Contrast, SatName, RefTime};

// Expected configuration file version.
const SETTINGS_VERSION: u32 = 2;


/// Returns a PathBuf of the requested resource file.
///
///
/// Expands the path using the resources directory given at compile time. For
/// example use `res_path!("shapefiles", "lakes.shp");` to get
/// `./res/shapefiles/lakes.shp` or `/usr/share/noaa-apt/shapefiles/lakes.shp`
#[macro_export]
macro_rules! res_path {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_path = std::path::PathBuf::from(
                option_env!("NOAA_APT_RES_DIR").unwrap_or("./res/")
            );
            $(
                temp_path = temp_path.join($x);
            )*
            temp_path
        }
    };
}

/// How to launch the program.
#[derive(Debug)]
pub enum Mode {
    /// Open GUI.
    Gui {
        settings: Settings
    },

    /// Show version and quit.
    Version,

    /// Decode image from commandline.
    Decode {
        settings: Settings,
        input_filename: PathBuf,
        output_filename: PathBuf,
        sync: bool,
        contrast_adjustment: Contrast,
        rotate: Rotate,
        orbit_settings: Option<OrbitSettings>,
    },

    /// Resample image from commandline.
    Resample {
        settings: Settings,
        input_filename: PathBuf,
        output_filename: PathBuf,
        output_rate: u32,
    },
}

/// Settings for decoding/resampling
///
/// This structs contains settings loaded from the command line or toml file.
#[derive(Clone, Debug)]
pub struct Settings {
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

    /// Attenuation in positive dB for the resampling filter (used only when
    /// resampling WAV files).
    pub wav_resample_atten: f32,

    /// Transition band width in fractions of pi radians per second for the
    /// resampling filter (used only when resampling WAV files).
    pub wav_resample_delta_freq: f32,

    /// If the user prefers to obtain recording times from timestamps instead of
    /// filenames.
    pub prefer_timestamps: bool,

    /// Filename formats to parse when trying to obtain recording times.
    pub filename_formats: Vec<String>,

    /// Timezone offset to use when parsing filenames, in hours.
    pub filename_timezone: f32,

    /// Default countries and coastlines color as RGBA.
    pub default_countries_color: (u8, u8, u8, u8),

    /// Default provinces and states color as RGBA.
    pub default_states_color: (u8, u8, u8, u8),

    /// Default lakes color as RGBA.
    pub default_lakes_color: (u8, u8, u8, u8),
}

/// Holds the deserialized raw parsed settings file.
#[derive(Deserialize)]
struct DeSettings {
    check_updates: bool,
    version: u32,
    timestamps: DeTimestamps,
    profiles: DeProfiles,
    map_overlay: DeMapOverlay,
}

/// Holds the deserialized raw parsed timestamps table
#[derive(Deserialize)]
struct DeTimestamps {
    prefer_timestamps: bool,
    filenames: Vec<String>,
    timezone: f32,
}

/// Holds the deserialized raw parsed map_overlay table
#[derive(Deserialize)]
struct DeMapOverlay {
    default_countries_color: (u8, u8, u8, u8),
    default_states_color: (u8, u8, u8, u8),
    default_lakes_color: (u8, u8, u8, u8),
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
    let de_settings: DeSettings = toml::from_str(text.as_str())?;

    if de_settings.version == SETTINGS_VERSION {
        Ok(de_settings)
    } else {
        Err(err::Error::Deserialize(
            format!("Wrong settings file version {}. Should be {}",
                    de_settings.version, SETTINGS_VERSION)
        ))
    }
}

/// Load `DeSettings` from settings file.
///
/// Tries to create the settings file if it's not available and loads the
/// default settings.
fn load_de_settings() -> DeSettings {

    let default_settings_str = include_str!("default_settings.toml");

    if let Some(proj_dirs) = directories::ProjectDirs::from("ar.com.mbernardi", "", "noaa-apt") {

        let filename = proj_dirs.config_dir().join("settings.toml");

        match parse_from_file(&filename) {
            Ok(de_settings) => {
                return de_settings
            },
            Err(e) => {
                println!("Error loading settings file {:?}: {}", filename, e);

                let _result = std::fs::create_dir_all(proj_dirs.config_dir());

                if filename.exists() {

                    let mut dest = filename.clone();
                    dest.set_extension("OLD");
                    println!(
                        "Outdated or corrupted settings file, moving to {:?} \
                        and saving default settings file on {:?}",
                        &dest, &filename);

                    if let Err(e) = std::fs::rename(&filename, &dest) {
                        println!("Unable to move {:?} to {:?}: {}",
                                 &dest, &filename, e);
                    }
                }

                if let Ok(mut file) = std::fs::File::create(&filename) {

                    println!("Saving default settings to {:?}", &filename);
                    if let Err(e) = file.write_all(default_settings_str.as_bytes()) {
                        println!("Unable to write: {}", e);
                    }

                } else {
                    println!(
                        "Could not open or create settings file {:?}, using default settings",
                        &filename,
                    );
                }
                return toml::from_str(default_settings_str).expect(
                    "Failed to parse default settings"
                );
            }
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

    let mut arg_input_filename: Option<PathBuf> = None;
    let mut arg_debug = false;
    let mut arg_quiet = false;
    let mut arg_wav_steps = false;
    let mut arg_export_resample_filtered = false;
    let mut arg_sync = true;
    let mut arg_contrast_adjustment: Option<String> = None;
    let mut arg_profile: Option<String> = None;
    let mut arg_print_version = false;
    let mut arg_output_filename: Option<PathBuf> = None;
    let mut arg_resample_output: Option<u32> = None;
    let mut arg_sat: Option<String> = None;
    let mut arg_start_time: Option<String> = None;
    let mut arg_tle_filename: Option<String> = None;
    let mut arg_map: Option<String> = None;
    let mut arg_rotate: Option<String> = None;
    let mut arg_rotate_deprecated = false;
    {
        let mut parser = argparse::ArgumentParser::new();
        parser.set_description("Decode NOAA APT images from WAV files. Run \
                               without arguments to launch the GUI");
        parser.refer(&mut arg_input_filename)
            .add_argument("input_filename", argparse::StoreOption,
            "Input WAV file.");
        parser.refer(&mut arg_output_filename)
            .add_option(&["-o", "--output"], argparse::StoreOption,
            "Set output path. When decoding images the default is \
            './output.png', when resampling the default is './output.wav'.")
            .metavar("FILENAME");
        parser.refer(&mut arg_print_version)
            .add_option(&["-v", "--version"], argparse::StoreTrue,
            "Show version and quit.");
        parser.refer(&mut arg_debug)
            .add_option(&["-d", "--debug"], argparse::StoreTrue,
            "Print debugging messages.");
        parser.refer(&mut arg_quiet)
            .add_option(&["-q", "--quiet"], argparse::StoreTrue,
            "Don't print info messages.");
        parser.refer(&mut arg_resample_output)
            .add_option(&["-r", "--resample"], argparse::StoreOption,
            "Resample WAV file to a given sample rate, no APT image will be \
            decoded.")
            .metavar("SAMPLE_RATE");
        parser.refer(&mut arg_sync)
            .add_option(&["--no-sync"], argparse::StoreFalse,
            "Disable syncing, useful when the sync frames are noisy and the \
            syncing attempts do more harm than good.");
        parser.refer(&mut arg_contrast_adjustment)
            .add_option(&["-c", "--contrast"], argparse::StoreOption,
            "Contrast adjustment method for decode. Possible values: \
            \"98_percent\" (default), \"telemetry\" or \"disable\".")
            .metavar("METHOD");
        parser.refer(&mut arg_sat)
            .add_option(&["-s", "--sat"], argparse::StoreOption,
            "Enable orbit calculations and indicate satellite name. Possible \
            values \"noaa_15\", \"noaa_18\" or \"noaa_19\". If no --tle was \
            provided and the current cached TLE is older than a week, a new \
            weather.txt TLE from celestrak.com will be downloaded and cached.")
            .metavar("SATELLITE");
        parser.refer(&mut arg_map)
            .add_option(&["-m", "--map"], argparse::StoreOption,
            "Enable map overlay, a --sat must be provided. Possible values: \
            \"yes\" or \"no\".")
            .metavar("MAP_MODE");
        parser.refer(&mut arg_rotate)
            .add_option(&["-R", "--rotate"], argparse::StoreOption,
            "Rotate image, useful for South to North passes where the raw image \
            is received upside-down. Possible values: \"auto\", \"yes\", \
            \"no\" (default). If using \"auto\", a --sat must be provided. In \
            that case the program uses orbit calculations and reception time to \
            determine if the pass was South to North.")
            .metavar("METHOD");
        parser.refer(&mut arg_start_time)
            .add_option(&["-t", "--start-time"], argparse::StoreOption,
            "Provide recording start time, used for orbit calculations. Use \
            RFC 3339 format which includes date, time and timezone, e.g. \
            \"1996-12-19T16:39:57-08:00\". If this option is not provided, it \
            will be inferred from the filename or from the file modification \
            timestamp.");
        parser.refer(&mut arg_tle_filename)
            .add_option(&["-T", "--tle"], argparse::StoreOption,
            "Load TLE from given path. Very useful when decoding old images and \
            if you have a TLE from around that date.");
        parser.refer(&mut arg_profile)
            .add_option(&["-p", "--profile"], argparse::StoreOption,
            "Profile to use, values loaded from settings file. Possible values: \
            \"standard\", \"fast\" or \"slow\".");
        parser.refer(&mut arg_wav_steps)
            .add_option(&["--wav-steps"], argparse::StoreTrue,
            "Export a WAV for every step of the decoding process for debugging, \
            the files will be located on the current folder, named \
            {number}_{description}.wav");
        parser.refer(&mut arg_export_resample_filtered)
            .add_option(&["--export-resample-filtered"], argparse::StoreTrue,
            "Export a WAV for the expanded and filtered signal on the resampling
            step. Very expensive operation, can take several GiB of both RAM and
            disk. --wav-steps should be set.");
        parser.refer(&mut arg_rotate_deprecated)
            .add_option(&["--rotate-image"], argparse::StoreTrue,
            "Deprecated. Use --rotate instead");
        parser.parse_args_or_exit();
    }

    // Open settings file
    let de_settings = load_de_settings();

    // Now there is a lot of code decide and merge commandline arguments and
    // settings

    // Select commandline profile, otherwise load default
    let profile: String = arg_profile.unwrap_or(de_settings.profiles.default_profile);
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

    let verbosity = if arg_debug {
        log::Level::Debug
    } else if arg_quiet {
        log::Level::Warn
    } else {
        log::Level::Info
    };

    if arg_print_version {
        return (check_updates, verbosity, Mode::Version);
    }

    // Build Settings struct

    let settings = Settings {
        export_wav: arg_wav_steps,
        export_resample_filtered: arg_export_resample_filtered,
        work_rate: profile.work_rate as u32,
        resample_atten: profile.resample_atten as f32,
        resample_delta_freq: profile.resample_delta_freq as f32,
        resample_cutout: profile.resample_cutout as f32,
        demodulation_atten: profile.demodulation_atten as f32,
        wav_resample_atten: profile.wav_resample_atten as f32,
        wav_resample_delta_freq: profile.wav_resample_delta_freq as f32,
        prefer_timestamps: de_settings.timestamps.prefer_timestamps,
        filename_formats: de_settings.timestamps.filenames,
        filename_timezone: de_settings.timestamps.timezone,
        default_countries_color: de_settings.map_overlay.default_countries_color,
        default_states_color: de_settings.map_overlay.default_states_color,
        default_lakes_color: de_settings.map_overlay.default_lakes_color,
    };

    // If set, then the program will be used as a command-line one, otherwise we
    // open the GUI
    if let Some(input_filename) = arg_input_filename {

        // If set, we are resampling, otherwise we are decoding
        if let Some(rate) = arg_resample_output {

            return (check_updates, verbosity, Mode::Resample {
                settings,
                input_filename,
                output_filename: arg_output_filename.unwrap_or_else(
                    || PathBuf::from("./output.wav")),
                output_rate: rate,
            });

        // resample_output option not set, decode WAV file
        } else {

            let contrast_adjustment: Contrast = match arg_contrast_adjustment.as_deref() {
                Some("telemetry") => Contrast::Telemetry,
                Some("disable") => Contrast::MinMax,
                Some("98_percent") | None => Contrast::Percent(0.98),
                Some(_) => {
                    println!("Invalid contrast adjustment argument");
                    std::process::exit(0);
                },
            };

            let mut rotate: Rotate = match arg_rotate.as_deref() {
                Some("auto") => Rotate::Orbit,
                Some("yes") => Rotate::Yes,
                Some("no") => Rotate::No,
                Some(_) => {
                    println!("Invalid rotate argument");
                    std::process::exit(0);
                },
                None => Rotate::No,
            };

            if arg_rotate_deprecated {
                rotate = Rotate::Yes;
            }

            // A satellite name was provided
            if let Some(_) = arg_sat {

                let sat_name: SatName = match arg_sat.as_deref() {
                    Some("noaa_15") => SatName::Noaa15,
                    Some("noaa_18") => SatName::Noaa18,
                    Some("noaa_19") => SatName::Noaa19,
                    Some(_) => {
                        println!("Invalid satellite name");
                        std::process::exit(0);
                    },
                    None => unreachable!(),
                };

                let custom_tle: Option<String> = match arg_tle_filename {
                    Some(s) => {
                        let path = PathBuf::from(s);
                        let mut file = File::open(path).unwrap_or_else(|e| {
                            println!("Could not open custom TLE file: {}", e);
                            std::process::exit(0);
                        });
                        let mut tle = String::new();
                        if let Err(e) = file.read_to_string(&mut tle) {
                            println!("Could not read custom TLE file: {}", e);
                            std::process::exit(0);
                        }

                        Some(tle)
                    },
                    None => None,
                };

                let ref_time: RefTime = match arg_start_time {
                    Some(s) => {
                        RefTime::Start(
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .unwrap_or_else(|e| {
                                    println!("Could not parse date and time given: {}", e);
                                    std::process::exit(0);
                                })
                            .into()
                        )
                    },
                    None => {
                        misc::infer_ref_time(&settings, &input_filename)
                            .unwrap_or_else(|e| {
                                println!("Could not infer recording date and \
                                         time from file: {}", e);
                                std::process::exit(0);
                            })
                    }
                };

                let draw_map = match arg_map.as_deref() {
                    Some("yes") => Some(MapSettings {
                        // TODO
                        yaw: 0.,
                        hscale: 1.,
                        vscale: 1.,
                        countries_color: settings.default_countries_color,
                        states_color: settings.default_states_color,
                        lakes_color: settings.default_lakes_color,
                    }),
                    Some("no") => None,
                    Some(_) => {
                        println!("Invalid map argument");
                        std::process::exit(0);
                    },
                    None => None,
                };

                let orbit_settings = OrbitSettings {
                    sat_name,
                    custom_tle,
                    ref_time,
                    draw_map,
                };

                return (check_updates, verbosity, Mode::Decode {
                    settings,
                    input_filename,
                    output_filename: arg_output_filename.unwrap_or_else(
                        || PathBuf::from("./output.wav")),
                    sync: arg_sync,
                    contrast_adjustment,
                    rotate,
                    orbit_settings: Some(orbit_settings),
                });

            // A satellite name was not provided
            } else {

                return (check_updates, verbosity, Mode::Decode {
                    settings,
                    input_filename,
                    output_filename: arg_output_filename.unwrap_or_else(
                        || PathBuf::from("./output.wav")),
                    sync: arg_sync,
                    contrast_adjustment,
                    rotate,
                    orbit_settings: None,
                });

            }
        }

    // Input filename not set, launch GUI
    } else {

        return (check_updates, verbosity, Mode::Gui { settings } );

    }

}
