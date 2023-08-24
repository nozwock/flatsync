use crate::application::FlatsyncApplication;
use adw::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
};
use libflatsync_common::config::{APP_ID, PROFILE};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/app/drey/FlatSync/ui/window.ui")]
    pub struct FlatsyncApplicationWindow {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub github_token_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub github_id_entry: TemplateChild<adw::EntryRow>,
        pub settings: gio::Settings,
    }

    impl Default for FlatsyncApplicationWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                github_token_entry: TemplateChild::default(),
                github_id_entry: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatsyncApplicationWindow {
        const NAME: &'static str = "FlatsyncApplicationWindow";
        type Type = super::FlatsyncApplicationWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FlatsyncApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();
            obj.connect_handlers();
        }
    }

    impl WidgetImpl for FlatsyncApplicationWindow {}
    impl WindowImpl for FlatsyncApplicationWindow {
        // Save window state on delete event
        fn close_request(&self) -> gtk::Inhibit {
            if let Err(err) = self.obj().save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for FlatsyncApplicationWindow {}
}

glib::wrapper! {
    pub struct FlatsyncApplicationWindow(ObjectSubclass<imp::FlatsyncApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl FlatsyncApplicationWindow {
    pub fn new(app: &FlatsyncApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    fn connect_handlers(&self) {
        let imp = self.imp();
        imp.github_id_entry
            .connect_apply(clone!(@weak self as obj => move |entry| {
                let text = entry.text();
                obj.imp().settings.set_string("github-id", text.as_str()).unwrap();
            }));
        imp.github_token_entry
            .connect_apply(clone!(@weak self as obj => move |entry| {
                let text = entry.text();
                obj.imp().settings.set_string("github-token", text.as_str()).unwrap();
            }));
    }
}
