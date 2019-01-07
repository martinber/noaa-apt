//! I'm using two threads, one for the GTK+ GUI and another one that starts when
//! decoding/resampling.
//!
//! GTK+ is not thread safe so everything GUI related is on the GTK+ thread that
//! is also the main thread. When pressing the Start button, a temporary thread
//! starts for decoding/resampling.
//!
//! I'm using a WidgetList struct for keeping track of every Widget I'm
//! interested in. This struct is wrapped on the Rc smart pointer to allow
//! multiple ownership of the struct. Previously I wrapped inside Rc and RefCell
//! too to allow mutable access to everyone, but AFAIK having mutable access
//! to a Widget is not neccesary.

use noaa_apt;
use dsp::Rate;
use err;

use gtk;
use gdk;
use gio;

use std::env::args;
use std::rc::Rc;
use std::sync::mpsc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
struct WidgetList {
    window:                   gtk::ApplicationWindow,
    status_label:             gtk::Label,
    footer_label:             gtk::Label,
    start_button:             gtk::Button,
    decode_output_entry:      gtk::Entry,
    resample_output_entry:    gtk::Entry,
    resample_rate_spinner:    gtk::SpinButton,
    input_file_chooser:       gtk::FileChooserButton,
    options_stack:            gtk::Stack,
    decode_sync_check:        gtk::CheckButton,
    decode_wav_steps_check:   gtk::CheckButton,
    resample_wav_steps_check: gtk::CheckButton,
}

impl WidgetList {
    fn create(builder: &gtk::Builder) -> WidgetList {
        WidgetList {
            window:                   builder.get_object("window"                  ).expect("Couldn't get window"                  ),
            status_label:             builder.get_object("status_label"            ).expect("Couldn't get status_label"            ),
            footer_label:             builder.get_object("footer_label"            ).expect("Couldn't get footer_label"            ),
            start_button:             builder.get_object("start_button"            ).expect("Couldn't get start_button"            ),
            decode_output_entry:      builder.get_object("decode_output_entry"     ).expect("Couldn't get decode_output_entry"     ),
            resample_output_entry:    builder.get_object("resample_output_entry"   ).expect("Couldn't get resample_output_entry"   ),
            resample_rate_spinner:    builder.get_object("resample_rate_spinner"   ).expect("Couldn't get resample_rate_spinner"   ),
            input_file_chooser:       builder.get_object("input_file_chooser"      ).expect("Couldn't get input_file_chooser"      ),
            options_stack:            builder.get_object("options_stack"           ).expect("Couldn't get options_stack"           ),
            decode_sync_check:        builder.get_object("decode_sync_check"       ).expect("Couldn't get decode_sync_check"       ),
            decode_wav_steps_check:   builder.get_object("decode_wav_steps_check"  ).expect("Couldn't get decode_wav_steps_check"  ),
            resample_wav_steps_check: builder.get_object("resample_wav_steps_check").expect("Couldn't get resample_wav_steps_check"),
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

    // Set version on footer

    widgets.footer_label.set_label(format!(
        "noaa-apt {}\n\
        Martín Bernardi\n\
        martin@mbernardi.com.ar",
        VERSION
    ).as_str());

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

enum Action {
    Decode,
    Resample,
}

/// Start decoding or resampling
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

    // Hack to refresh the label
    while gtk::events_pending() {
        gtk::main_iteration();
    }
    gdk::Window::process_all_updates();

    let (tx, rx) = mpsc::channel();
    enum Message {
        Success,
        Failed(err::Error),
    }

    match action {
        Action::Decode => {
            let sync = widgets.decode_sync_check.get_active();
            let wav_steps = widgets.decode_wav_steps_check.get_active();
            debug!("Decode {} to {}", input_filename, output_filename);

            std::thread::spawn(move || {
                match noaa_apt::decode(
                        input_filename.as_str(),
                        output_filename.as_str(),
                        wav_steps,
                        sync,
                ) {
                    Ok(_) => tx.send(Message::Success)
                        .expect("Failed to send message to main thread"),
                    Err(e) => tx.send(Message::Failed(e))
                        .expect("Failed to send message to main thread"),
                };
            })
        }
        Action::Resample => {
            let rate = widgets.resample_rate_spinner.get_value_as_int() as u32;
            let wav_steps = widgets.resample_wav_steps_check.get_active();
            debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

            std::thread::spawn(move || {
                match noaa_apt::resample_wav(
                        input_filename.as_str(),
                        output_filename.as_str(),
                        Rate::hz(rate),
                        wav_steps,
                ) {
                    Ok(_) => tx.send(Message::Success)
                        .expect("Failed to send message to main thread"),
                    Err(e) => tx.send(Message::Failed(e))
                        .expect("Failed to send message to main thread"),
                };
            })
        }
    };

    // Wait until the thread ends, I can't figure out how to make a callback to
    // thread.join() or a callback for when a message is ready on the rx
    // channel. So I'm polling when the gtk thread is idle.
    // I can't poll until the thread ends, so I poll until there are no more
    // messages,

    // TODO: fix this, I'm using 100% CPU!

    // Continue(false) stops this GTK+ task/thread whatever it is.
    gtk::idle_add(move || {
        match rx.try_recv() {
            Ok(Message::Success) => {
                widgets.status_label.set_markup("Finished");
                gtk::Continue(true)
            },
            Ok(Message::Failed(e)) => {
                widgets.status_label.set_markup(format!("<b>Error: {}</b>", e).as_str());
                error!("{}", e);
                gtk::Continue(true)
            },
            Err(mpsc::TryRecvError::Empty) => gtk::Continue(true),
            Err(mpsc::TryRecvError::Disconnected) => gtk::Continue(false),
        }
    });
}
