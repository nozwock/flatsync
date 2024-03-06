use crate::application::FlatsyncApplication;
use crate::glib::clone;
use crate::glib::MainContext;
use crate::network_state::NetworkState;
use crate::power_state::PowerState;
use adw::prelude::*;
use adw::subclass::prelude::AdwApplicationWindowImpl;
use adw::subclass::prelude::AdwWindowImpl;
use gettextrs::gettext;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libflatsync_common::config::{APP_ID, PROFILE};
use tokio::runtime::Runtime;

mod imp {
    use super::*;
    use libflatsync_common::dbus::DaemonProxy;
    use std::cell::OnceCell;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/app/drey/FlatSync/ui/window.ui")]
    pub struct FlatsyncApplicationWindow {
        #[template_child]
        pub autosync_status: TemplateChild<adw::Banner>,
        #[template_child]
        pub welcome_status: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub sync_now_button: TemplateChild<gtk::Button>,
        pub settings: gio::Settings,
        pub network_monitor: gio::NetworkMonitor,
        pub power_profile_monitor: gio::PowerProfileMonitor,
        pub proxy: OnceCell<DaemonProxy<'static>>,
        pub tokio_runtime: OnceCell<Runtime>,
    }

    impl Default for FlatsyncApplicationWindow {
        fn default() -> Self {
            Self {
                autosync_status: TemplateChild::default(),
                welcome_status: TemplateChild::default(),
                sync_now_button: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                network_monitor: gio::NetworkMonitor::default(),
                power_profile_monitor: gio::PowerProfileMonitor::get_default(),
                proxy: OnceCell::new(),
                tokio_runtime: OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatsyncApplicationWindow {
        const NAME: &'static str = "FlatsyncApplicationWindow";
        type Type = super::FlatsyncApplicationWindow;
        type ParentType = adw::ApplicationWindow;

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
            // obj.setup_settings_values();

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

            //TODO: Load Title and Summary dynamically from metadata
            self.welcome_status.set_title("FlatSync");
            self.welcome_status.set_description(Some(&gettext(
                "Keep your Flatpak apps synchronized between devices",
            )));
            self.welcome_status.set_icon_name(Some(APP_ID));

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
    impl AdwWindowImpl for FlatsyncApplicationWindow {}
    impl AdwApplicationWindowImpl for FlatsyncApplicationWindow {}
}

glib::wrapper! {
    pub struct FlatsyncApplicationWindow(ObjectSubclass<imp::FlatsyncApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::Window,
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

        imp.network_monitor
            .connect_network_changed(clone!(@weak self as obj => move |_, _| {
                obj.update_autosync_status();
            }));

        imp.power_profile_monitor
            .connect_power_saver_enabled_notify(clone!(@weak self as obj => move |_| {
                obj.update_autosync_status();
            }));

        imp.settings.connect_changed(
            Some("autosync"),
            clone!(@weak self as obj => move |_,_| {
                obj.update_autosync_status();
            }),
        );
        // The Setting Key only emits the `changed` signal if it has been read after the listener has been setted up
        imp.settings.get::<bool>("autosync");

        imp.sync_now_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                let ctx = MainContext::default();
                ctx.spawn_local(clone!(@weak obj => async move {
                    let _ = obj.imp().proxy.get().unwrap().sync_now().await;
                }));
            }));
    }

    fn network_state(&self) -> NetworkState {
        let imp = self.imp();

        let network_is_metered = imp.network_monitor.is_network_metered();
        let network_is_available = imp.network_monitor.is_network_available();

        match (network_is_metered, network_is_available) {
            (_, false) => NetworkState::NoNetwork,
            (false, true) => NetworkState::Ok,
            (true, _) => NetworkState::NetworkMetered,
        }
    }

    fn power_state(&self) -> PowerState {
        let imp = self.imp();

        match imp.power_profile_monitor.is_power_saver_enabled() {
            true => PowerState::PowerSaver,
            false => PowerState::Ok,
        }
    }

    fn update_sync_now_button(&self, network_monitor_state: NetworkState) {
        let imp = self.imp();

        match network_monitor_state {
            NetworkState::NoNetwork => {
                imp.sync_now_button.set_sensitive(false);
            }
            _ => {
                imp.sync_now_button.set_sensitive(true);
            }
        }
    }

    fn update_autosync_status(&self) {
        let imp = self.imp();

        let network_monitor_state = self.network_state();
        let power_profiles_monitor_state = self.power_state();
        let is_autosync_enabled = imp.settings.get::<bool>("autosync");

        self.update_sync_now_button(network_monitor_state);

        let title: Option<String> = match (
            network_monitor_state,
            power_profiles_monitor_state,
            is_autosync_enabled,
        ) {
            (NetworkState::Ok, PowerState::Ok, _) => None,
            (NetworkState::Ok, PowerState::PowerSaver, true) => {
                Some(gettext("Autosync disabled: power saver is on"))
            }
            (NetworkState::NoNetwork, PowerState::Ok, true) => {
                Some(gettext("Autosync disabled: system is offline"))
            }
            (NetworkState::NoNetwork, PowerState::PowerSaver, true) => Some(gettext(
                "Autosync disabled: system is offline, power saver is on",
            )),
            (NetworkState::NoNetwork, _, false) => Some(gettext("System is offline")),
            (NetworkState::NetworkMetered, PowerState::Ok, true) => {
                Some(gettext("Autosync disabled: network is metered"))
            }
            (NetworkState::NetworkMetered, PowerState::PowerSaver, true) => Some(gettext(
                "Autosync disabled: network is metered, power saver is on",
            )),
            (_, _, _) => None,
        };

        if let Some(title) = title {
            imp.autosync_status.set_title(&title);
            imp.autosync_status.set_revealed(true);
        } else {
            imp.autosync_status.set_revealed(false);
        }
    }

    // fn setup_settings_values(&self) {
    //     let imp = self.imp();
    // }
}
