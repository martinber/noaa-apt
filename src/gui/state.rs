//! Code related to managing the state of the GUI.
//!
//! This includes a widget list and a state struct.

use std::cell::RefCell;
use std::path::Path;

use gtk::prelude::*;

use crate::config::Settings;
use crate::dsp::Signal;
use crate::noaa_apt::Image;


// Stores the Widgets.
//
// Use the functions below when accesing it. Only available from the GUI thread.
// Wrapped on Option because it's None before building the GUI.
// Wrapped on RefCell because I need mutable references when building the GUI.
thread_local!(static GLOBAL_WIDGETS: RefCell<Option<Widgets>> = RefCell::new(None));

// Stores the GuiState.
//
// Use the functions below when accesing it. Only available from the GUI thread.
// Wrapped on Option because it's None before building the GUI.
// Wrapped on RefCell because I need mutable references when saving state.
thread_local!(static GLOBAL_STATE: RefCell<Option<GuiState>> = RefCell::new(None));


/// Work with mutable reference to GuiState.
///
/// Panics if called from a thread different than the GUI one. And also if the
/// state was already borrowed. So the closures used should be as small as
/// possible to avoid borrowing twice.
pub fn borrow_state_mut<F, R>(f: F) -> R
where F: FnOnce(&mut GuiState) -> R
{
    GLOBAL_STATE.with(|global| {
        if let Some(ref mut state) = *global.borrow_mut() {
            (f)(state)
        } else {
            panic!("Can't get GuiState. Tried to borrow from another thread \
                    or tried to borrow before building the GUI")
        }
    })
}

/// Work with reference to GuiState.
///
/// Panics if called from a thread different than the GUI one. And also if the
/// state was already borrowed mutably. So the closures used should be as small
/// as possible to avoid borrowing twice.
pub fn borrow_state<F, R>(f: F) -> R
where F: FnOnce(&GuiState) -> R
{
    GLOBAL_STATE.with(|global| {
        if let Some(ref state) = *global.borrow() {
            (f)(state)
        } else {
            panic!("Can't get GuiState. Tried to borrow from another thread \
                    or tried to borrow before building the GUI")
        }
    })
}

/// Set the Widgets.
///
/// Called when building the GUI.
pub fn set_state(state: GuiState) {
    GLOBAL_STATE.with(|global| {
        *global.borrow_mut() = Some(state);
    });
}

/// Work with reference to Widgets.
///
/// Panics if called from a thread different than the GUI one. Also panics if
/// the GUI is not built yet.
pub fn borrow_widgets<F, R>(f: F) -> R
where F: FnOnce(&Widgets) -> R
{
    GLOBAL_WIDGETS.with(|global| {
        if let Some(ref widgets) = *global.borrow() {
            (f)(widgets)
        } else {
            panic!("Can't get Widgets. Tried to borrow from another thread \
                    or tried to borrow before building the GUI")
        }
    })
}

/// Set the Widgets.
///
/// Called when building the GUI.
pub fn set_widgets(widgets: Widgets) {
    GLOBAL_WIDGETS.with(|global| {
        *global.borrow_mut() = Some(widgets);
    });
}

/// Contains changing state.
///
/// For instance it keeps track of the decoded image.
#[derive(Debug, Clone)]
pub struct GuiState {
    pub settings:                  Settings,
    pub decoded_signal:            Option<Signal>,
    pub processed_image:           Option<Image>,
}

/// Contains references to widgets and some fixed objects.
///
/// Some used prefixes:
/// - img: Related to the image panel.
/// - main: Some widgets that are almost always visible.
/// - info: Related to a popup bar used for warnings and errors.
/// - dec: Decoding tab.
/// - p: Processing tab.
/// - sav: Saving tab.
/// - res: Resample tool.
/// - ts: Timesamp tool.
#[derive(Debug, Clone)]
pub struct Widgets {
    pub application:               gtk::Application,
    pub window:                    gtk::ApplicationWindow,
    pub outer_box:                 gtk::Box,

    pub dec_action:                gio::SimpleAction,
    pub res_action:                gio::SimpleAction,
    pub ts_action:                 gio::SimpleAction,

    pub info_bar:                  gtk::InfoBar,
    pub info_label:                gtk::Label,
    pub info_revealer:             gtk::Revealer,

    pub img_scroll:                gtk::ScrolledWindow,
    pub img_viewport:              gtk::Viewport,
    pub img_image:                 gtk::Image,
    pub img_size_toggle:           gtk::ToggleButton,
    pub img_def_pixbuf:            gdk_pixbuf::Pixbuf,

    pub main_paned:                gtk::Paned,
    pub main_progress_bar:         gtk::ProgressBar,
    pub main_stack:                gtk::Stack,
    pub dec_stack_child:           gtk::Notebook,
    pub res_stack_child:           gtk::Box,
    pub ts_stack_child:            gtk::Box,

    pub dec_input_chooser:         gtk::FileChooserButton,
    pub dec_sync_check:            gtk::CheckButton,
    pub dec_wav_steps_check:       gtk::CheckButton,
    pub dec_resample_step_check:   gtk::CheckButton,
    pub dec_decode_button:         gtk::Button,

    pub p_contrast_combo:          gtk::ComboBoxText,
    pub p_rotate_combo:            gtk::ComboBoxText,
    pub p_satellite_combo:         gtk::ComboBoxText,
    pub p_custom_tle_check:        gtk::CheckButton,
    pub p_custom_tle_chooser:      gtk::FileChooserButton,
    pub p_ref_time_combo:          gtk::ComboBoxText,
    pub p_hs_spinner:              gtk::SpinButton,
    pub p_min_spinner:             gtk::SpinButton,
    pub p_sec_spinner:             gtk::SpinButton,
    pub p_timezone_label:          gtk::Label,
    pub p_calendar:                gtk::Calendar,
    pub p_overlay_check:           gtk::CheckButton,
    pub p_yaw_spinner:             gtk::SpinButton,
    pub p_vscale_spinner:          gtk::SpinButton,
    pub p_hscale_spinner:          gtk::SpinButton,
    pub p_process_button:          gtk::Button,

    pub sav_output_entry:          gtk::Entry,
    pub sav_folder_tip_box:        gtk::Box,
    pub sav_folder_tip_label:      gtk::Label,
    pub sav_extension_tip_label:   gtk::Label,
    pub sav_overwrite_tip_label:   gtk::Label,
    pub sav_save_button:           gtk::Button,

    pub res_input_chooser:         gtk::FileChooserButton,
    pub res_output_entry:          gtk::Entry,
    pub res_rate_spinner:          gtk::SpinButton,
    pub res_folder_tip_box:        gtk::Box,
    pub res_folder_tip_label:      gtk::Label,
    pub res_extension_tip_label:   gtk::Label,
    pub res_overwrite_tip_label:   gtk::Label,
    pub res_wav_steps_check:       gtk::CheckButton,
    pub res_resample_step_check:   gtk::CheckButton,

    pub ts_read_chooser:           gtk::FileChooserButton,
    pub ts_read_button:            gtk::Button,
    pub ts_write_chooser:          gtk::FileChooserButton,
    pub ts_write_button:           gtk::Button,
    pub ts_hs_spinner:             gtk::SpinButton,
    pub ts_min_spinner:            gtk::SpinButton,
    pub ts_sec_spinner:            gtk::SpinButton,
    pub ts_timezone_label:         gtk::Label,
    pub ts_calendar:               gtk::Calendar,
}

impl Widgets {
    /// Create from widgets on Glade builder and create the rest.
    pub fn from_builder(
        builder: &gtk::Builder,
        window: &gtk::ApplicationWindow,
        application: &gtk::Application,
    ) -> Self {

        Self {
            application:             application.clone(),
            window:                  window.clone(),
            outer_box:               gtk::Box::new(gtk::Orientation::Vertical, 0),

            dec_action:              gio::SimpleAction::new("decode", None),
            res_action:              gio::SimpleAction::new("resample", None),
            ts_action:               gio::SimpleAction::new("timestamp", None),

            info_bar:                gtk::InfoBar::new(),
            info_label:              gtk::Label::new(None),
            info_revealer:           gtk::Revealer::new(),

            img_scroll:              builder.get_object("img_scroll"             ).expect("Couldn't get img_scroll"             ),
            img_viewport:            builder.get_object("img_viewport"           ).expect("Couldn't get img_viewport"           ),
            img_image:               builder.get_object("img_image"              ).expect("Couldn't get img_image"              ),
            img_size_toggle:         builder.get_object("img_size_toggle"        ).expect("Couldn't get img_size_toggle"        ),

            // TODO: See path
            img_def_pixbuf:          gdk_pixbuf::Pixbuf::new_from_file(Path::new("./res/icon.png")).expect("Couldn't load ./res/icon.png"),

            main_paned:              builder.get_object("main_paned"             ).expect("Couldn't get main_paned"             ),
            main_progress_bar:       builder.get_object("main_progress_bar"      ).expect("Couldn't get main_progress_bar"      ),
            main_stack:              builder.get_object("main_stack"             ).expect("Couldn't get main_stack"             ),
            dec_stack_child:         builder.get_object("dec_stack_child"        ).expect("Couldn't get dec_stack_child"        ),
            res_stack_child:         builder.get_object("res_stack_child"        ).expect("Couldn't get res_stack_child"        ),
            ts_stack_child:          builder.get_object("ts_stack_child"         ).expect("Couldn't get ts_stack_child"         ),

            dec_input_chooser:       builder.get_object("dec_input_chooser"      ).expect("Couldn't get dec_input_chooser"      ),
            dec_sync_check:          builder.get_object("dec_sync_check"         ).expect("Couldn't get dec_sync_check"         ),
            dec_wav_steps_check:     builder.get_object("dec_wav_steps_check"    ).expect("Couldn't get dec_wav_steps_check"    ),
            dec_resample_step_check: builder.get_object("dec_resample_step_check").expect("Couldn't get dec_resample_step_check"),
            dec_decode_button:       builder.get_object("dec_decode_button"      ).expect("Couldn't get dec_decode_button"      ),

            p_contrast_combo:        builder.get_object("p_contrast_combo"       ).expect("Couldn't get p_contrast_combo"       ),
            p_rotate_combo:          builder.get_object("p_rotate_combo"         ).expect("Couldn't get p_rotate_combo"         ),
            p_satellite_combo:       builder.get_object("p_satellite_combo"      ).expect("Couldn't get p_satellite_combo"      ),
            p_custom_tle_check:      builder.get_object("p_custom_tle_check"     ).expect("Couldn't get p_custom_tle_check"     ),
            p_custom_tle_chooser:    builder.get_object("p_custom_tle_chooser"   ).expect("Couldn't get p_custom_tle_chooser"   ),
            p_ref_time_combo:        builder.get_object("p_ref_time_combo"       ).expect("Couldn't get p_ref_time_combo"       ),
            p_hs_spinner:            builder.get_object("p_hs_spinner"           ).expect("Couldn't get p_hs_spinner"           ),
            p_min_spinner:           builder.get_object("p_min_spinner"          ).expect("Couldn't get p_min_spinner"          ),
            p_sec_spinner:           builder.get_object("p_sec_spinner"          ).expect("Couldn't get p_sec_spinner"          ),
            p_timezone_label:        builder.get_object("p_timezone_label"       ).expect("Couldn't get p_timezone_label"       ),
            p_calendar:              builder.get_object("p_calendar"             ).expect("Couldn't get p_calendar"             ),
            p_overlay_check:         builder.get_object("p_overlay_check"        ).expect("Couldn't get p_overlay_check"        ),
            p_yaw_spinner:           builder.get_object("p_yaw_spinner"          ).expect("Couldn't get p_yaw_spinner"          ),
            p_vscale_spinner:        builder.get_object("p_vscale_spinner"       ).expect("Couldn't get p_vscale_spinner"       ),
            p_hscale_spinner:        builder.get_object("p_hscale_spinner"       ).expect("Couldn't get p_hscale_spinner"       ),
            p_process_button:        builder.get_object("p_process_button"       ).expect("Couldn't get p_process_button"       ),

            sav_output_entry:        builder.get_object("sav_output_entry"       ).expect("Couldn't get sav_output_entry"       ),
            sav_folder_tip_box:      builder.get_object("sav_folder_tip_box"     ).expect("Couldn't get sav_folder_tip_box"     ),
            sav_folder_tip_label:    builder.get_object("sav_folder_tip_label"   ).expect("Couldn't get sav_folder_tip_label"   ),
            sav_extension_tip_label: builder.get_object("sav_extension_tip_label").expect("Couldn't get sav_extension_tip_label"),
            sav_overwrite_tip_label: builder.get_object("sav_overwrite_tip_label").expect("Couldn't get sav_overwrite_tip_label"),
            sav_save_button:         builder.get_object("sav_save_button"        ).expect("Couldn't get sav_save_button"        ),

            res_input_chooser:       builder.get_object("res_input_chooser"      ).expect("Couldn't get res_input_chooser"      ),
            res_output_entry:        builder.get_object("res_output_entry"       ).expect("Couldn't get res_output_entry"       ),
            res_rate_spinner:        builder.get_object("res_rate_spinner"       ).expect("Couldn't get res_rate_spinner"       ),
            res_folder_tip_box:      builder.get_object("res_folder_tip_box"     ).expect("Couldn't get res_folder_tip_box"     ),
            res_folder_tip_label:    builder.get_object("res_folder_tip_label"   ).expect("Couldn't get res_folder_tip_label"   ),
            res_extension_tip_label: builder.get_object("res_extension_tip_label").expect("Couldn't get res_extension_tip_label"),
            res_overwrite_tip_label: builder.get_object("res_overwrite_tip_label").expect("Couldn't get res_overwrite_tip_label"),
            res_wav_steps_check:     builder.get_object("res_wav_steps_check"    ).expect("Couldn't get res_wav_steps_check"    ),
            res_resample_step_check: builder.get_object("res_resample_step_check").expect("Couldn't get res_resample_step_check"),

            ts_read_chooser:         builder.get_object("ts_read_chooser"        ).expect("Couldn't get ts_read_chooser"        ),
            ts_read_button:          builder.get_object("ts_read_button"         ).expect("Couldn't get ts_read_button"         ),
            ts_write_chooser:        builder.get_object("ts_write_chooser"       ).expect("Couldn't get ts_write_chooser"       ),
            ts_write_button:         builder.get_object("ts_write_button"        ).expect("Couldn't get ts_write_button"        ),
            ts_hs_spinner:           builder.get_object("ts_hs_spinner"          ).expect("Couldn't get ts_hs_spinner"          ),
            ts_min_spinner:          builder.get_object("ts_min_spinner"         ).expect("Couldn't get ts_min_spinner"         ),
            ts_sec_spinner:          builder.get_object("ts_sec_spinner"         ).expect("Couldn't get ts_sec_spinner"         ),
            ts_timezone_label:       builder.get_object("ts_timezone_label"      ).expect("Couldn't get ts_timezone_label"      ),
            ts_calendar:             builder.get_object("ts_calendar"            ).expect("Couldn't get ts_calendar"            ),
        }

    }
}
