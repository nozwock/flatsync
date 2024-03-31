use glib::{IsA, Variant};
use libflatpak::gio::{self, prelude::*};
use std::sync::Once;

#[derive(Clone, Debug)]
pub struct Settings(gio::Settings);

static mut SETTINGS: Option<Settings> = None;
static SETTINGS_INIT: Once = Once::new();

impl Default for Settings {
    fn default() -> Self {
        Self::instance()
    }
}

impl Settings {
    pub fn instance() -> Self {
        unsafe {
            SETTINGS_INIT.call_once(|| {
                SETTINGS = Some(Self(gio::Settings::new("app.drey.FlatSync.Devel")));
            });
            SETTINGS
                .clone()
                .expect("SETTINGS is initialised before clone()")
        }
    }
}

#[allow(dead_code)]
impl Settings {
    delegate::delegate! {
        to self.0 {
            pub fn bind<'a, P: IsA<glib::Object>>(
                &'a self,
                key: &'a str,
                object: &'a P,
                property: &'a str
            ) -> gio::BindingBuilder<'a>;

            pub fn connect_changed<F: Fn(&gio::Settings, &str) + 'static>(
                &self,
                detail: Option<&str>,
                f: F
            ) -> glib::SignalHandlerId;

            pub fn disconnect(&self, handler_id: glib::SignalHandlerId);

            fn enum_(&self, key: &str) -> i32;

            pub fn get<U: glib::FromVariant>(&self, key: &str) -> U;

            pub fn set(&self, key: &str, value: impl Into<Variant>) -> Result<(), glib::BoolError>;

            fn set_enum(&self, key: &str, value: i32) -> Result<(), glib::BoolError>;

            fn set_strv(&self, key: &str, value: &[&str]) -> Result<(), glib::BoolError>;

            fn set_string(&self, key: &str, value: &str) -> Result<(), glib::BoolError>;

            fn string(&self, key: &str) -> glib::GString;

            fn strv(&self, key: &str) -> glib::StrV;
        }
    }
}
