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
    let glade_src = include_str!("gui.glade");
    let builder = Builder::new();
    builder.add_from_string(glade_src).expect("Couldn't add from string");

    let window: gtk::ApplicationWindow = builder.get_object("window")
            .expect("Couldn't get window");
    window.set_application(application);

    info!("GUI opened");

    let status_label: gtk::Label = builder.get_object("status_label")
            .expect("Couldn't get status_label");
    let start_button: gtk::Button = builder.get_object("start_button")
            .expect("Couldn't get start_button");
    let decode_output_entry: gtk::Entry = builder.get_object("decode_output_entry")
            .expect("Couldn't get decode_output_entry");
    let resample_output_entry: gtk::Entry = builder.get_object("resample_output_entry")
            .expect("Couldn't get resample_output_entry");

    status_label.set_label("Ready");
    start_button.set_sensitive(true);

    decode_output_entry.connect_icon_press(clone!(window => move |_, _, _| {
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
            // let file = File::open(&filename).expect("Couldn't open file");
//
            // let mut reader = BufReader::new(file);
            // let mut contents = String::new();
            // let _ = reader.read_to_string(&mut contents);

            // text_view.get_buffer().expect("Couldn't get window").set_text(&contents);
        }
    }));

    window.connect_delete_event(clone!(window => move |_, _| {
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
}
