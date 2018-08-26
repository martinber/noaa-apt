use noaa_apt;

use gtk;
use gdk;
use gio;

use std::env::args;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

/// Start GUI.
pub fn main() {
    let application = gtk::Application::new("ar.com.mbernardi.noaa-apt",
                                            gio::ApplicationFlags::empty())
                                       .expect("Initialization failed...");

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

    let window: gtk::ApplicationWindow = builder.get_object("window")
            .expect("Couldn't get window");
    window.set_application(application);

    info!("GUI opened");


    // Set status_label and start_button to ready

    let status_label: gtk::Label = builder.get_object("status_label")
            .expect("Couldn't get status_label");
    let start_button: gtk::Button = builder.get_object("start_button")
            .expect("Couldn't get start_button");
    status_label.set_label("Ready");
    start_button.set_sensitive(true);

    // Set version on footer

    let footer_label: gtk::Label = builder.get_object("footer_label")
            .expect("Couldn't get footer_label");
    footer_label.set_label(format!(
"noaa-apt {}
Martín Bernardi
martin@mbernardi.com.ar", VERSION).as_str());

    // Configure decode_output_entry file chooser

    let decode_output_entry: gtk::Entry = builder.get_object("decode_output_entry")
            .expect("Couldn't get decode_output_entry");
    decode_output_entry.connect_icon_press(clone!(window, builder => move |_, _, _| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save file as"), Some(&window), gtk::FileChooserAction::Save);
        file_chooser.add_buttons(&[
            ("Ok", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok.into() {
            let filename = file_chooser.get_filename().expect("Couldn't get filename");

            let entry: gtk::Entry = builder.get_object("decode_output_entry")
                    .expect("Couldn't get decode_output_entry");
            entry.set_text(filename.to_str().unwrap());
        }

        file_chooser.destroy();
    }));

    // Configure resample_output_entry file chooser

    let resample_output_entry: gtk::Entry = builder.get_object("resample_output_entry")
            .expect("Couldn't get resample_output_entry");
    resample_output_entry.connect_icon_press(clone!(window, builder => move |_, _, _| {
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Save file as"), Some(&window), gtk::FileChooserAction::Save);
        file_chooser.add_buttons(&[
            ("Ok", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);
        if file_chooser.run() == gtk::ResponseType::Ok.into() {
            let filename = file_chooser.get_filename().expect("Couldn't get filename");

            let entry: gtk::Entry = builder.get_object("resample_output_entry")
                    .expect("Couldn't get decode_output_entry");
            entry.set_text(filename.to_str().unwrap());
        }

        file_chooser.destroy();
    }));

    // Connect start button

    let input_file_chooser: gtk::FileChooserButton = builder.get_object("input_file_chooser")
            .expect("Couldn't get input_file_chooser");
    start_button.connect_clicked(clone!(builder, input_file_chooser, status_label => move |_| {

        // Check inputs
        let input_filename = match input_file_chooser.get_filename() {
            Some(f) => String::from(f.to_str().expect("Invalid character in input path")),
            None => {
                status_label.set_markup("<b>Error: Select input file</b>");
                info!("Input file not selected");
                return
            },
        };

        // Check if we are decoding or resampling

        let options_stack: gtk::Stack = builder.get_object("options_stack")
                .expect("Couldn't get options_stack");

        match options_stack.get_visible_child_name()
                .expect("Stack has no visible child").as_str() {

            "decode_page" => {
                let filename_entry: gtk::Entry = builder.get_object("decode_output_entry")
                        .expect("Couldn't get decode_output_entry");
                let output_filename = filename_entry.get_text()
                        .expect("Couldn't get decode_output_entry text");

                status_label.set_markup("Processing");
                debug!("Decode {} to {}", input_filename, output_filename);

                // Hack to refresh the label
                while gtk::events_pending() {
                    gtk::main_iteration();
                }
                gdk::Window::process_all_updates();

                match noaa_apt::decode(
                        input_filename.as_str(), output_filename.as_str()) {
                    Ok(_) => status_label.set_markup("Finished"),
                    Err(e) => status_label.set_markup(
                            format!("<b>Error: {}</b>", e).as_str()),
                }
            },

            "resample_page" => {
                let filename_entry: gtk::Entry = builder.get_object("resample_output_entry")
                        .expect("Couldn't get resample_output_entry");
                let output_filename = filename_entry.get_text()
                        .expect("Couldn't get resample_output_entry text");

                let rate_spinner: gtk::SpinButton = builder.get_object("resample_rate_spinner")
                        .expect("Couldn't get resample_rate_entry");
                let rate = rate_spinner.get_value_as_int() as u32;

                status_label.set_markup("Processing");
                debug!("Resample {} as {} to {}", input_filename, rate, output_filename);

                // Hack to refresh the label
                while gtk::events_pending() {
                    gtk::main_iteration();
                }
                gdk::Window::process_all_updates();

                match noaa_apt::resample_wav(
                        input_filename.as_str(), output_filename.as_str(), rate) {
                    Ok(_) => status_label.set_markup("Finished"),
                    Err(e) => status_label.set_markup(
                            format!("<b>Error: {}</b>", e).as_str()),
                }
            },

            x => panic!("Unexpected stack child name {}", x),
        }


    }));

    // Finish and show

    window.connect_delete_event(clone!(window => move |_, _| {
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
}
