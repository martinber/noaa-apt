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

use gtk;
use gio;
use glib;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

use noaa_apt::{self, Contrast};
use context::Context;
use misc;
use dsp::Rate;
use misc::ThreadGuard;


/// Defined by Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Contains references to widgets, so I can pass them together around.
#[derive(Debug)]
struct WidgetList {
    window:                       gtk::ApplicationWindow,
    status_label:                 gtk::Label,
    footer_label:                 gtk::Label,
    start_button:                 gtk::Button,
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
}

impl WidgetList {
    /// Create and load widgets from `gtk::Builder`.
    fn create(builder: &gtk::Builder) -> WidgetList {
        WidgetList {
            window:                       builder.get_object("window"                      ).expect("Couldn't get window"                      ),
            status_label:                 builder.get_object("status_label"                ).expect("Couldn't get status_label"                ),
            footer_label:                 builder.get_object("footer_label"                ).expect("Couldn't get footer_label"                ),
            start_button:                 builder.get_object("start_button"                ).expect("Couldn't get start_button"                ),
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

    let widgets = Rc::new(WidgetList::create(&builder));

    widgets.window.set_application(application);

    info!("GUI opened");

    // Set status_label and start_button to ready

    widgets.status_label.set_label("Ready");
    widgets.start_button.set_sensitive(true);

    // Set footer

    update_footer(Rc::clone(&widgets));

    // Configure decode_output_entry file chooser

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

    // Configure resample_output_entry file chooser

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

    // Connect start button

    let widgets_clone = Rc::clone(&widgets);
    widgets.start_button.connect_clicked(move |_| {

        // Check if we are decoding or resampling

        match widgets_clone.options_stack.get_visible_child_name()
            .expect("Stack has no visible child").as_str() {

            "decode_page" => run_noaa_apt(Action::Decode, Rc::clone(&widgets_clone)),
            "resample_page" => run_noaa_apt(Action::Resample, Rc::clone(&widgets_clone)),

            x => panic!("Unexpected stack child name {}", x),
        }
    });

    // Finish and show

    let widgets_clone = Rc::clone(&widgets);
    widgets.window.connect_delete_event(move |_, _| {
        widgets_clone.window.destroy();
        Inhibit(false)
    });

    widgets.window.show_all();
}

/// If the user wants to decode or resample.
enum Action {
    Decode,
    Resample,
}

/// Start decoding or resampling.
///
/// Starts another working thread and updates the `status_label` when finished.
fn run_noaa_apt(action: Action, widgets: Rc<WidgetList>) {

    let input_filename = match widgets.input_file_chooser.get_filename() {
        Some(f) => {
            String::from(f.to_str().expect("Invalid character in input path"))
        }
        None => {
            widgets.status_label.set_markup("<b>Error: Select input file</b>");
            error!("Input file not selected");
            return
        },
    };

    let output_filename = match action {
        Action::Decode => widgets.decode_output_entry.get_text()
            .expect("Couldn't get decode_output_entry text"),
        Action::Resample => widgets.resample_output_entry.get_text()
            .expect("Couldn't get resample_output_entry text"),
    };

    if output_filename == "" {
        widgets.status_label.set_markup("<b>Error: Select output filename</b>");
        return
    }

    widgets.status_label.set_markup("Processing");

    // Callback called when decode/resample ends. Using ThreadGuard to send
    // widgets to another thread and back
    let widgets_cell = ThreadGuard::new(widgets.clone());
    let callback = move |result| {
        glib::idle_add(move || {
            let widgets = widgets_cell.borrow();
            match result {
                Ok(()) => {
                    widgets.status_label.set_markup("Finished");
                },
                Err(ref e) => {
                    widgets.status_label.set_markup(format!("<b>Error: {}</b>", e).as_str());
                    error!("{}", e);
                },
            }
            gtk::Continue(false)
        });
    };

    match action {
        Action::Decode => {
            let sync = widgets.decode_sync_check.get_active();
            let wav_steps = widgets.decode_wav_steps_check.get_active();
            let resample_step = widgets.decode_resample_step_check.get_active();
            debug!("Decode {} to {}", input_filename, output_filename);

            std::thread::spawn(move || {
                let context = Context::decode(
                    Rate::hz(noaa_apt::WORK_RATE),
                    Rate::hz(noaa_apt::FINAL_RATE),
                    wav_steps,
                    resample_step,
                );

                callback(noaa_apt::decode(
                    context,
                    input_filename.as_str(),
                    output_filename.as_str(),
                    Contrast::MinMax,
                    sync,
                ));
            });
        },
        Action::Resample => {
            let rate = widgets.resample_rate_spinner.get_value_as_int() as u32;
            let wav_steps = widgets.resample_wav_steps_check.get_active();
            let resample_step = widgets.resample_resample_step_check.get_active();
            debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

            std::thread::spawn(move || {
                let context = Context::resample(
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
}

/// Check for updates on another thread and show the result on the footer.
fn update_footer(widgets: Rc<WidgetList>) {

    // Show this while we check for updates online

    widgets.footer_label.set_label(format!(
        "noaa-apt {}\n\
        Martín Bernardi\n\
        martin@mbernardi.com.ar",
        VERSION
    ).as_str());


    // Callback called when check_update ends. Using ThreadGuard to send
    // widgets to another thread and back
    let widgets_cell = ThreadGuard::new(widgets);
    let callback = move |result| {
        glib::idle_add(move || {
            let widgets = widgets_cell.borrow();
            match result {
                Some((true, ref latest)) => {
                    widgets.footer_label.set_markup(format!(
                        "noaa-apt {} - <b>Version \"{}\" available for download!</b>\n\
                        Martín Bernardi\n\
                        martin@mbernardi.com.ar",
                        VERSION, latest
                    ).as_str());
                },
                Some((false, ref _latest)) => {
                    widgets.footer_label.set_markup(format!(
                        "noaa-apt {} - You have the latest version available\n\
                        Martín Bernardi\n\
                        martin@mbernardi.com.ar",
                        VERSION
                    ).as_str());
                },
                None => {
                    widgets.footer_label.set_markup(format!(
                        "noaa-apt {} - Error checking for updates\n\
                        Martín Bernardi\n\
                        martin@mbernardi.com.ar",
                        VERSION
                    ).as_str());
                },
            }
            gtk::Continue(false)
        });
    };

    std::thread::spawn(move || {
        callback(misc::check_updates(VERSION));
    });

}
