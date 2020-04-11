//! Main GUI code. Mostly initialization code.
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

use chrono::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;
use log::info;

use crate::config;
use super::state::{
    GuiState, borrow_state, borrow_state_mut, set_state,
    Widgets, borrow_widgets, set_widgets
};
use super::misc;
use super::work;


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
        misc::check_updates_and_show(VERSION);
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
                            misc::show_info(gtk::MessageType::Error,
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
    borrow_state_mut(|state| {
        // Reset working signal and image
        state.decoded_signal = None;
        state.processed_image = None;
    });

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
        misc::set_progress(0., "Ready");

        // TODO Reiniciar imagen
        // Reset image

        let pixbuf = gdk_pixbuf::Pixbuf::new_from_file(Path::new("./res/icon.png"))
            .expect("Couldn't load ./res/icon.png");
        widgets.img_image.set_from_pixbuf(Some(&pixbuf));

        // Connect buttons

        widgets.dec_decode_button.connect_clicked(|_| work::decode());
        widgets.p_process_button.connect_clicked(|_| work::process());
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
                    misc::show_info(&widgets_clone, gtk::MessageType::Error, error.to_string().as_str());
                    error!("{}", error);
                }
            });
            let widgets_clone = widgets.clone();
            widgets.read_button.expect("Couldn't get read_button")
                .connect_clicked(move |_|
            {
                if let Err(error) = read_timestamp() {
                    misc::show_info(&widgets_clone, gtk::MessageType::Error, error.to_string().as_str());
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
        misc::open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/usage.html")
            .expect("Failed to open usage webpage");
    });
    widgets.application.add_action(&usage);

    let guide = gio::SimpleAction::new("guide", None);
    let w = widgets.window.clone();
    guide.connect_activate(move |_, _| {
        misc::open_in_browser(&w, "https://noaa-apt.mbernardi.com.ar/guide.html")
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
                misc::open_in_browser(dialog, url).expect("Failed to open link");
                return gtk::Inhibit(true); // Override `show_uri_on_window`
            });
        }

        dialog.run();
        dialog.destroy();
    });
    widgets.application.add_action(&about);
}
