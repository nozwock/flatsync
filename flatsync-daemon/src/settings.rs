use glib::{IsA, Variant};
use libflatpak::gio::{self, prelude::*};

#[derive(Clone, Debug)]
pub struct Settings(gio::Settings);

static mut SETTINGS: Option<Settings> = None;

impl Default for Settings {
    fn default() -> Self {
        Self::instance()
    }
}

impl Settings {
    pub fn instance() -> Self {
        unsafe {
            SETTINGS.as_ref().map_or_else(
                || {
                    let settings = Self(gio::Settings::new("app.drey.FlatSync.Devel"));
                    SETTINGS = Some(settings.clone());
                    settings
                },
                std::clone::Clone::clone,
            )
        }
    }

    pub fn get_gist_id(&self) -> Option<String> {
        let res: String = self.get("gist-id");

        match res.as_str() {
            "" => None,
            _ => Some(res),
        }
    }

    pub fn set_gist_id(&self, gist_id: &str) {
        self.set("gist-id", gist_id).unwrap();
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

            fn get<U: glib::FromVariant>(&self, key: &str) -> U;

            fn set(&self, key: &str, value: impl Into<Variant>) -> Result<(), glib::BoolError>;

            fn set_enum(&self, key: &str, value: i32) -> Result<(), glib::BoolError>;

            fn set_strv(&self, key: &str, value: &[&str]) -> Result<(), glib::BoolError>;

            fn set_string(&self, key: &str, value: &str) -> Result<(), glib::BoolError>;

            fn string(&self, key: &str) -> glib::GString;

            fn strv(&self, key: &str) -> glib::StrV;
        }
    }
}
