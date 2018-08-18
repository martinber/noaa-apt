use gtk;
use gdk;
use gio;

use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

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
    let builder = Builder::new();
    builder.add_from_string(glade_src).expect("Couldn't add from string");

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
            Some(f) => f,
            None => {
                status_label.set_markup("<b>Error: Select input file</b>");
                info!("Input file not selected");
                return
            },
        };

        let file = match File::open(&input_filename) {
            Ok(f) => f,
            Err(_) => {
                status_label.set_markup("<b>Error: Couldn't open file</b>");
                error!("Couldn't open file");
                return
            },
        };

        // Check if we are decoding or resampling

        let options_stack: gtk::Stack = builder.get_object("options_stack")
                .expect("Couldn't get options_stack");
        // let visible_box = options_stack.get_visible_child()
                // .expect("Couldn't get stack visible child");
        // match options_stack.child_get_property(visible_box, "position") {
        match options_stack.get_visible_child_name()
                .expect("Stack has no visible child").as_ref() {

            "decode_page" => {
                let entry: gtk::Entry = builder.get_object("decode_output_entry")
                        .expect("Couldn't get decode_output_entry");
                let output_filename = entry.get_text()
                        .expect("Couldn't get decode_output_entry text");
                println!("Decode: {}", output_filename);
            },

            "resample_page" => {
                let entry: gtk::Entry = builder.get_object("resample_output_entry")
                        .expect("Couldn't get resample_output_entry");
                let output_filename = entry.get_text()
                        .expect("Couldn't get resample_output_entry text");
                println!("Resample: {}", output_filename);
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
