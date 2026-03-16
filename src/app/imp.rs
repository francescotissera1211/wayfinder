use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::window::WayfinderWindow;

#[derive(Default)]
pub struct WayfinderApplicationInner;

#[glib::object_subclass]
impl ObjectSubclass for WayfinderApplicationInner {
    const NAME: &'static str = "WayfinderApplication";
    type Type = super::WayfinderApplication;
    type ParentType = gtk::Application;
}

impl ObjectImpl for WayfinderApplicationInner {}

impl ApplicationImpl for WayfinderApplicationInner {
    fn activate(&self) {
        let app = self.obj();
        let window = WayfinderWindow::new(app.upcast_ref());
        window.present();
    }

    fn open(&self, files: &[gio::File], _hint: &str) {
        let app = self.obj();

        for file in files {
            let window = WayfinderWindow::new(app.upcast_ref());

            if let Some(path) = file.path() {
                let path_str = path.to_string_lossy().to_string();
                if path.is_dir() {
                    // It's a directory — navigate to it
                    window.navigate_to_path(&path_str);
                } else if path.is_file() {
                    // It's a file — navigate to its parent and open it
                    if let Some(parent) = path.parent() {
                        window.navigate_to_path(&parent.to_string_lossy());
                    }
                    // Open the file with its associated app
                    if let Some(file_obj) = window.get_selected_file() {
                        window.open_file(&file_obj);
                    }
                } else {
                    // Path doesn't exist yet — try navigating (will walk up)
                    window.navigate_to_path(&path_str);
                }
            }

            window.present();
        }
    }
}

impl GtkApplicationImpl for WayfinderApplicationInner {}
