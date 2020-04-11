//! Functions to decode, process, resample, etc.

use std::path::PathBuf;

use gio::prelude::*;
use gtk::prelude::*;
use log::error;

use crate::context::Context;
use crate::dsp::{Signal, Rate};
use crate::err;
use crate::noaa_apt::{self, Image, Contrast, Rotate};
use super::misc;
use super::state::{
    GuiState, borrow_state, borrow_state_mut, set_state,
    Widgets, borrow_widgets, set_widgets
};

/// Get values from widgets, decode and update widgets.
///
/// Starts another working thread.Sets buttons as not sensitive until the
/// decoding finishes, etc. Saves the result on the GUI state.
pub fn decode() {

    // Create callbacks

    let callback = |result: err::Result<Signal>| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                widgets.dec_decode_button.set_sensitive(true);
                widgets.main_start_button.set_sensitive(true);
                match &result {
                    Ok(signal) => {
                        misc::set_progress(1., "Decoded");
                        widgets.p_process_button.set_sensitive(true);
                        borrow_state_mut(|state| {
                            state.decoded_signal = Some(signal.clone());
                            state.processed_image = None;
                            // TODO
                            // let pixbuf = gdk_pixbuf::Pixbuf::new_from_file(Path::new("./res/icon.png"))
                                // .expect("Couldn't load ./res/icon.png");
                            // widgets.img_image.set_from_pixbuf(Some(&pixbuf));
                        });
                    },
                    Err(e) => {
                        misc::set_progress(1., "Error");
                        misc::show_info(gtk::MessageType::Error, &e.to_string());
                        error!("{}", e);
                        borrow_state_mut(|state| {
                            state.decoded_signal = None;
                            state.processed_image = None;
                        });
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

        widgets.info_revealer.set_reveal_child(false);
        widgets.dec_decode_button.set_sensitive(false);
        widgets.sav_save_button.set_sensitive(false);
        widgets.p_process_button.set_sensitive(false);
        widgets.main_start_button.set_sensitive(false);

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
                widgets.main_start_button.set_sensitive(true);
                match &result {
                    Ok(image) => {
                        misc::set_progress(1., "Processed");
                        widgets.sav_save_button.set_sensitive(true);
                        borrow_state_mut(|state| {
                            state.processed_image = Some(image.clone());
                            // TODO
                            let flat_image = image.as_flat_samples();
                            let pixbuf = gdk_pixbuf::Pixbuf::new_from_bytes(
                                &glib::Bytes::from(&flat_image.samples),
                                gdk_pixbuf::Colorspace::Rgb,
                                false, // has_alpha
                                8, // bits_per_sample
                                flat_image.layout.width as i32,
                                flat_image.layout.height as i32,
                                flat_image.layout.height_stride as i32,
                            );
                            widgets.img_image.set_from_pixbuf(Some(&pixbuf));
                        });
                    },
                    Err(e) => {
                        misc::set_progress(1., "Error");
                        misc::show_info(gtk::MessageType::Error, &e.to_string());
                        error!("{}", e);
                        borrow_state_mut(|state| {
                            state.processed_image = None;
                        });
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

        widgets.info_revealer.set_reveal_child(false);
        widgets.dec_decode_button.set_sensitive(false);
        widgets.sav_save_button.set_sensitive(false);
        widgets.p_process_button.set_sensitive(false);
        widgets.main_start_button.set_sensitive(false);

        // Read widgets

        // let input_filename: PathBuf = match widgets.dec_input_chooser.get_filename() {
            // Some(path) => path,
            // None => {
                // callback(Err(err::Error::Internal(
                    // "Select input file".to_string())));
                // return;
            // }
        // };
//
        // let sync = widgets.dec_sync_check.get_active();
//
        let wav_steps = widgets.dec_wav_steps_check.get_active();

        let resample_step = widgets.dec_resample_step_check.get_active();

//
        let settings = borrow_state(|state| state.settings.clone());

        let signal = borrow_state(|state| state.decoded_signal.clone())
            .expect("TODO: No decoded signal");

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
                Contrast::MinMax,
                Rotate::Yes,
                None,
            ));
        });
    });
}
