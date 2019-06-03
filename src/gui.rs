//! GUI code.
//!
//! I'm using two threads, one for the GTK+ GUI and another one that starts when
//! decoding/resampling.
//!
//! GTK+ is not thread safe so everything GUI related is on the GTK+ thread that
//! is also the main thread. When pressing the Start button, a temporary thread
//! starts for decoding/resampling.
//!
//! I'm using a `WidgetList` struct for keeping track of every Widget I'm
//! interested in. This struct is wrapped on the `Rc` smart pointer to allow
//! multiple ownership of the struct. Previously I wrapped inside `Rc` and
//! `RefCell` too to allow mutable access to everyone, but AFAIK having mutable
//! access to a Widget is not neccesary.
//!
//! When doing a callback from another thread I use `ThreadGuard`, lets you
//! `Send` the Widgets to another thread but you cant use them there (panics in
//! that case). So I use `glib::idle_add()` to execute code on the main thread
//! from another thread. In the end, we send the widgets to another thread and
//! back.

use std::cell::RefCell;

use gtk;
use gio;
use glib;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;
use chrono;
use chrono::prelude::*;

use err;
use noaa_apt::{self, Contrast};
use context::Context;
use misc;
use config;
use dsp::Rate;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// If the user wants to decode, resample or change timestamps.
#[derive(Debug, Clone, Copy)]
enum Mode {
    Decode,
    Resample,
    Timestamp,
}

// Stores the WidgetList.
//
// Use the functions below when accesing it. Only available from the GUI thread.
// Wrapped on Option because it's None before building the GUI.
// Wrapped on RefCell because I need mutable references when modifying the GUI.
thread_local!(static GLOBAL: RefCell<Option<WidgetList>> = RefCell::new(None));


/// Work with reference to WidgetList.
///
/// Panics if called from a thread different than the GUI one. Also panics if
/// the GUI is not built yet.
fn borrow_widgets<F, R>(f: F) -> R
where F: FnOnce(&WidgetList) -> R
{
    GLOBAL.with(|global| {
        if let Some(ref widgets) = *global.borrow() {
            (f)(widgets)
        } else {
            panic!("Can't get WidgetList. Tried to borrow from another thread \
                    or tried to borrow before building the GUI")
        }
    })
}

/// Set the WidgetList.
///
/// Called when building the GUI.
fn set_widgets(widget_list: WidgetList) {
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some(widget_list);
    });
}


/// Contains references to widgets, so I can pass them together around.
///
/// Widgets that may not exist are wrapped on Option.
#[derive(Debug, Clone)]
struct WidgetList {
    mode:                  Mode,
    window:                gtk::ApplicationWindow,
    outer_box:             gtk::Box,
    main_box:              gtk::Box,
    progress_bar:          Option<gtk::ProgressBar>,
    start_button:          gtk::Button,
    info_bar:              gtk::InfoBar,
    info_label:            gtk::Label,
    info_revealer:         gtk::Revealer,
    output_entry:          gtk::Entry,
    rate_spinner:          Option<gtk::SpinButton>,
    input_file_chooser:    gtk::FileChooserButton,
    sync_check:            Option<gtk::CheckButton>,
    wav_steps_check:       Option<gtk::CheckButton>,
    resample_step_check:   Option<gtk::CheckButton>,
    contrast_combo:        Option<gtk::ComboBoxText>,
    read_button:           Option<gtk::Button>,
    hour_spinner:          Option<gtk::SpinButton>,
    minute_spinner:        Option<gtk::SpinButton>,
    second_spinner:        Option<gtk::SpinButton>,
    timezone_label:        Option<gtk::Label>,
    calendar:              Option<gtk::Calendar>,
}

/// Start GUI.
///
/// Build the window.
pub fn main(check_updates: bool, settings: config::GuiSettings) {
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

/// Create empty window and call build_ui().
fn create_window(
    check_updates: bool,
    settings: config::GuiSettings,
    application: &gtk::Application,
) {

    let window = gtk::ApplicationWindow::new(application);

    let mode = Mode::Decode;

    window.set_title("noaa-apt");
    window.set_default_size(450, -1);

    // Set WM_CLASS property. Without it, on KDE the taskbar icon is correct,
    // but for some reason the window has a stock X11 icon on the top-left
    // corner. When I set WM_CLASS the window gets the correct icon.
    // GTK docs say that this option is deprecated?
    // https://gtk-rs.org/docs/gtk/trait.GtkWindowExt.html#tymethod.set_wmclass
    window.set_wmclass("noaa-apt", "noaa-apt");

    build_ui(check_updates, settings, mode, &application, &window);
}

/// Build GUI.
///
/// Loads GUI from glade file depending if decoding, resampling or changing
/// timestamps.
fn build_ui(
    check_updates: bool,
    settings: config::GuiSettings,
    mode: Mode,
    application: &gtk::Application,
    window: &gtk::ApplicationWindow
) {

    // Clean GUI if there was something previously

    if let Some(previous_outer_box) = window.get_child() {
        window.remove(&previous_outer_box);
    }

    // Load widgets from glade file depending if we are decoding or resampling
    // Every element loaded is inside main_box

    let builder = match mode {
        Mode::Decode => Builder::new_from_string(include_str!("decode.glade")),
        Mode::Resample => Builder::new_from_string(include_str!("resample.glade")),
        Mode::Timestamp => Builder::new_from_string(include_str!("timestamp.glade")),
    };

    let rate_spinner;
    let sync_check;
    let contrast_combo;
    let progress_bar;
    let wav_steps_check;
    let resample_step_check;
    let read_button;
    let hour_spinner;
    let minute_spinner;
    let second_spinner;
    let timezone_label;
    let calendar;
    match mode {
        Mode::Decode => {
            rate_spinner = None;
            sync_check = Some(builder.get_object("sync_check")
                .expect("Couldn't get sync_check"));
            contrast_combo = Some(builder.get_object("contrast_combo")
                .expect("Couldn't get contrast_combo"));
            progress_bar = Some(builder.get_object("progress_bar")
                .expect("Couldn't get progress_bar"));
            wav_steps_check = Some(builder.get_object("wav_steps_check")
                .expect("Couldn't get wav_steps_check"));
            resample_step_check = Some(builder.get_object("resample_step_check")
                .expect("Couldn't get resample_step_check"));
            read_button = None;
            hour_spinner = None;
            minute_spinner = None;
            second_spinner = None;
            timezone_label = None;
            calendar = None;
        },
        Mode::Resample => {
            rate_spinner = Some(builder.get_object("rate_spinner")
                .expect("Couldn't get sync_check"));
            sync_check = None;
            contrast_combo = None;
            progress_bar = Some(builder.get_object("progress_bar")
                .expect("Couldn't get progress_bar"));
            wav_steps_check = Some(builder.get_object("wav_steps_check")
                .expect("Couldn't get wav_steps_check"));
            resample_step_check = Some(builder.get_object("resample_step_check")
                .expect("Couldn't get resample_step_check"));
            read_button = None;
            hour_spinner = None;
            minute_spinner = None;
            second_spinner = None;
            timezone_label = None;
            calendar = None;
        },
        Mode::Timestamp => {
            rate_spinner = None;
            sync_check = None;
            contrast_combo = None;
            progress_bar = None;
            wav_steps_check = None;
            resample_step_check = None;
            read_button = Some(builder.get_object("read_button")
                .expect("Couldn't get read_button"));
            hour_spinner = Some(builder.get_object("hour_spinner")
                .expect("Couldn't get hour_spinner"));
            minute_spinner = Some(builder.get_object("minute_spinner")
                .expect("Couldn't get minute_spinner"));
            second_spinner = Some(builder.get_object("second_spinner")
                .expect("Couldn't get second_spinner"));
            timezone_label = Some(builder.get_object("timezone_label")
                .expect("Couldn't get timezone_label"));
            calendar = Some(builder.get_object("calendar")
                .expect("Couldn't get calendar"));
        }
    };

    let widgets = WidgetList {
        mode,
        window:              window.clone(),
        outer_box:           gtk::Box::new(gtk::Orientation::Vertical, 0),
        info_bar:            gtk::InfoBar::new(),
        info_label:          gtk::Label::new(None),
        info_revealer:       gtk::Revealer::new(),
        rate_spinner,
        sync_check,
        contrast_combo,
        main_box:            builder.get_object("main_box"           ).expect("Couldn't get main_box"           ),
        progress_bar,
        start_button:        builder.get_object("start_button"       ).expect("Couldn't get start_button"       ),
        output_entry:        builder.get_object("output_entry"       ).expect("Couldn't get output_entry"       ),
        input_file_chooser:  builder.get_object("input_file_chooser" ).expect("Couldn't get input_file_chooser" ),
        wav_steps_check,
        resample_step_check,
        read_button,
        hour_spinner,
        minute_spinner,
        second_spinner,
        timezone_label,
        calendar,
    };

    // Add info_bar

    widgets.info_revealer.add(&widgets.info_bar);
    widgets.info_bar.set_show_close_button(true);
    widgets.info_bar.connect_response(|_, response| {
        if gtk::ResponseType::Close == response {
            borrow_widgets(|widgets| {
                widgets.info_revealer.set_reveal_child(false);
            });
        }
    });
    let info_content_area = widgets
        .info_bar
        .get_content_area()
        .expect("Couldn't get info_content_area (is None)")
        .downcast::<gtk::Box>()
        .expect("Couldn't get info_content_area (not a gtk::Box)");
    info_content_area.add(&widgets.info_label);

    // Finish adding elements
    //
    // - window
    //     - outer_box
    //         - main_box (everything loaded from glade file)
    //             - (everything you see on screen)
    //             - ...
    //         - info_revealer
    //             - info_bar

    widgets.outer_box.pack_start(&widgets.main_box, true, true, 0);
    widgets.outer_box.pack_end(&widgets.info_revealer, false, false, 0);

    widgets.window.add(&widgets.outer_box);

    set_widgets(widgets.clone());

    info!("GUI opened: {:?}", mode);

    // Set progress_bar and buttons to ready

    if let Some(progress_bar) = widgets.progress_bar.as_ref() {
        progress_bar.set_text(Some("Ready"));
    }
    widgets.start_button.set_sensitive(true);

    // Set timezone if on timestamp mode
    if let Some(label) = widgets.timezone_label.as_ref() {
        // Create any chrono::DateTime from chrono::Local, then ignore the
        // result and only take the timezone
        let time = chrono::Local::now();
        label.set_text(format!(
            "Local time\n(UTC{})",
            time.format("%:z"),
        ).as_str());
    }

    if check_updates {
        check_updates_and_show();
    }

    // Configure output_entry file chooser

    widgets.output_entry.connect_icon_press(|_, _, _| {
        borrow_widgets(|widgets| {
            let file_chooser = gtk::FileChooserDialog::new(
                Some("Save file as"),
                Some(&widgets.window),
                gtk::FileChooserAction::Save,
            );

            file_chooser.add_buttons(&[
                ("Ok", gtk::ResponseType::Ok.into()),
                ("Cancel", gtk::ResponseType::Cancel.into()),
            ]);

            if file_chooser.run() == gtk::ResponseType::Ok {
                let filename = file_chooser.get_filename()
                    .expect("Couldn't get filename");

                widgets.output_entry.set_text(filename.to_str().unwrap());
            }

            file_chooser.destroy();
        });
    });

    // Connect start button

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

    } else {

        let settings_clone = settings.clone();
        widgets.start_button.connect_clicked(move |_| {
            borrow_widgets(|widgets| {
                widgets.info_revealer.set_reveal_child(false);

                run_noaa_apt(settings_clone.clone(), mode).unwrap_or_else(|error| {
                    show_info(&widgets, gtk::MessageType::Error, error.to_string().as_str());
                    error!("{}", error);
                    widgets.start_button.set_sensitive(true);
                });
            });
        });
    }

    // Finish and show

    widgets.window.connect_delete_event(|_, _| {
        borrow_widgets(|widgets| {
            widgets.window.destroy();
            Inhibit(false)
        })
    });

    build_system_menu(check_updates, settings, mode, application, &window);

    widgets.window.show_all();
}

/// Build menu bar
fn build_system_menu(
    check_updates: bool,
    settings: config::GuiSettings,
    mode: Mode,
    application: &gtk::Application,
    window: &gtk::ApplicationWindow
) {

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

    application.set_menubar(Some(&menu_bar));

    // Add actions to buttons

    let decode = gio::SimpleAction::new("decode", None);
    let w = window.clone();
    let a = application.clone();
    let s = settings.clone();
    decode.connect_activate(move |_, _| {
        build_ui(check_updates, s.clone(), Mode::Decode, &a, &w);
    });

    let resample = gio::SimpleAction::new("resample", None);
    let w = window.clone();
    let a = application.clone();
    let s = settings.clone();
    resample.connect_activate(move |_, _| {
        build_ui(check_updates, s.clone(), Mode::Resample, &a, &w);
    });

    let timestamp = gio::SimpleAction::new("timestamp", None);
    let w = window.clone();
    let a = application.clone();
    let s = settings.clone();
    timestamp.connect_activate(move |_, _| {
        build_ui(check_updates, s.clone(), Mode::Timestamp, &a, &w);
    });

    application.add_action(&decode);
    application.add_action(&resample);
    application.add_action(&timestamp);
    match mode {
        Mode::Decode => application.remove_action("decode"),
        Mode::Resample => application.remove_action("resample"),
        Mode::Timestamp => application.remove_action("timestamp"),
    }

    let usage = gio::SimpleAction::new("usage", None);
    let w = window.clone();
    usage.connect_activate(move |_, _| {
        open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/usage.html")
            .expect("Failed to open usage webpage");
    });
    application.add_action(&usage);

    let guide = gio::SimpleAction::new("guide", None);
    let w = window.clone();
    guide.connect_activate(move |_, _| {
        open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/guide.html")
            .expect("Failed to open usage webpage");
    });
    application.add_action(&guide);

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(|_, _| {
        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name("noaa-apt");
        dialog.set_version(Some(VERSION));
        dialog.set_authors(&["Martín Bernardi <martin@mbernardi.com.ar>"]);
        dialog.add_credit_section("Thank you",
                &[
                    "RTL-SDR.com", "pietern", "Grant T. Olson", "FMighty",
                    "Peter Vogel", "wren84", "Florentin314", "xxretartistxx",
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
    application.add_action(&about);
}

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
fn run_noaa_apt(settings: config::GuiSettings, mode: Mode) -> err::Result<()> {

    // Create callbacks

    let callback = move |result| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                widgets.start_button.set_sensitive(true);
                match result {
                    Ok(()) => {
                        // widgets.status_label.set_markup("Finished");
                        set_progress(1., "Finished".to_string());
                    },
                    Err(ref e) => {
                        set_progress(1., "Error".to_string());
                        show_info(&widgets, gtk::MessageType::Error, format!("{}", e).as_str());

                        error!("{}", e);
                    },
                }
            });
            gtk::Continue(false)
        });
    };
    let progress_callback = |progress, description: String| {
        glib::idle_add(move || {
            set_progress(progress, description.clone());
            gtk::Continue(false)
        });
    };

    borrow_widgets(|widgets| {

        widgets.start_button.set_sensitive(false);

        // input_filename has to be a String instead of GString because I need it to
        // implement Sync

        let input_filename: String = widgets
            .input_file_chooser
            .get_filename() // Option<std::path::PathBuf>
            .ok_or_else(|| err::Error::Internal("Select input file".to_string()))
            .and_then(|path: std::path::PathBuf| {
                 path.to_str()
                     .ok_or_else(|| err::Error::Internal("Invalid character on input path".to_string()))
                     .map(|s: &str| s.to_string())
            })?;

        // output_filename has to be a String instead of GString because I need it
        // to implement Sync

        let output_filename: String = widgets
            .output_entry
            .get_text()
            .expect("Couldn't get decode_output_entry text")
            .as_str()
            .to_string();

        if output_filename == "" {
            return Err(err::Error::Internal("Select output filename".to_string()))
        }

        match mode {
            Mode::Decode => {
                let sync = widgets
                    .sync_check
                    .as_ref()
                    .expect("Couldn't get sync_check")
                    .get_active();

                let wav_steps = widgets
                    .wav_steps_check
                    .as_ref()
                    .expect("Couldn't get wav_steps_check")
                    .get_active();

                let resample_step = widgets
                    .resample_step_check
                    .as_ref()
                    .expect("Couldn't get resample_step_check")
                    .get_active();

                // See https://stackoverflow.com/questions/48034119/rust-matching-a-optionstring
                let contrast_adjustment: Contrast = match widgets
                    .contrast_combo
                    .as_ref()
                    .expect("Couldn't get contrast_combo")
                    .get_active_text()
                    .as_ref()
                    .map(|s| s.as_str())
                {
                    Some("Keep 98 percent") => Ok(Contrast::Percent(0.98)),
                    Some("From telemetry") => Ok(Contrast::Telemetry),
                    Some("Disable") => Ok(Contrast::MinMax),
                    Some(id) => Err(err::Error::Internal(
                        format!("Unknown contrast adjustment \"{}\"", id)
                    )),
                    None => Err(err::Error::Internal(
                        "Select contrast adjustment".to_string()
                    )),
                }?;

                debug!("Decode {} to {}", input_filename, output_filename);

                std::thread::spawn(move || {
                    let context = Context::decode(
                        progress_callback,
                        Rate::hz(settings.work_rate),
                        Rate::hz(noaa_apt::FINAL_RATE),
                        wav_steps,
                        resample_step,
                    );

                    let settings = config::DecodeSettings {
                        input_filename,
                        output_filename,
                        sync,
                        contrast_adjustment,
                        export_wav: wav_steps,
                        export_resample_filtered: resample_step,
                        work_rate: settings.work_rate,
                        resample_atten: settings.resample_atten,
                        resample_delta_freq: settings.resample_delta_freq,
                        resample_cutout: settings.resample_cutout,
                        demodulation_atten: settings.demodulation_atten,
                    };

                    callback(noaa_apt::decode(
                        context,
                        settings,
                    ));
                });

                Ok(())
            },
            Mode::Resample => {
                let rate = widgets
                    .clone() // Why I need this clone()?
                    .rate_spinner
                    .expect("Couldn't get rate_spinner")
                    .get_value_as_int() as u32;

                let wav_steps = widgets
                    .clone() // Why I need this clone()?
                    .wav_steps_check
                    .expect("Couldn't get wav_steps_check")
                    .get_active();

                let resample_step = widgets
                    .clone() // Why I need this clone()?
                    .resample_step_check
                    .expect("Couldn't get resample_step_check")
                    .get_active();

                debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

                widgets.start_button.set_sensitive(false);
                std::thread::spawn(move || {
                    let context = Context::resample(
                        progress_callback,
                        wav_steps,
                        resample_step,
                    );

                    let settings = config::ResampleSettings {
                        input_filename,
                        output_filename,
                        output_rate: rate,
                        export_wav: wav_steps,
                        export_resample_filtered: resample_step,
                        wav_resample_atten: settings.wav_resample_atten,
                        wav_resample_delta_freq: settings.wav_resample_delta_freq,
                    };

                    callback(noaa_apt::resample_wav(
                        context,
                        settings,
                    ));
                });

                Ok(())
            },
            Mode::Timestamp => {
                Err(err::Error::Internal(
                    format!("Unexpected mode 'Timestamp'")
                ))
            },
        }
    })
}

fn read_timestamp() -> err::Result<()> {
    borrow_widgets(|widgets| {

        let calendar = widgets.calendar.as_ref().expect("Couldn't get calendar");
        let hour_spinner = widgets.hour_spinner.as_ref().expect("Couldn't get hour_spinner");
        let minute_spinner = widgets.minute_spinner.as_ref().expect("Couldn't get minute_spinner");
        let second_spinner = widgets.second_spinner.as_ref().expect("Couldn't get second_spinner");

        let input_filename: String = widgets
            .input_file_chooser
            .get_filename() // Option<std::path::PathBuf>
            .ok_or_else(|| err::Error::Internal("Select input file".to_string()))
            .and_then(|path: std::path::PathBuf| {
                 path.to_str()
                     .ok_or_else(|| err::Error::Internal("Invalid character on input path".to_string()))
                     .map(|s: &str| s.to_string())
            })?;

        let timestamp = misc::read_timestamp(input_filename.as_str())?;
        let datetime = chrono::Local.timestamp(timestamp, 0);

        // GTK counts months from 0 to 11. Years and days are fine
        calendar.select_month(datetime.month0() as u32, datetime.year() as u32);
        calendar.select_day(datetime.day());
        hour_spinner.set_value(datetime.hour() as f64);
        minute_spinner.set_value(datetime.minute() as f64);
        second_spinner.set_value(datetime.second() as f64);

        Ok(())
    })
}

fn write_timestamp() -> err::Result<()> {
    borrow_widgets(|widgets| {

        let calendar = widgets.calendar.as_ref().expect("Couldn't get calendar");
        let hour_spinner = widgets.hour_spinner.as_ref().expect("Couldn't get hour_spinner");
        let minute_spinner = widgets.minute_spinner.as_ref().expect("Couldn't get minute_spinner");
        let second_spinner = widgets.second_spinner.as_ref().expect("Couldn't get second_spinner");

        let output_filename: String = widgets
            .output_entry
            .get_text()
            .expect("Couldn't get decode_output_entry text")
            .as_str()
            .to_string();

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

        misc::write_timestamp(datetime.timestamp(), output_filename.as_str())?;

        Ok(())
    })
}

/// Set progress of ProgressBar
fn set_progress(fraction: f32, description: String) {
    borrow_widgets(|widgets| {
        let progress_bar = widgets
            .progress_bar
            .as_ref()
            .expect("Couldn't get progress_bar");
        progress_bar.set_fraction(fraction as f64);
        progress_bar.set_text(Some(description.as_str()));
    });
}

/// Show InfoBar with custom message.
fn show_info(widgets: &WidgetList, message_type: gtk::MessageType, text: &str) {
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
}

/// Check for updates on another thread and show the result on the info_bar.
fn check_updates_and_show() {
    let callback = move |result| {
        glib::idle_add(move || {
            borrow_widgets(|widgets| {
                match result {
                    Some((true, ref latest)) => {
                        show_info(
                            &widgets,
                            gtk::MessageType::Info,
                            format!("Version \"{}\" available for download!", latest).as_str(),
                        );
                    },
                    Some((false, _)) => {}, // Do nothing, already on latest version
                    None => {
                        show_info(
                            &widgets,
                            gtk::MessageType::Info,
                            format!("Error checking for updates, do you have an internet connection?").as_str(),
                        );
                    },
                }
            });
            gtk::Continue(false)
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
where W: gtk::IsA<gtk::Window>
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
        ).or(Err(err::Error::Internal("Could not open browser".to_string())))
    }
}
