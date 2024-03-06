use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::FromVariant;
use gtk::glib::MainContext;
use gtk::{
    gio,
    glib::{self, clone, Boxed, Properties},
};
use libflatsync_common::config::{APP_ID, PROFILE};
use libflatsync_common::dbus::DaemonProxy;
use log::error;

#[derive(Boxed, Clone, Debug)]
#[boxed_type(name = "ProxyProp")]
pub struct ProxyProp(DaemonProxy<'static>);

mod imp {
    use super::*;

    use std::cell::OnceCell;

    #[derive(Debug, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::FlatsyncPreferencesWindow)]
    #[template(resource = "/app/drey/FlatSync/ui/preferences.ui")]
    pub struct FlatsyncPreferencesWindow {
        #[template_child]
        pub github_token_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub github_id_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub autosync_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub autosync_timer_spin: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub autosync_timer_adjustment: TemplateChild<gtk::Adjustment>,
        pub settings: gio::Settings,
        #[property(construct_only, name = "proxy")]
        pub proxy: OnceCell<ProxyProp>,
    }

    impl Default for FlatsyncPreferencesWindow {
        fn default() -> Self {
            Self {
                github_token_entry: TemplateChild::default(),
                github_id_entry: TemplateChild::default(),
                autosync_switch: TemplateChild::default(),
                autosync_timer_spin: TemplateChild::default(),
                autosync_timer_adjustment: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                proxy: OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatsyncPreferencesWindow {
        const NAME: &'static str = "FlatsyncPreferencesWindow";
        type Type = super::FlatsyncPreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FlatsyncPreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.connect_handlers();
            obj.setup_settings_values();
        }
    }
    impl WidgetImpl for FlatsyncPreferencesWindow {}
    impl WindowImpl for FlatsyncPreferencesWindow {}
    impl ApplicationWindowImpl for FlatsyncPreferencesWindow {}
    impl AdwWindowImpl for FlatsyncPreferencesWindow {}
    impl PreferencesWindowImpl for FlatsyncPreferencesWindow {}
}

glib::wrapper! {
    pub struct FlatsyncPreferencesWindow(ObjectSubclass<imp::FlatsyncPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::Window, adw::PreferencesWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl FlatsyncPreferencesWindow {
    pub fn new<Window: glib::IsA<gtk::Window>>(
        parent: &Window,
        proxy: DaemonProxy<'static>,
    ) -> Self {
        glib::Object::builder::<Self>()
            .property("transient-for", Some(parent))
            .property("proxy", ProxyProp(proxy))
            .build()
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

    fn proxy(&self) -> &DaemonProxy<'static> {
        &self.imp().proxy.get().unwrap().0
    }

    fn setup_settings_values(&self) {
        let imp = self.imp();

        imp.github_id_entry
            .set_text(&imp.settings.get::<String>("github-gists-id"));

        imp.github_token_entry.set_text("1234");

        imp.settings
            .bind("autosync", &imp.autosync_switch.get(), "active")
            .build();

        let autosync_timer_key = imp
            .settings
            .settings_schema()
            .unwrap()
            .key("autosync-timer");
        let range_variant = autosync_timer_key.range().child_value(1).child_value(0);
        let range = <(u32, u32)>::from_variant(&range_variant).unwrap();

        imp.autosync_timer_adjustment.set_lower(range.0.into());
        imp.autosync_timer_adjustment.set_upper(range.1.into());

        imp.settings
            .bind("autosync-timer", &imp.autosync_timer_spin.get(), "value")
            .build();
    }
}
