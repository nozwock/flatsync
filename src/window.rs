use crate::application::FlatsyncApplication;
use adw::prelude::*;
use gtk::glib::MainContext;
use gtk::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
};
use libflatsync_common::config::{APP_ID, PROFILE};
use libflatsync_common::dbus::DaemonProxy;
use log::error;
use tokio::runtime::Runtime;

mod imp {
    use super::*;
    use libflatsync_common::dbus::DaemonProxy;
    use std::cell::OnceCell;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/app/drey/FlatSync/ui/window.ui")]
    pub struct FlatsyncApplicationWindow {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub github_token_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub github_id_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub autosync_entry: TemplateChild<adw::SwitchRow>,
        pub settings: gio::Settings,
        pub proxy: OnceCell<DaemonProxy<'static>>,
        pub tokio_runtime: OnceCell<Runtime>,
    }

    impl Default for FlatsyncApplicationWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                github_token_entry: TemplateChild::default(),
                github_id_entry: TemplateChild::default(),
                autosync_entry: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                proxy: OnceCell::new(),
                tokio_runtime: OnceCell::new(),
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
            self.tokio_runtime.set(Runtime::new().unwrap()).unwrap();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load initial settings
            obj.setup_settings_values();

            // Init zbus proxy
            let (sender, mut reciever) = tokio::sync::mpsc::channel::<DaemonProxy>(1);

            self.tokio_runtime.get().unwrap().spawn(async move {
                let connection = zbus::Connection::session().await.unwrap();
                let proxy = libflatsync_common::dbus::DaemonProxy::new(&connection)
                    .await
                    .unwrap();

                sender.send(proxy).await.unwrap();
            });

            self.proxy.set(reciever.blocking_recv().unwrap()).unwrap();

            // Load latest window state
            obj.load_window_size();
            obj.connect_handlers();
        }
    }

    impl WidgetImpl for FlatsyncApplicationWindow {}
    impl WindowImpl for FlatsyncApplicationWindow {
        // Save window state on delete event
        fn close_request(&self) -> glib::Propagation {
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

    fn proxy(&self) -> &DaemonProxy<'static> {
        self.imp().proxy.get().unwrap()
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
                let ctx = MainContext::default();
                let text = entry.text();
                ctx.spawn_local(clone!(@weak obj => async move {
                    if let Err(e) = obj.proxy().set_gist_id(text.as_str()).await {
                        error!("{e}");
                    }
                }));
            }));
        imp.github_token_entry
            .connect_apply(clone!(@weak self as obj => move |entry| {
                let ctx = MainContext::default();
                let text = entry.text();
                ctx.spawn_local(clone!(@weak obj => async move {
                    if let Err(e) = obj.proxy().set_gist_secret(text.as_str()).await {
                        error!("{e}");
                    }
                }));
            }));
    }

    fn setup_settings_values(&self) {
        let imp = self.imp();

        imp.github_id_entry
            .set_text(&imp.settings.get::<String>("github-gists-id"));

        imp.github_token_entry.set_text("1234");

        imp.settings
            .bind("autosync", &imp.autosync_entry.get(), "active")
            .build();
    }
}
