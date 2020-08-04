//! Functions to decode, process, resample, etc.
//!
//! These functions are called when the user presses the most important buttons
//! (decode, resample, process, save, etc.).

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use chrono::offset::TimeZone;
use chrono::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;
use log::error;

use crate::context::Context;
use crate::dsp::{Signal, Rate};
use crate::err;
use crate::noaa_apt::{self, Image, Contrast, Rotate, RefTime, SatName, OrbitSettings, MapSettings};
use super::misc;
use super::state::{borrow_state, borrow_state_mut, borrow_widgets};

/// Get values from widgets, decode and update widgets.
///
/// Starts another working thread.Sets buttons as not sensitive until the
/// decoding finishes, etc. Saves the result on the GUI state.
pub fn decode() {

    // Called when decoding finishes
    let callback = |result: err::Result<Signal>| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {

                widgets.dec_decode_button.set_sensitive(true);
                match &result {
                    Ok(signal) => {
                        misc::set_progress(1., "Decoded");
                        widgets.p_process_button.set_sensitive(true);
                        borrow_state_mut(|state| {
                            state.decoded_signal = Some(signal.clone());
                            state.processed_image = None;
                        });
                        misc::update_image();

                        // Read start time from file and update widgets
                        //
                        let input_filename: PathBuf =
                            match widgets.dec_input_chooser.get_filename()
                        {
                            Some(path) => path,
                            None => {
                                misc::show_info(gtk::MessageType::Info,
                                    "Could not infer recording start date and \
                                    time. Set it manually. No input file?");

                                return;
                            }
                        };

                        let settings = borrow_state(|state| state.settings.clone());

                        match crate::misc::infer_time_sat(&settings, &input_filename) {
                            Ok((RefTime::Start(time), sat_name)) => {
                                widgets.p_ref_time_combo.set_active_id(Some("start"));
                                let local_time = time.with_timezone(&chrono::Local);
                                // GTK counts months from 0 to 11. Years and days are fine
                                widgets.p_calendar.select_month(
                                    local_time.month0() as u32, local_time.year() as u32);
                                widgets.p_calendar.select_day(local_time.day());
                                widgets.p_hs_spinner.set_value(local_time.hour() as f64);
                                widgets.p_min_spinner.set_value(local_time.minute() as f64);
                                widgets.p_sec_spinner.set_value(local_time.second() as f64);

                                match sat_name {
                                    SatName::Noaa15 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_15")),
                                    SatName::Noaa18 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_18")),
                                    SatName::Noaa19 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_19")),
                                };
                            },
                            Ok((RefTime::End(time), sat_name)) => {
                                widgets.p_ref_time_combo.set_active_id(Some("end"));
                                let local_time = time.with_timezone(&chrono::Local);
                                // GTK counts months from 0 to 11. Years and days are fine
                                widgets.p_calendar.select_month(
                                    local_time.month0() as u32, local_time.year() as u32);
                                widgets.p_calendar.select_day(local_time.day());
                                widgets.p_hs_spinner.set_value(local_time.hour() as f64);
                                widgets.p_min_spinner.set_value(local_time.minute() as f64);
                                widgets.p_sec_spinner.set_value(local_time.second() as f64);

                                match sat_name {
                                    SatName::Noaa15 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_15")),
                                    SatName::Noaa18 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_18")),
                                    SatName::Noaa19 => widgets.p_satellite_combo
                                        .set_active_id(Some("noaa_19")),
                                };
                            },
                            Err(e) => {
                                misc::show_info(gtk::MessageType::Info,
                                    format!("Could not infer recording time and
                                        satellite. Set them manually: {}", e).as_str()
                                );
                            }
                        };
                    },
                    Err(e) => {
                        misc::set_progress(1., "Error");
                        misc::show_info(gtk::MessageType::Error, &e.to_string());
                        error!("{}", e);
                        borrow_state_mut(|state| {
                            state.decoded_signal = None;
                            state.processed_image = None;
                        });
                        misc::update_image();
                    },
                }
            });
            Continue(false)
        });
    };
    let progress_callback = |progress, description: String| {
        glib::idle_add(move || {
            misc::set_progress(progress, &description);
            Continue(false)
        });
    };

    borrow_widgets(|widgets| {

        misc::set_progress(0., "Decoding");
        widgets.info_revealer.set_reveal_child(false);
        widgets.dec_decode_button.set_sensitive(false);
        widgets.sav_save_button.set_sensitive(false);
        widgets.p_process_button.set_sensitive(false);

        // Read widgets

        let input_filename: PathBuf = match widgets.dec_input_chooser.get_filename() {
            Some(path) => path,
            None => {
                callback(Err(err::Error::Internal(
                    "Select input file".to_string())));
                return;
            }
        };

        let sync = widgets.dec_sync_check.get_active();

        let wav_steps = widgets.dec_wav_steps_check.get_active();

        let resample_step = widgets.dec_resample_step_check.get_active();

        let settings = borrow_state(|state| state.settings.clone());

        std::thread::spawn(move || {

            let (signal, rate) = match noaa_apt::load(&input_filename) {
                Ok(result) => result,
                Err(e) => {
                    callback(Err(e));
                    return;
                }
            };

            let mut context = Context::decode(
                progress_callback,
                Rate::hz(settings.work_rate),
                Rate::hz(noaa_apt::FINAL_RATE),
                wav_steps,
                resample_step,
            );
            callback(noaa_apt::decode(
                &mut context,
                &settings,
                &signal,
                rate,
                sync
            ));
        });
    });
}

/// Get values from widgets, process and update widgets.
///
/// Starts another working thread.Sets buttons as not sensitive until the
/// decoding finishes, etc. Saves the result on the GUI state.
pub fn process() {

    // Create callbacks

    let callback = |result: err::Result<Image>| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                widgets.dec_decode_button.set_sensitive(true);
                widgets.p_process_button.set_sensitive(true);
                match &result {
                    Ok(image) => {
                        misc::set_progress(1., "Processed");
                        widgets.sav_save_button.set_sensitive(true);
                        borrow_state_mut(|state| {
                            state.processed_image = Some(image.clone());
                        });
                        misc::update_image();
                    },
                    Err(e) => {
                        misc::set_progress(1., "Error");
                        misc::show_info(gtk::MessageType::Error, &e.to_string());
                        error!("{}", e);
                        borrow_state_mut(|state| {
                            state.processed_image = None;
                        });
                        misc::update_image();
                    },
                }
            });
            Continue(false)
        });
    };
    let progress_callback = |progress, description: String| {
        glib::idle_add(move || {
            misc::set_progress(progress, &description);
            Continue(false)
        });
    };

    borrow_widgets(|widgets| {

        misc::set_progress(0., "Processing");
        widgets.info_revealer.set_reveal_child(false);
        widgets.dec_decode_button.set_sensitive(false);
        widgets.sav_save_button.set_sensitive(false);
        widgets.p_process_button.set_sensitive(false);

        // Read widgets

        let wav_steps = widgets.dec_wav_steps_check.get_active();

        let resample_step = widgets.dec_resample_step_check.get_active();

        let contrast_adjustment: Contrast = match widgets.p_contrast_combo
            .get_active_id()
            .as_ref()
            .map(|s| s.as_str())
            {
                Some("98_percent") => Contrast::Percent(0.98),
                Some("telemetry") => Contrast::Telemetry,
                Some("histogram") => Contrast::Histogram,
                Some("minmax") => Contrast::MinMax,
                Some(id) => {
                    callback(Err(err::Error::Internal(
                        format!("Unknown contrast adjustment \"{}\"", id)
                    )));
                    return;
                },
                None => {
                    callback(Err(err::Error::Internal(
                        "Select contrast adjustment".to_string()
                    )));
                    return;
                },
            };

        let rotate: Rotate = match widgets.p_rotate_combo
            .get_active_id()
            .as_ref()
            .map(|s| s.as_str())
            {
                Some("auto") => Rotate::Orbit,
                Some("no") => Rotate::No,
                Some("yes") => Rotate::Yes,
                Some(id) => {
                    callback(Err(err::Error::Internal(
                        format!("Unknown rotation \"{}\"", id)
                    )));
                    return;
                },
                None => {
                    callback(Err(err::Error::Internal(
                        "Select rotation option".to_string()
                    )));
                    return;
                },
            };

        let sat_name: SatName = match widgets.p_satellite_combo
            .get_active_id()
            .as_ref()
            .map(|s| s.as_str())
            {
                Some("noaa_15") => SatName::Noaa15,
                Some("noaa_18") => SatName::Noaa18,
                Some("noaa_19") => SatName::Noaa19,
                Some(id) => {
                    callback(Err(err::Error::Internal(
                        format!("Unknown satellite \"{}\"", id)
                    )));
                    return;
                },
                None => {
                    callback(Err(err::Error::Internal(
                        "Select satellite option".to_string()
                    )));
                    return;
                },
            };

        // Custom TLE

        let custom_tle = match widgets.p_custom_tle_check.get_active() {
            false => {
                None
            },
            true => {
                match widgets.p_custom_tle_chooser.get_filename() {
                    Some(path) => {
                        let mut file = match File::open(path) {
                            Ok(f) => f,
                            Err(e) => {
                                callback(Err(err::Error::Internal(
                                    format!("Could not open custom TLE file: {}", e))));
                                return;
                            },
                        };
                        let mut tle = String::new();
                        if let Err(e) = file.read_to_string(&mut tle) {
                            callback(Err(err::Error::Internal(
                                format!("Could not read custom TLE file: {}", e))));
                            return;
                        }

                        Some(tle)
                    },
                    None => {
                        callback(Err(err::Error::Internal(
                            "Select custom TLE input file".to_string())));
                        return;
                    }
                }
            }
        };

        // Get date and time

        let hour = widgets.p_hs_spinner.get_value_as_int();
        let minute = widgets.p_min_spinner.get_value_as_int();
        let second = widgets.p_sec_spinner.get_value_as_int();
        let (year, month, day) = widgets.p_calendar.get_date();

        let time = match chrono::Local
            .ymd_opt(year as i32, month + 1, day)
            .and_hms_opt(hour as u32, minute as u32, second as u32)
        {
            chrono::offset::LocalResult::None => {
                callback(Err(err::Error::Internal(
                    "Invalid date or time".to_string()
                )));
                return;
            },
            chrono::offset::LocalResult::Single(dt) =>
                dt.with_timezone(&chrono::Utc), // Convert to UTC
            chrono::offset::LocalResult::Ambiguous(_, _) => {
                callback(Err(err::Error::Internal(
                    "Ambiguous date or time".to_string()
                )));
                return;
            }
        };

        let ref_time = match widgets.p_ref_time_combo
            .get_active_id()
            .as_ref()
            .map(|s| s.as_str())
        {
            Some("start") => RefTime::Start(time),
            Some("end") => RefTime::End(time),
            Some(_) | None => {
                callback(Err(err::Error::Internal(
                    "Select if provided time is recording start or end".to_string()
                )));
                return;
            },
        };

        // Map settings

        let draw_map = match widgets.p_overlay_check.get_active() {
            false =>
                None,
            true => {
                use std::f64::consts::PI;
                let rgba_to_tuple = |rgba: gdk::RGBA| -> (u8, u8, u8, u8) {
                    (
                        (rgba.red * 255.).max(0.).min(255.) as u8,
                        (rgba.green * 255.).max(0.).min(255.) as u8,
                        (rgba.blue * 255.).max(0.).min(255.) as u8,
                        (rgba.alpha * 255.).max(0.).min(255.) as u8,
                    )
                };
                Some(MapSettings {
                    // Convert degrees to radians
                    yaw: widgets.p_yaw_spinner.get_value() * PI / 180.,
                    // Convert percent to fraction
                    hscale: widgets.p_hscale_spinner.get_value() / 100.,
                    vscale: widgets.p_vscale_spinner.get_value() / 100.,
                    countries_color: rgba_to_tuple(widgets.p_countries_color.get_rgba()),
                    states_color: rgba_to_tuple(widgets.p_states_color.get_rgba()),
                    lakes_color: rgba_to_tuple(widgets.p_lakes_color.get_rgba())
                })
            },
        };

        // Compose OrbitSettings

        let orbit = OrbitSettings {
            sat_name,
            custom_tle,
            ref_time,
            draw_map,
        };

        // Get settings and signal from state

        let settings = borrow_state(|state| state.settings.clone());

        let signal = match borrow_state(|state| state.decoded_signal.clone()) {
            Some(s) => s,
            None => {
                callback(Err(err::Error::Internal("No decoded image?".to_string())));
                return;
            },
        };

        std::thread::spawn(move || {

            let mut context = Context::decode(
                progress_callback,
                Rate::hz(settings.work_rate),
                Rate::hz(noaa_apt::FINAL_RATE),
                wav_steps,
                resample_step,
            );
            callback(noaa_apt::process(
                &mut context,
                &signal,
                contrast_adjustment,
                rotate,
                Some(orbit),
            ));
        });
    });
}

/// Get values from widgets, save and update widgets.
///
/// Takes image from the GUI state.
pub fn save() {

    borrow_widgets(|widgets| {

        widgets.info_revealer.set_reveal_child(false);
        misc::set_progress(0., "Saving");

        let output_filename: PathBuf = match widgets
            .sav_output_entry
            .get_text()
            .map(|text| PathBuf::from(text.as_str()))
        {
            Some(f) => f,
            None => {
                misc::set_progress(1., "Error");
                misc::show_info(gtk::MessageType::Info,
                    "Error parsing output filename");
                error!("Error parsing output filename");

                return;
            },
        };

        if output_filename.as_os_str().is_empty() {
            misc::set_progress(1., "Error");
            misc::show_info(gtk::MessageType::Error, "Select output filename");
            error!("Select output filename");
            return;
        }

        let processed_image = match borrow_state(|state| state.processed_image.clone()) {
            Some(i) => i,
            None => {
                misc::show_info(gtk::MessageType::Info,
                    "No processed image to save?");
                error!("No processed image to save?");
                return;
            },
        };

        if let Err(e) = processed_image.save(&output_filename) {
            misc::set_progress(1., "Error");
            misc::show_info(gtk::MessageType::Info,
                &format!("Error saving image: {}", e));
            error!("Error saving image: {}", e);
        } else {
            misc::set_progress(1., "Saved");
        }
    });
}

/// Get values from widgets, resample and update widgets.
///
/// Starts another working thread. Sets buttons as not sensitive until the
/// resample finishes, etc.
pub fn resample() {

    // Called when resampling finishes
    let callback = |result: err::Result<()>| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                widgets.res_resample_button.set_sensitive(true);
                match &result {
                    Ok(()) => {
                        misc::set_progress(1., "Finished");
                    },
                    Err(e) => {
                        misc::set_progress(1., "Error");
                        misc::show_info(gtk::MessageType::Error, &e.to_string());
                        error!("{}", e);
                    },
                }
            });
            Continue(false)
        });
    };
    let progress_callback = |progress, description: String| {
        glib::idle_add(move || {
            misc::set_progress(progress, &description);
            Continue(false)
        });
    };

    borrow_widgets(|widgets| {

        misc::set_progress(0., "Resampling");
        widgets.info_revealer.set_reveal_child(false);
        widgets.res_resample_button.set_sensitive(false);

        // Read widgets

        let input_filename: PathBuf = match widgets.res_input_chooser.get_filename() {
            Some(path) => path,
            None => {
                callback(Err(err::Error::Internal(
                    "Select input file".to_string())));
                return;
            }
        };
        let output_filename: PathBuf = match widgets
            .res_output_entry
            .get_text()
            .map(|text| PathBuf::from(text.as_str()))
        {
            Some(f) => f,
            None => {
                misc::set_progress(1., "Error");
                misc::show_info(gtk::MessageType::Info,
                    "Error parsing output filename");
                error!("Error parsing output filename");

                return;
            },
        };

        let wav_steps = widgets.res_wav_steps_check.get_active();
        let resample_step = widgets.res_resample_step_check.get_active();
        let output_rate = widgets.res_rate_spinner.get_value_as_int() as u32;

        let settings = borrow_state(|state| state.settings.clone());

        std::thread::spawn(move || {

            let mut context = Context::resample(
                progress_callback,
                wav_steps,
                resample_step,
            );
            callback(noaa_apt::resample(
                &mut context,
                settings,
                &input_filename,
                &output_filename,
                output_rate,
            ));
        });
    });
}

/// Get values from widgets, timestamp and update widgets.
pub fn write_timestamp() {

    let show_error = |msg: &str| {
        misc::show_info(gtk::MessageType::Error, msg);
        error!("{}", msg);
    };

    borrow_widgets(|widgets| {
        let filename = match widgets
            .ts_write_chooser
            .get_filename()
        {
            Some(f) => f,
            None => {
                show_error("Select file to write");
                return;
            },
        };

        let hour = widgets.ts_hs_spinner.get_value_as_int();
        let minute = widgets.ts_min_spinner.get_value_as_int();
        let second = widgets.ts_sec_spinner.get_value_as_int();
        let (year, month, day) = widgets.ts_calendar.get_date();

        // Write modification timestamp to file. The filetime library uses
        // the amount of seconds from the Unix epoch (Jan 1, 1970). I ignore the
        // nanoseconds precision.
        // I use the chrono library to convert date and time to timestamp.
        // As far as I know the timestamp unix_seconds will be relative to
        // 0:00:00hs UTC.

        // GTK counts months from 0 to 11. Years and days are fine
        let datetime = match chrono::Local
            .ymd_opt(year as i32, month + 1, day)
            .and_hms_opt(hour as u32, minute as u32, second as u32)
        {
            chrono::offset::LocalResult::Single(dt) => dt,
            chrono::offset::LocalResult::None => {
                show_error("Invalid date or time");
                return;
            },
            chrono::offset::LocalResult::Ambiguous(_, _) => {
                show_error("Ambiguous date or time");
                return;
            }
        };

        match crate::misc::write_timestamp(datetime.timestamp(), &filename) {
            Ok(()) =>
                misc::show_info(gtk::MessageType::Info, "Timestamp written to file"),
            Err(e) =>
                show_error(&format!("Error writing timestamp: {}", e)),
        }
    });
}

/// Read file chooser, get timestamp and update widgets.
pub fn read_timestamp() {

    let show_error = |msg: &str| {
        misc::show_info(gtk::MessageType::Error, msg);
        error!("{}", msg);
    };

    borrow_widgets(|widgets| {
        let filename = match widgets
            .ts_read_chooser
            .get_filename()
        {
            Some(f) => f,
            None => {
                show_error("Select file to read");
                return;
            },
        };

        let timestamp = match crate::misc::read_timestamp(&filename) {
            Ok(ts) => ts,
            Err(e) => {
                show_error(&format!("Error reading timestamp: {}", e));
                return;
            }
        };
        let datetime = chrono::Local.timestamp(timestamp, 0); // 0 milliseconds

        // GTK counts months from 0 to 11. Years and days are fine
        widgets.ts_calendar.select_month(datetime.month0() as u32, datetime.year() as u32);
        widgets.ts_calendar.select_day(datetime.day());
        widgets.ts_hs_spinner.set_value(datetime.hour() as f64);
        widgets.ts_min_spinner.set_value(datetime.minute() as f64);
        widgets.ts_sec_spinner.set_value(datetime.second() as f64);

        misc::show_info(gtk::MessageType::Info, "Loaded timestamp from file");
    });
}
