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
use std::rc::Rc;
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
use misc::ThreadGuard;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

// Option because it's none before building the GUI
// RefCell because I need mutable references

// Stores the WidgetList.
//
// Use the functions below when accesing it. Only available from the GUI thread.
thread_local!(static GLOBAL: RefCell<Option<WidgetList>> = RefCell::new(None));

/// Work with reference to WidgetList.
///
/// Panics if called from a thread different than the GUI one. Also panics if
/// the GUI is not built yet.
fn borrow_widgets<F: FnOnce(&WidgetList)>(f: F) {
    GLOBAL.with(|global| {
        if let Some(ref widgets) = *global.borrow() {
            (f)(widgets)
        } else {
            panic!("Can't get WidgetList. Tried to borrow from another thread \
                    or tried to borrow before building the GUI")
        }
    });
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
#[derive(Debug)]
struct WidgetList {
    window:                       gtk::ApplicationWindow,
    progress_bar:                 gtk::ProgressBar,
    start_button:                 gtk::Button,
    info_bar:                     gtk::InfoBar,
    info_label:                   gtk::Label,
    info_revealer:                gtk::Revealer,
    decode_output_entry:          gtk::Entry,
    resample_output_entry:        gtk::Entry,
    resample_rate_spinner:        gtk::SpinButton,
    input_file_chooser:           gtk::FileChooserButton,
    options_stack:                gtk::Stack,
    decode_sync_check:            gtk::CheckButton,
    decode_wav_steps_check:       gtk::CheckButton,
    resample_wav_steps_check:     gtk::CheckButton,
    decode_resample_step_check:   gtk::CheckButton,
    resample_resample_step_check: gtk::CheckButton,
    decode_contrast_combo:        gtk::ComboBoxText,
}

impl WidgetList {
    /// Create and load widgets from `gtk::Builder`.
    fn create(builder: &gtk::Builder) -> Self {
        Self {
            window:                       builder.get_object("window"                      ).expect("Couldn't get window"                      ),
            progress_bar:                 builder.get_object("progress_bar"                ).expect("Couldn't get progress_bar"                ),
            start_button:                 builder.get_object("start_button"                ).expect("Couldn't get start_button"                ),
            info_bar:                     builder.get_object("info_bar"                    ).expect("Couldn't get info_bar"                    ),
            info_label:                   builder.get_object("info_label"                  ).expect("Couldn't get info_label"                  ),
            info_revealer:                builder.get_object("info_revealer"               ).expect("Couldn't get info_revealer"               ),
            decode_output_entry:          builder.get_object("decode_output_entry"         ).expect("Couldn't get decode_output_entry"         ),
            resample_output_entry:        builder.get_object("resample_output_entry"       ).expect("Couldn't get resample_output_entry"       ),
            resample_rate_spinner:        builder.get_object("resample_rate_spinner"       ).expect("Couldn't get resample_rate_spinner"       ),
            input_file_chooser:           builder.get_object("input_file_chooser"          ).expect("Couldn't get input_file_chooser"          ),
            options_stack:                builder.get_object("options_stack"               ).expect("Couldn't get options_stack"               ),
            decode_sync_check:            builder.get_object("decode_sync_check"           ).expect("Couldn't get decode_sync_check"           ),
            decode_wav_steps_check:       builder.get_object("decode_wav_steps_check"      ).expect("Couldn't get decode_wav_steps_check"      ),
            decode_resample_step_check:   builder.get_object("decode_resample_step_check"  ).expect("Couldn't get decode_resample_step_check"  ),
            resample_wav_steps_check:     builder.get_object("resample_wav_steps_check"    ).expect("Couldn't get resample_wav_steps_check"    ),
            resample_resample_step_check: builder.get_object("resample_resample_step_check").expect("Couldn't get resample_resample_step_check"),
            decode_contrast_combo:        builder.get_object("decode_contrast_combo"       ).expect("Couldn't get decode_contrast_combo"       ),
        }
    }
}

/// Start GUI.
pub fn main() {
    let application = gtk::Application::new(
        "ar.com.mbernardi.noaa-apt",
        gio::ApplicationFlags::empty(),
    ).expect("Initialization failed");

    application.connect_startup(move |app| {
        build_ui(app);
    });
    application.connect_activate(|_| {});

    application.run(&args().collect::<Vec<_>>());
}

/// Build GUI from .glade file and get everything ready.
///
/// Connect signals to Widgets.
fn build_ui(application: &gtk::Application) {

    // Build GUI

    let glade_src = include_str!("gui.glade");
    let builder = Builder::new_from_string(glade_src);

    set_widgets(WidgetList::create(&builder));

    borrow_widgets(|widgets| {
        widgets.window.set_application(application);
    });

    // Set WM_CLASS property. Without it, on KDE the taskbar icon is correct,
    // but for some reason the window has a stock X11 icon on the top-left
    // corner. When I set WM_CLASS the window gets the correct icon.
    // GTK docs say that this option is deprecated?
    // https://gtk-rs.org/docs/gtk/trait.GtkWindowExt.html#tymethod.set_wmclass
    borrow_widgets(|widgets| {
        widgets.window.set_wmclass("noaa-apt", "noaa-apt");
    });

    build_system_menu(application);

    info!("GUI opened");

    // Set progress_bar and start_button to ready

    // GLOBAL.with(|global| {
        // if let Some(ref widgets) = *global.borrow() {
            // widgets.progress_bar.set_text("Ready");
            // widgets.start_button.set_sensitive(true);
        // }
    // });
    borrow_widgets(|widgets| {
        widgets.progress_bar.set_text("Ready");
        widgets.start_button.set_sensitive(true);
    });

    /*
    check_updates(Rc::clone(&widgets));
    */

    // Configure decode_output_entry file chooser

    /*
    let widgets_clone = Rc::clone(&widgets);
    widgets.decode_output_entry.connect_icon_press(move |_, _, _| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save file as"),
            Some(&widgets_clone.window),
            gtk::FileChooserAction::Save
        );

        file_chooser.add_buttons(&[
            ("Ok", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);

        if file_chooser.run() == Into::<i32>::into(gtk::ResponseType::Ok) {
            let filename = file_chooser.get_filename()
                .expect("Couldn't get filename");

            widgets_clone.decode_output_entry.set_text(filename.to_str().unwrap());
        }

        file_chooser.destroy();
    });
    */

    // Configure resample_output_entry file chooser

    /*
    let widgets_clone = Rc::clone(&widgets);
    widgets.resample_output_entry.connect_icon_press(move |_, _, _| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save file as"),
            Some(&widgets_clone.window),
            gtk::FileChooserAction::Save
        );

        file_chooser.add_buttons(&[
            ("Ok", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);

        if file_chooser.run() == Into::<i32>::into(gtk::ResponseType::Ok) {
            let filename =
                file_chooser.get_filename()
                .expect("Couldn't get filename");

            widgets_clone.resample_output_entry.set_text(filename.to_str().unwrap());
        }

        file_chooser.destroy();
    });
    */

    /*
    // Connect start button

    let widgets_clone = Rc::clone(&widgets);
    widgets.start_button.connect_clicked(move |_| {

        widgets_clone.info_revealer.set_reveal_child(false);

        // Check if we are decoding or resampling
        match widgets_clone.options_stack.get_visible_child_name()
            .expect("Stack has no visible child").as_str()
        {
            "decode_page" => run_noaa_apt(Action::Decode, Rc::clone(&widgets_clone)),
            "resample_page" => run_noaa_apt(Action::Resample, Rc::clone(&widgets_clone)),

            x => panic!("Unexpected stack child name {}", x),
        }.unwrap_or_else(|error| {
            let widgets= Rc::clone(&widgets_clone);
            show_info(&widgets, gtk::MessageType::Error, error.to_string().as_str());

            error!("{}", error);
        });
    });

    // Connect info_bar close button

    let widgets_clone = Rc::clone(&widgets);
    widgets.info_bar.connect_response(move |_, response| {
        if gtk::ResponseType::Close == response {
            widgets_clone.info_revealer.set_reveal_child(false);
        }
    });
    */

    // Finish and show

    /*
    let widgets_clone = Rc::clone(&widgets);
    widgets.window.connect_delete_event(move |_, _| {
        widgets_clone.window.destroy();
        Inhibit(false)
    });
    */

    borrow_widgets(|widgets| {
        widgets.window.show_all();
    });
}

/// Build menu bar
fn build_system_menu(application: &gtk::Application) {
    // let menu = gio::Menu::new();
    let menu_bar = gio::Menu::new();
    let more_menu = gio::Menu::new();
    let switch_menu = gio::Menu::new();
    let settings_menu = gio::Menu::new();
    let submenu = gio::Menu::new();

    // The first argument is the label of the menu item whereas the second is the action name. It'll
    // makes more sense when you'll be reading the "add_actions" function.
    // menu.append("Quit", "app.quit");

    switch_menu.append("Switch", "app.switch");
    menu_bar.append_submenu("_Switch", &switch_menu);

    settings_menu.append("Sub another", "app.sub_another");
    submenu.append("Sub sub another", "app.sub_sub_another");
    submenu.append("Sub sub another2", "app.sub_sub_another2");
    settings_menu.append_submenu("Sub menu", &submenu);
    menu_bar.append_submenu("_Another", &settings_menu);

    more_menu.append("About", "app.about");
    menu_bar.append_submenu("?", &more_menu);

    // application.set_app_menu(&menu);
    application.set_menubar(&menu_bar);
}

/*
/// If the user wants to decode or resample.
enum Action {
    Decode,
    Resample,
}

/// Start decoding or resampling.
///
/// Starts another working thread and updates the `status_label` when finished.
/// Also sets the button as not sensitive and then as sensitive again.
fn run_noaa_apt(action: Action, widgets: Rc<WidgetList>) -> err::Result<()> {

    // input_filename has to be a String instead of GString because I need to
    // move to another thread
    let input_filename: String = widgets
        .input_file_chooser
        .get_filename() // Option<std::path::PathBuf>
        .ok_or_else(|| err::Error::Internal("Select input file".to_string()))
        .and_then(|path: std::path::PathBuf| {
             path.to_str()
                 .ok_or_else(|| err::Error::Internal("Invalid character on input path".to_string()))
                 .map(|s: &str| s.to_string())
        })?;

    // output_filename has to be a String instead of GString because I need to
    // move to another thread
    let output_filename = match action {
        Action::Decode => widgets.decode_output_entry.get_text()
            .expect("Couldn't get decode_output_entry text").as_str().to_string(),
        Action::Resample => widgets.resample_output_entry.get_text()
            .expect("Couldn't get resample_output_entry text").as_str().to_string(),
    };

    if output_filename == "" {
        return Err(err::Error::Internal("Select output filename".to_string()))
    }

    // widgets.status_label.set_markup("Processing");

    // Callback called when decode/resample ends. Using ThreadGuard to send
    // widgets to another thread and back
    let widgets_cell = ThreadGuard::new(widgets.clone());
    let callback = move |result| {
        glib::idle_add(move || {
            let widgets = widgets_cell.borrow();

            widgets.start_button.set_sensitive(true);
            match result {
                Ok(()) => {
                    // widgets.status_label.set_markup("Finished");
                },
                Err(ref e) => {
                    show_info(&widgets, gtk::MessageType::Error, format!("{}", e).as_str());

                    error!("{}", e);
                },
            }
            gtk::Continue(false)
        });
    };
    let widgets_cell = ThreadGuard::new(widgets.clone());
    let progress_callback = move |progress| {
        glib::idle_add(move || {
            let widgets = widgets_cell.borrow();
            widgets.progress_bar.set_fraction(progress as f64);
            gtk::Continue(false)
        });
    };

    match action {
        Action::Decode => {
            let sync = widgets.decode_sync_check.get_active();
            let wav_steps = widgets.decode_wav_steps_check.get_active();
            let resample_step = widgets.decode_resample_step_check.get_active();

            // See https://stackoverflow.com/questions/48034119/rust-matching-a-optionstring
            let contrast_adjustment = match widgets
                .decode_contrast_combo
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

            widgets.start_button.set_sensitive(false);
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
        Action::Resample => {
            let rate = widgets.resample_rate_spinner.get_value_as_int() as u32;
            let wav_steps = widgets.resample_wav_steps_check.get_active();
            let resample_step = widgets.resample_resample_step_check.get_active();
            debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

            widgets.start_button.set_sensitive(false);
            std::thread::spawn(move || {
                let context = Context::resample(
                    |progress| println!("{}", progress),
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
}

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
fn check_updates(widgets: Rc<WidgetList>) {
    // Callback called when check_update ends. Using ThreadGuard to send
    // widgets to another thread and back
    let widgets_cell = ThreadGuard::new(widgets);
    let callback = move |result| {
        glib::idle_add(move || {
            let widgets = widgets_cell.borrow();
            match result {
                Some((true, ref latest)) => {
                    show_info(
                        &widgets,
                        gtk::MessageType::Info,
                        format!("Version \"{}\" available for download!", latest).as_str(),
                    );
                },
                Some((false, _)) => {},
                None => {
                    show_info(
                        &widgets,
                        gtk::MessageType::Info,
                        format!("Error checking for updates, do you have an internet connection?").as_str(),
                    );
                },
            }
            gtk::Continue(false)
        });
    };

    std::thread::spawn(move || {
        callback(misc::check_updates(VERSION));
    });
}
*/
