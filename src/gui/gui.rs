//! GUI code.
//!
//! I'm using two threads, one for the GTK+ GUI and another one that starts when
//! decoding/resampling.
//!
//! GTK+ is not thread safe so everything GUI related is on the GTK+ thread that
//! is also the main thread. When pressing a start button, a temporary thread
//! starts for decoding/resampling.
//!
//! I never add/remove widgets during runtime, everything is created on startup
//! and I hide/show widgets if necessary. This makes things easier, otherwise
//! the code fills up with `Option<>`s and `expect()`s.
//!
//! I'm using a `GuiState` struct for keeping track of the current processed
//! image and also every Widget I'm interested in. This struct is wrapped on
//! the `RefCell` smart pointer to allow mutable access everywhere.
//!
//! When doing a callback from another thread I use `ThreadGuard`, lets you
//! `Send` the Widgets to another thread but you cant use them there (panics in
//! that case). So I use `glib::idle_add()` to execute code on the main thread
//! from another thread. In the end, we send the widgets to another thread and
//! back.

use std::env;
use std::path::Path;
use std::path::PathBuf;

use chrono::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;
use log::{debug, error, info};

use crate::config;
use crate::context::Context;
use crate::dsp::{Signal, Rate};
use crate::err;
use crate::misc;
use crate::noaa_apt::{self, Contrast};
use super::state::{
    GuiState, borrow_state, borrow_state_mut, set_state,
    Widgets, borrow_widgets, set_widgets
};


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Start GUI.
///
/// Build the window.
pub fn main(check_updates: bool, settings: config::Settings) {
    let application = gtk::Application::new(
        Some("ar.com.mbernardi.noaa-apt"),
        gio::ApplicationFlags::empty(),
    ).expect("Initialization failed");

    application.connect_startup(move |app| {
        create_window(check_updates, settings.clone(), app);
    });
    application.connect_activate(|_| {});

    application.run(&[]);
}

/// Create window
fn create_window(
    check_updates: bool,
    settings: config::Settings,
    application: &gtk::Application,
) {

    let window = gtk::ApplicationWindow::new(application);

    window.set_title("noaa-apt");
    window.set_default_size(450, -1);

    // Set WM_CLASS property. Without it, on KDE the taskbar icon is correct,
    // but for some reason the window has a stock X11 icon on the top-left
    // corner. When I set WM_CLASS the window gets the correct icon.
    // GTK docs say that this option is deprecated?
    // https://gtk-rs.org/docs/gtk/trait.GtkWindowExt.html#tymethod.set_wmclass
    window.set_wmclass("noaa-apt", "noaa-apt");

    // Load widgets from glade file and create some others

    let builder = Builder::new_from_string(include_str!("main.glade"));
    let widgets = Widgets::from_builder(&builder, &window, &application);

    // Add info_bar

    widgets.info_revealer.add(&widgets.info_bar);
    widgets.info_bar.set_show_close_button(true);
    widgets.info_bar.connect_response(|_, response| {
        if response == gtk::ResponseType::Close {
            borrow_widgets(|widgets| {
                widgets.info_revealer.set_reveal_child(false);
            });
        }
    });
    let info_content_area = widgets.info_bar
        .get_content_area()
        .expect("Couldn't get info_content_area (is None)")
        .downcast::<gtk::Box>()
        .expect("Couldn't get info_content_area (not a gtk::Box)");
    info_content_area.add(&widgets.info_label);

    // Finish adding elements

    // - window
    //     - outer_box
    //         - main_paned (everything loaded from glade file)
    //             - (everything you see on screen)
    //             - ...
    //         - info_revealer
    //             - info_bar

    widgets.outer_box.pack_start(&widgets.main_paned, true, true, 0);
    widgets.outer_box.pack_end(&widgets.info_revealer, false, false, 0);

    widgets.window.add(&widgets.outer_box);

    set_widgets(widgets.clone());

    // Init GuiState

    set_state(GuiState { settings, decoded_signal: None, processed_image: None });

    // Connect close button

    widgets.window.connect_delete_event(|_, _| {
        borrow_widgets(|widgets| {
            widgets.window.destroy();
            Inhibit(false)
        })
    });

    // Finish initial widgets configuration

    build_system_menu(&widgets);
    init_widgets(&widgets);

    // Show and check for updates

    widgets.window.show_all();

    if check_updates {
        check_updates_and_show();
    }

    info!("GUI opened");
}

/// Initialize widgets and set up them for decoding.
fn init_widgets(widgets: &Widgets) {

    dec_ready();

    // Set timezone labels

    // Create any chrono::DateTime from chrono::Local, then ignore the
    // result and only take the timezone
    let time = chrono::Local::now();
    widgets.ts_timezone_label.set_text(format!(
        "Local time\n(UTC{})",
        time.format("%:z"),
    ).as_str());
    widgets.p_timezone_label.set_text(format!(
        "Local time\n(UTC{})",
        time.format("%:z"),
    ).as_str());

    // Configure GtkEntry filechoosers for saving:
    // sav_output_entry and res_output_entry

    widgets.sav_output_entry.connect_icon_press(|entry, _, _| {
        borrow_widgets(|widgets| {
            let file_chooser = gtk::FileChooserDialog::new(
                Some("Save file as"),
                Some(&widgets.window),
                gtk::FileChooserAction::Save,
            );

            file_chooser.add_buttons(&[
                ("Ok", gtk::ResponseType::Ok),
                ("Cancel", gtk::ResponseType::Cancel),
            ]);

            if file_chooser.run() == gtk::ResponseType::Ok {
                let filename = file_chooser.get_filename()
                    .expect("Couldn't get filename");

                entry.set_text(filename.to_str().unwrap());
            }

            file_chooser.destroy();
        });
    });
    widgets.res_output_entry.connect_icon_press(|entry, _, _| {
        borrow_widgets(|widgets| {
            let file_chooser = gtk::FileChooserDialog::new(
                Some("Save file as"),
                Some(&widgets.window),
                gtk::FileChooserAction::Save,
            );

            file_chooser.add_buttons(&[
                ("Ok", gtk::ResponseType::Ok),
                ("Cancel", gtk::ResponseType::Cancel),
            ]);

            if file_chooser.run() == gtk::ResponseType::Ok {
                let filename = file_chooser.get_filename()
                    .expect("Couldn't get filename");

                entry.set_text(filename.to_str().unwrap());
            }

            file_chooser.destroy();
        });
    });

    // Configure tips to update when GtkEntry changes

    fn configure_tips(
        entry: gtk::Entry,
        folder_tip_box: gtk::Box,
        folder_tip_label: gtk::Label,
        extension_tip_label: gtk::Label,
        overwrite_tip_label: gtk::Label,
        output_filename_extension: &'static str,
    ) {
        entry.connect_changed(move |this| {
            borrow_widgets(|widgets| {

                folder_tip_box.hide();
                extension_tip_label.hide();
                overwrite_tip_label.hide();

                // Exit if no output_filename

                let output_filename = match this.get_text() {
                    None => return,
                    Some(s) => s,
                };
                if output_filename.as_str() == "" {
                    return;
                }

                // If saving in CWD

                if !output_filename.starts_with("/") {
                    match env::current_dir() {
                        Ok(cwd) => {
                            folder_tip_label.set_text(&format!("{}", cwd.display()));
                            folder_tip_label.set_tooltip_text(Some(&format!("{}", cwd.display())));
                            folder_tip_box.show();
                        },
                        Err(_) => {
                            show_info(gtk::MessageType::Error,
                                "Invalid current working directory, use \
                                an absolute output path");
                        }
                    };
                }

                // Warn missing filename extension

                if !output_filename.ends_with(output_filename_extension) {
                    extension_tip_label.set_markup(&format!(
                        "<b>Warning:</b> Missing <i>{}</i> extension in filename",
                        output_filename_extension
                    ));
                    extension_tip_label.show();
                }

                // Warn already existing file

                if Path::new(&output_filename).exists() {
                    overwrite_tip_label.show();
                }
            })
        });
    }

    configure_tips(
        widgets.sav_output_entry.clone(),
        widgets.sav_folder_tip_box.clone(),
        widgets.sav_folder_tip_label.clone(),
        widgets.sav_extension_tip_label.clone(),
        widgets.sav_overwrite_tip_label.clone(),
        ".png",
    );
    configure_tips(
        widgets.res_output_entry.clone(),
        widgets.res_folder_tip_box.clone(),
        widgets.res_folder_tip_label.clone(),
        widgets.res_extension_tip_label.clone(),
        widgets.res_overwrite_tip_label.clone(),
        ".wav",
    );
}

/// Show widgets as ready for decoding/processing/saving
///
/// Called on startup and every time the user selects the decode action on the
/// menu bar.
fn dec_ready() {
    // borrow_state_mut(|state| {
        // // Reset working signal and image
//
        // state.decoded_signal = None;
        // state.processed_image = None;
    // });

    borrow_widgets(|widgets| {

        // Set enabled actions on the menu bar

        widgets.dec_action.set_enabled(false);
        widgets.res_action.set_enabled(true);
        widgets.ts_action.set_enabled(true);

        // Configure widgets

        widgets.main_stack.set_visible_child(&widgets.dec_stack_child);

        widgets.main_start_button.set_sensitive(true);
        widgets.main_start_button.set_tooltip_text(
            Some("Do everything at once, make sure to configure every tab first."));
        widgets.dec_decode_button.set_sensitive(true);
        set_progress(0., "Ready");

        // Poner texto y tooltip a main_start_button
        // Conectar
        // Reiniciar progressbar
        // Mover stack
        // TODO Reiniciar imagen

        // Connect start button

        widgets.dec_decode_button.connect_clicked(|_| decode());
    });
}

/// Show widgets as ready for resampling.
///
/// Called every time the user selects the resample action on the menu bar.
fn res_ready() {

    borrow_widgets(|widgets| {

        // Set enabled actions on the menu bar

        widgets.dec_action.set_enabled(true);
        widgets.res_action.set_enabled(false);
        widgets.ts_action.set_enabled(true);

        widgets.main_stack.set_visible_child(&widgets.res_stack_child);

    });
}

/// Show widgets as ready for resampling.
///
/// Called every time the user selects the timestamp action on the menu bar.
fn ts_ready() {

    borrow_widgets(|widgets| {

        // Set enabled actions on the menu bar

        widgets.dec_action.set_enabled(true);
        widgets.res_action.set_enabled(true);
        widgets.ts_action.set_enabled(false);

        widgets.main_stack.set_visible_child(&widgets.ts_stack_child);

        /*

        if let Mode::Timestamp = mode {

            let widgets_clone = widgets.clone();
            widgets.start_button.connect_clicked(move |_| {
                if let Err(error) = write_timestamp() {
                    show_info(&widgets_clone, gtk::MessageType::Error, error.to_string().as_str());
                    error!("{}", error);
                }
            });
            let widgets_clone = widgets.clone();
            widgets.read_button.expect("Couldn't get read_button")
                .connect_clicked(move |_|
            {
                if let Err(error) = read_timestamp() {
                    show_info(&widgets_clone, gtk::MessageType::Error, error.to_string().as_str());
                    error!("{}", error);
                }
            });
        */

    });
}

/// Build menu bar
fn build_system_menu(widgets: &Widgets) {

    // Create menu bar

    let menu_bar = gio::Menu::new();
    let help_menu = gio::Menu::new();
    let tools_menu = gio::Menu::new();

    tools_menu.append(Some("_Decode"), Some("app.decode"));
    tools_menu.append(Some("_Resample WAV"), Some("app.resample"));
    tools_menu.append(Some("_Timestamp WAV"), Some("app.timestamp"));
    menu_bar.append_submenu(Some("_Tools"), &tools_menu);

    help_menu.append(Some("_Usage"), Some("app.usage"));
    help_menu.append(Some("_Guide"), Some("app.guide"));
    help_menu.append(Some("_About"), Some("app.about"));
    menu_bar.append_submenu(Some("_Help"), &help_menu);

    widgets.application.set_menubar(Some(&menu_bar));

    // Add actions to buttons

    widgets.dec_action.connect_activate(move |_, _| dec_ready());
    widgets.res_action.connect_activate(move |_, _| res_ready());
    widgets.ts_action.connect_activate(move |_, _| ts_ready());

    widgets.application.add_action(&widgets.dec_action);
    widgets.application.add_action(&widgets.res_action);
    widgets.application.add_action(&widgets.ts_action);

    let usage = gio::SimpleAction::new("usage", None);
    let w = widgets.window.clone();
    usage.connect_activate(move |_, _| {
        open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/usage.html")
            .expect("Failed to open usage webpage");
    });
    widgets.application.add_action(&usage);

    let guide = gio::SimpleAction::new("guide", None);
    let w = widgets.window.clone();
    guide.connect_activate(move |_, _| {
        open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/guide.html")
            .expect("Failed to open usage webpage");
    });
    widgets.application.add_action(&guide);

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(|_, _| {
        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name("noaa-apt");
        dialog.set_version(Some(VERSION));
        dialog.set_authors(&["Martín Bernardi <martin@mbernardi.com.ar>"]);
        dialog.add_credit_section("Thank you",
                &[
                    "RTL-SDR.com", "pietern", "Ossi Herrala", "Arcadie Z.",
                    "Grant T. Olson", "FMighty", "Sylogista", "Peter Vogel",
                    "wren84", "Florentin314", "Gagootron", "xxretartistxx",
                    "unknownantipatriot",
                ]);

        dialog.set_website_label(Some("noaa-apt website"));
        dialog.set_website(Some("https://noaa-apt.mbernardi.com.ar/"));
        dialog.set_license_type(gtk::License::Gpl30);
        dialog.set_title("About noaa-apt");
        // dialog.set_transient_for(Some(&window)); // Not working?

        // Override links on Windows, by default GTK uses `show_uri_on_window`, see
        // documentation on `open_in_browser`
        #[cfg(windows)]
        {
            dialog.connect_activate_link(|dialog, url| {
                open_in_browser(dialog, url).expect("Failed to open link");
                return gtk::Inhibit(true); // Override `show_uri_on_window`
            });
        }

        dialog.run();
        dialog.destroy();
    });
    widgets.application.add_action(&about);
}

/*

/// Get values from widgets and decode
fn decode<F>(callback: F) where F: FnOnce(err::Result<Signal>) {

    borrow_state(|state| {

        // Read widgets

        let input_filename: PathBuf = match state.dec_input_chooser.get_filename() {
            Some(path) => path,
            None => {
                callback(Err(err::Error::Internal(
                    "Select input file".to_string())));
                return;
            }
        };

        let sync = state.dec_sync_check.get_active();

        let wav_steps = state.dec_wav_steps_check.get_active();

        let resample_step = state.dec_resample_step_check.get_active();

        // Decode

        let progress_callback = |progress: f32, description: String| {
            glib::idle_add(move || {
                set_progress(progress, &description.clone());
                Continue(false)
            });
        };

        let (signal, rate) = match noaa_apt::load(&input_filename) {
            Ok(result) => result,
            Err(e) => {
                callback(Err(e));
                return;
            }
        };

        let settings = state.settings.clone();

        std::thread::spawn(move || {

            let mut context = Context::decode(
                progress_callback,
                Rate::hz(state.settings.work_rate),
                Rate::hz(noaa_apt::FINAL_RATE),
                wav_steps,
                resample_step,
            );
            noaa_apt::decode(
                &mut context,
                &settings,
                &signal,
                rate,
                sync
            );
        });

    });

}
*/

/// Start decoding or resampling.
///
/// Starts another working thread and sets the start_button as not sensitive.
///
/// If this function returns Err() before starting the decode/resample, the
/// message will be shown on the info_bar thank to build_ui() who calls this
/// function
///
/// When the decode/resample ends the callback will set the start_button as
/// sensitive again. If there is an error decoding/resampling will also show the
/// error on the info_bar
fn decode() {

    // Create callbacks

    let callback = |result: err::Result<Signal>| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                widgets.dec_decode_button.set_sensitive(true);
                widgets.main_start_button.set_sensitive(true);
                match &result {
                    Ok(signal) => {
                        set_progress(1., "Decoded");
                        widgets.p_process_button.set_sensitive(true);
                        borrow_state_mut(|state| {
                            state.decoded_signal = Some(signal.clone());
                            state.processed_image = None;
                        });
                    },
                    Err(e) => {
                        set_progress(1., "Error");
                        show_info(gtk::MessageType::Error, &e.to_string());
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
            set_progress(progress, &description);
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

/*

fn read_timestamp() -> err::Result<()> {
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

fn write_timestamp() -> err::Result<()> {
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

/// Set progress of ProgressBar
fn set_progress(fraction: f32, description: &str) {
    borrow_widgets(|widgets| {
        widgets.main_progress_bar.set_fraction(fraction as f64);
        widgets.main_progress_bar.set_text(Some(description));
    });
}

/// Show InfoBar with custom message.
fn show_info(message_type: gtk::MessageType, text: &str) {
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
fn check_updates_and_show() {
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
        callback(misc::check_updates(VERSION));
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
fn open_in_browser<W>(window: &W, url: &str) -> err::Result<()>
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
