use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    prelude::*,
    {gdk, gio, glib},
};
use libflatsync_common::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use log::{debug, info};

use crate::preferences::FlatsyncPreferencesWindow;
use crate::window::FlatsyncApplicationWindow;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    #[derive(Debug, Default)]
    pub struct FlatsyncApplication {
        pub window: OnceCell<WeakRef<FlatsyncApplicationWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatsyncApplication {
        const NAME: &'static str = "FlatsyncApplication";
        type Type = super::FlatsyncApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for FlatsyncApplication {}

    impl ApplicationImpl for FlatsyncApplication {
        fn activate(&self) {
            debug!("GtkApplication<FlatsyncApplication>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = FlatsyncApplicationWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("GtkApplication<FlatsyncApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for FlatsyncApplication {}
    impl AdwApplicationImpl for FlatsyncApplication {}
}

glib::wrapper! {
    pub struct FlatsyncApplication(ObjectSubclass<imp::FlatsyncApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl FlatsyncApplication {
    fn main_window(&self) -> FlatsyncApplicationWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_window();
            })
            .build();
        // Preferences
        let action_preferences = gio::ActionEntry::builder("preferences")
            .activate(|app: &Self, _, _| {
                app.show_preferences_window();
            })
            .build();
        self.add_action_entries([action_quit, action_about, action_preferences]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
        self.set_accels_for_action("app.preferences", &["<Control>p"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/app/drey/FlatSync/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_window(&self) {
        let about_window = adw::AboutWindow::builder()
            .application_icon(APP_ID)
            .application_name("Flatsync")
            .developer_name("Rasmus Thomsen")
            .version(VERSION)
            // TODO: Add License
            .transient_for(&self.main_window())
            .translator_credits(gettext("translator-credits"))
            .modal(true)
            .developers(vec!["Rasmus Thomsen"])
            .artists(vec!["Rasmus Thomsen"])
            .website("https://gitlab.gnome.org/Cogitri/flatsync")
            .issue_url("https://gitlab.gnome.org/Cogitri/flatsync/-/issues")
            .build();

        about_window.present();
    }

    fn show_preferences_window(&self) {
        let main_window = &self.main_window();

        let preferences_window = FlatsyncPreferencesWindow::new(
            main_window,
            main_window.imp().proxy.get().unwrap().clone(),
        );

        preferences_window.present();
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("FlatSync ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for FlatsyncApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/app/drey/FlatSync/")
            .build()
    }
}
