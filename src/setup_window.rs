use adw::prelude::*;
use adw::subclass::prelude::*;

use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use gtk::glib::clone;
use gtk::{gio, glib, Box, ListBox, Orientation, SelectionMode};

mod imp {
    use gtk::glib::{subclass::types::FromObject, value::FromValue};

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(file = "../data/resources/ui/setup-window.ui")]
    pub struct FlatsyncSetupWindow {
        #[template_child]
        pub navigation: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub page_home: TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub page_provider: TemplateChild<adw::NavigationPage>,
        #[template_child]
        pub push_provider_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FlatsyncSetupWindow {
        const NAME: &'static str = "FlatsyncSetupWindow";
        type Type = super::FlatsyncSetupWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FlatsyncSetupWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.connect_handlers();
        }
    }
    impl WidgetImpl for FlatsyncSetupWindow {}
    impl WindowImpl for FlatsyncSetupWindow {}
    impl ApplicationWindowImpl for FlatsyncSetupWindow {}
    impl AdwApplicationWindowImpl for FlatsyncSetupWindow {}
}

glib::wrapper! {
    pub struct FlatsyncSetupWindow(ObjectSubclass<imp::FlatsyncSetupWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl FlatsyncSetupWindow {
    pub fn new<App: IsA<gtk::Application>>(app: &App) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn connect_handlers(&self) {
        let imp = self.imp();

        imp.push_provider_button
            .connect_clicked(clone!(@weak imp => move |_| {
                imp.navigation.push(imp.page_provider.downcast_ref::<adw::NavigationPage>().unwrap());
            }));
    }
}
