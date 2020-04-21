//! Some helper functions.

use gio::prelude::*;
use gtk::prelude::*;

use crate::err;
use crate::misc;
use super::state::{
    GuiState, borrow_state, borrow_state_mut, set_state,
    Widgets, borrow_widgets, set_widgets
};

/// Set progress of ProgressBar
pub fn set_progress(fraction: f32, description: &str) {
    borrow_widgets(|widgets| {
        widgets.main_progress_bar.set_fraction(fraction as f64);
        widgets.main_progress_bar.set_text(Some(description));
    });
}

/// Show InfoBar with custom message.
pub fn show_info(message_type: gtk::MessageType, text: &str) {
    borrow_widgets(|widgets| {
        match message_type {
            gtk::MessageType::Info =>
                widgets.info_label.set_markup(
                    text
                ),
            gtk::MessageType::Warning =>
                widgets.info_label.set_markup(
                    format!("<b>Warning: {}</b>", text).as_str()
                ),
            gtk::MessageType::Error =>
                widgets.info_label.set_markup(
                    format!("<b>Error: {}</b>", text).as_str()
                ),
            _ =>
                unreachable!(),
        }

        widgets.info_bar.set_message_type(message_type);
        widgets.info_revealer.set_reveal_child(true);
    });
}

/// Check for updates on another thread and show the result on the info_bar.
///
/// Provide current version.
pub fn check_updates_and_show(version: &'static str) {
    let callback = move |result| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                match result {
                    Some((true, ref latest)) => {
                        show_info(
                            gtk::MessageType::Info,
                            format!("Version \"{}\" available for download!", latest).as_str(),
                        );
                    },
                    Some((false, _)) => {}, // Do nothing, already on latest version
                    None => {
                        show_info(
                            gtk::MessageType::Info,
                            "Error checking for updates, do you have an internet connection?",
                        );
                    },
                }
            });
            Continue(false)
        });
    };

    std::thread::spawn(move || {
        callback(misc::check_updates(version));
    });
}

/// Open webpage in browser.
///
/// GTK provides `gtk::show_uri` but it only works for `http://` targets when
/// gvfs is present on the system. So in Windows I use the Windows API through
/// the `winapi` crate.
///
/// `url` should include `http://`.
///
/// References:
/// - https://github.com/gameblabla/pokemini/blob/37e3324ebf481a5f6ece725ae5c052332e3b84d1/sourcex/HelpSupport.c
/// - https://gitlab.com/varasev/parity-ethereum/blob/0a170efaa5ee9a1df824630db2a997ad52f6ef57/parity/url.rs
/// - https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-shellexecutea
#[allow(unused_variables)]
pub fn open_in_browser<W>(window: &W, url: &str) -> err::Result<()>
where W: glib::object::IsA<gtk::Window>
{
    #[cfg(windows)]
    {
        use std::ffi::CString;
        use std::ptr;

        unsafe {
            winapi::um::shellapi::ShellExecuteA(
                ptr::null_mut(), // Window
                CString::new("open").unwrap().as_ptr(), // Action
                CString::new(url).unwrap().as_ptr(), // URL
                ptr::null_mut(), // Parameters
                ptr::null_mut(), // Working directory
                winapi::um::winuser::SW_SHOWNORMAL // How to show the window
            );
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        gtk::show_uri(
            window.clone().upcast::<gtk::Window>().get_screen().as_ref(),
            url,
            gtk::get_current_event_time(),
        ).or_else(|_| Err(err::Error::Internal("Could not open browser".to_string())))
    }
}

pub fn update_image() {
    borrow_widgets(|widgets| {

        let pixbuf = match borrow_state_mut(|state| state.processed_image.clone()) {
            Some(image) => {
                let flat_image = image.as_flat_samples();
                gdk_pixbuf::Pixbuf::new_from_bytes(
                    &glib::Bytes::from(&flat_image.samples),
                    gdk_pixbuf::Colorspace::Rgb,
                    false, // has_alpha
                    8, // bits_per_sample
                    flat_image.layout.width as i32,
                    flat_image.layout.height as i32,
                    flat_image.layout.height_stride as i32,
                )
            }
            None => {
                widgets.img_def_pixbuf.clone()
            }
        };

        if widgets.img_size_toggle.get_active() {
            widgets.img_image.set_from_pixbuf(Some(&pixbuf));
        } else {
            let img_width = pixbuf.get_width() as f32;
            let img_height = pixbuf.get_height() as f32;
            let max_width = widgets.img_viewport.get_allocated_width() as f32;
            let max_height = widgets.img_viewport.get_allocated_height() as f32;

            let scale = f32::min(max_width / img_width, max_height / img_height);

            if scale < 1. {
                let w = (img_width * scale).floor() as i32;
                let h = (img_height * scale).floor() as i32;
                let p = pixbuf.scale_simple(w, h, gdk_pixbuf::InterpType::Bilinear).expect("TODO");
                widgets.img_image.set_from_pixbuf(Some(&p));
            } else {
                // Do not make images bigger than original size
                widgets.img_image.set_from_pixbuf(Some(&pixbuf));
            }
        }
    });
}

/*
pub fn read_timestamp() -> err::Result<()> {
    borrow_widgets(|widgets| {

        let calendar = widgets.calendar.as_ref().expect("Couldn't get calendar");
        let hour_spinner = widgets.hour_spinner.as_ref().expect("Couldn't get hour_spinner");
        let minute_spinner = widgets.minute_spinner.as_ref().expect("Couldn't get minute_spinner");
        let second_spinner = widgets.second_spinner.as_ref().expect("Couldn't get second_spinner");

        let input_filename: PathBuf = widgets
            .input_file_chooser
            .get_filename() // Option<std::path::PathBuf>
            .ok_or_else(|| err::Error::Internal("Select input file".to_string()))?;

        let timestamp = misc::read_timestamp(&input_filename)?;
        let datetime = chrono::Local.timestamp(timestamp, 0); // 0 milliseconds

        // GTK counts months from 0 to 11. Years and days are fine
        calendar.select_month(datetime.month0() as u32, datetime.year() as u32);
        calendar.select_day(datetime.day());
        hour_spinner.set_value(datetime.hour() as f64);
        minute_spinner.set_value(datetime.minute() as f64);
        second_spinner.set_value(datetime.second() as f64);

        show_info(&widgets, gtk::MessageType::Info, "Loaded timestamp from file");

        Ok(())
    })
}

pub fn write_timestamp() -> err::Result<()> {
    borrow_widgets(|widgets| {

        let calendar = widgets.calendar.as_ref().expect("Couldn't get calendar");
        let hour_spinner = widgets.hour_spinner.as_ref().expect("Couldn't get hour_spinner");
        let minute_spinner = widgets.minute_spinner.as_ref().expect("Couldn't get minute_spinner");
        let second_spinner = widgets.second_spinner.as_ref().expect("Couldn't get second_spinner");

        let output_filename: PathBuf = widgets
            .output_entry
            .get_text()
            .map(|text| PathBuf::from(text.as_str()))
            .ok_or_else(|| err::Error::Internal("Could not get decode_output_entry text".to_string()))?;

        let hour = hour_spinner.get_value_as_int();
        let minute = minute_spinner.get_value_as_int();
        let second = second_spinner.get_value_as_int();
        let (year, month, day) = calendar.get_date();

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
            chrono::offset::LocalResult::None =>
                Err(err::Error::Internal("Invalid date or time".to_string())),
            chrono::offset::LocalResult::Single(dt) =>
                Ok(dt),
            chrono::offset::LocalResult::Ambiguous(_, _) =>
                Err(err::Error::Internal("Ambiguous date or time".to_string())),
        }?;

        misc::write_timestamp(datetime.timestamp(), &output_filename)?;

        show_info(&widgets, gtk::MessageType::Info, "Timestamp written to file");

        Ok(())
    })
}
*/
