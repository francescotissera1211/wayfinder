mod imp;

use gtk::gio;
use gtk::glib;

glib::wrapper! {
    pub struct WayfinderApplication(ObjectSubclass<imp::WayfinderApplicationInner>)
        @extends gtk::Application, gtk::gio::Application,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap;
}

impl WayfinderApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "org.wayfinder.FileManager")
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .build()
    }
}
