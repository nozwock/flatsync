mod application;
mod network_state;
mod power_state;
mod preferences;
mod window;

use self::application::FlatsyncApplication;
use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};
use libflatsync_common::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() -> glib::ExitCode {
    // Initialize logger
    pretty_env_logger::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("FlatSync"));

    let res = gio::Resource::load(RESOURCES_FILE)
        .expect("Could not load gresource file - did you run ninja -C build install?");
    gio::resources_register(&res);

    let app = FlatsyncApplication::default();
    app.run()
}
