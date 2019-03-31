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

use std::env::args;
use std::cell::RefCell;

use gtk;
use gio;
use glib;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

use err;
use noaa_apt::{self, Contrast};
use context::Context;
use misc;
use dsp::Rate;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// If the user wants to decode or resample.
#[derive(Debug, Clone, Copy)]
enum Mode {
    Decode,
    Resample,
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
    progress_bar:          gtk::ProgressBar,
    start_button:          gtk::Button,
    info_bar:              gtk::InfoBar,
    info_label:            gtk::Label,
    info_revealer:         gtk::Revealer,
    output_entry:          gtk::Entry,
    rate_spinner:          Option<gtk::SpinButton>,
    input_file_chooser:    gtk::FileChooserButton,
    sync_check:            Option<gtk::CheckButton>,
    wav_steps_check:       gtk::CheckButton,
    resample_step_check:   gtk::CheckButton,
    contrast_combo:        Option<gtk::ComboBoxText>,
}

/// Start GUI.
///
/// Build the window.
pub fn main() {
    let application = gtk::Application::new(
        "ar.com.mbernardi.noaa-apt",
        gio::ApplicationFlags::empty(),
    ).expect("Initialization failed");

    application.connect_startup(move |app| {
        create_window(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}

/// Create empty window and call build_ui().
fn create_window(
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

    build_ui(mode, &application, &window);
}

/// Build GUI.
///
/// Loads GUI from glade file depending if decoding or resampling
fn build_ui(mode: Mode, application: &gtk::Application, window: &gtk::ApplicationWindow) {

    // Clean GUI if there was something previously

    if let Some(previous_outer_box) = window.get_child() {
        window.remove(&previous_outer_box);
    }

    // Load widgets from glade file depending if we are decoding or resampling
    // Every element loaded is inside main_box

    let builder = match mode {
        Mode::Decode => Builder::new_from_string(include_str!("decode.glade")),
        Mode::Resample => Builder::new_from_string(include_str!("resample.glade")),
    };

    let rate_spinner;
    let sync_check;
    let contrast_combo;
    match mode {
        Mode::Decode => {
            rate_spinner = None;
            sync_check = Some(builder.get_object("sync_check")
                .expect("Couldn't get sync_check"));
            contrast_combo = Some(builder.get_object("contrast_combo")
                .expect("Couldn't get contrast_combo"));
        },
        Mode::Resample => {
            rate_spinner = Some(builder.get_object("rate_spinner")
                .expect("Couldn't get sync_check"));
            sync_check = None;
            contrast_combo = None;
        },
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
        progress_bar:        builder.get_object("progress_bar"       ).expect("Couldn't get progress_bar"       ),
        start_button:        builder.get_object("start_button"       ).expect("Couldn't get start_button"       ),
        output_entry:        builder.get_object("output_entry"       ).expect("Couldn't get output_entry"       ),
        input_file_chooser:  builder.get_object("input_file_chooser" ).expect("Couldn't get input_file_chooser" ),
        wav_steps_check:     builder.get_object("wav_steps_check"    ).expect("Couldn't get wav_steps_check"    ),
        resample_step_check: builder.get_object("resample_step_check").expect("Couldn't get resample_step_check"),
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

    // Set progress_bar and start_button to ready

    widgets.progress_bar.set_text("Ready");
    widgets.start_button.set_sensitive(true);

    check_updates();

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

            if file_chooser.run() == Into::<i32>::into(gtk::ResponseType::Ok) {
                let filename = file_chooser.get_filename()
                    .expect("Couldn't get filename");

                widgets.output_entry.set_text(filename.to_str().unwrap());
            }

            file_chooser.destroy();
        });
    });

    // Connect start button

    widgets.start_button.connect_clicked(move |_| {
        borrow_widgets(|widgets| {
            widgets.info_revealer.set_reveal_child(false);

            run_noaa_apt(mode).unwrap_or_else(|error| {
                show_info(&widgets, gtk::MessageType::Error, error.to_string().as_str());
                error!("{}", error);
                widgets.start_button.set_sensitive(true);
            });
        });
    });

    // Finish and show

    widgets.window.connect_delete_event(|_, _| {
        borrow_widgets(|widgets| {
            widgets.window.destroy();
            Inhibit(false)
        })
    });

    build_system_menu(mode, application, &window);

    widgets.window.show_all();
}

/// Build menu bar
fn build_system_menu(mode: Mode, application: &gtk::Application, window: &gtk::ApplicationWindow) {

    // Create menu bar

    let menu_bar = gio::Menu::new();
    let help_menu = gio::Menu::new();
    let tools_menu = gio::Menu::new();

    tools_menu.append("_Decode", "app.decode");
    tools_menu.append("_Resample WAV", "app.resample");
    menu_bar.append_submenu("_Tools", &tools_menu);

    help_menu.append("_Usage", "app.usage");
    help_menu.append("_Guide", "app.guide");
    help_menu.append("_About", "app.about");
    menu_bar.append_submenu("_Help", &help_menu);

    application.set_menubar(&menu_bar);

    // Add actions to buttons

    let decode = gio::SimpleAction::new("decode", None);
    let w = window.clone();
    let a = application.clone();
    decode.connect_activate(move |_, _| {
        build_ui(Mode::Decode, &a, &w);
    });
    if let Mode::Resample = mode {
        application.add_action(&decode);
    } else {
        application.remove_action("decode");
    }

    let resample = gio::SimpleAction::new("resample", None);
    let w = window.clone();
    let a = application.clone();
    resample.connect_activate(move |_, _| {
        build_ui(Mode::Resample, &a, &w);
    });
    if let Mode::Decode = mode {
        application.add_action(&resample);
    } else {
        application.remove_action("resample");
    }

    let about = gio::SimpleAction::new("about", None);
    about.connect_activate(|_, _| {
        let dialog = gtk::AboutDialog::new();
        dialog.set_program_name("noaa-apt");
        dialog.set_version(VERSION);
        dialog.set_authors(&["Martín Bernardi <martin@mbernardi.com.ar>"]);
        dialog.set_website_label(Some("noaa-apt website"));
        dialog.set_website(Some("https://noaa-apt.mbernardi.com.ar/"));
        dialog.set_license_type(gtk::License::Gpl30);
        dialog.set_title("About noaa-apt");
        // dialog.set_transient_for(Some(&window)); // Not working?
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
fn run_noaa_apt(mode: Mode) -> err::Result<()> {

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

        let wav_steps = widgets.wav_steps_check.get_active();
        let resample_step = widgets.resample_step_check.get_active();

        match mode {
            Mode::Decode => {
                let sync = widgets
                    .clone() // Why I need this clone()?
                    .sync_check
                    .expect("Couldn't get sync_check")
                    .get_active();

                // See https://stackoverflow.com/questions/48034119/rust-matching-a-optionstring
                let contrast_adjustment: Contrast = match widgets
                    .clone() // Why I need this clone()?
                    .contrast_combo
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
                        Rate::hz(noaa_apt::WORK_RATE),
                        Rate::hz(noaa_apt::FINAL_RATE),
                        wav_steps,
                        resample_step,
                    );

                    callback(noaa_apt::decode(
                        context,
                        input_filename.as_str(),
                        output_filename.as_str(),
                        contrast_adjustment,
                        sync,
                    ));
                });
            },
            Mode::Resample => {
                let rate = widgets
                    .clone() // Why I need this clone()?
                    .rate_spinner
                    .expect("Couldn't get rate_spinner")
                    .get_value_as_int() as u32;

                debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

                widgets.start_button.set_sensitive(false);
                std::thread::spawn(move || {
                    let context = Context::resample(
                        progress_callback,
                        wav_steps,
                        resample_step,
                    );

                    callback(noaa_apt::resample_wav(
                        context,
                        input_filename.as_str(),
                        output_filename.as_str(),
                        Rate::hz(rate),
                    ));
                });
            },
        };

        Ok(())
    })
}

/// Set progress of ProgressBar
fn set_progress(fraction: f32, description: String) {
    borrow_widgets(|widgets| {
        widgets.progress_bar.set_fraction(fraction as f64);
        widgets.progress_bar.set_text(description.as_str());
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
fn check_updates() {
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
