mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct WayfinderSidebar(ObjectSubclass<imp::SidebarInner>);
}

struct PlaceEntry {
    name: &'static str,
    icon: &'static str,
    dir_fn: fn() -> Option<std::path::PathBuf>,
}

const PLACES: &[PlaceEntry] = &[
    PlaceEntry { name: "Home", icon: "user-home-symbolic", dir_fn: dirs::home_dir },
    PlaceEntry { name: "Desktop", icon: "user-desktop-symbolic", dir_fn: dirs::desktop_dir },
    PlaceEntry { name: "Documents", icon: "folder-documents-symbolic", dir_fn: dirs::document_dir },
    PlaceEntry { name: "Downloads", icon: "folder-download-symbolic", dir_fn: dirs::download_dir },
    PlaceEntry { name: "Music", icon: "folder-music-symbolic", dir_fn: dirs::audio_dir },
    PlaceEntry { name: "Pictures", icon: "folder-pictures-symbolic", dir_fn: dirs::picture_dir },
    PlaceEntry { name: "Videos", icon: "folder-videos-symbolic", dir_fn: dirs::video_dir },
];

impl WayfinderSidebar {
    pub fn new() -> Self {
        let sidebar: Self = glib::Object::builder().build();
        sidebar.populate_places();
        sidebar
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.imp().container
    }

    pub fn connect_place_activated<F: Fn(String) + 'static>(&self, callback: F) {
        self.imp().places_list.connect_row_activated(move |_list, row| {
            if let Some(path) = row.widget_name().strip_prefix("place:") {
                callback(path.to_string());
            }
        });
    }

    pub fn connect_trash_right_click<F: Fn() + 'static>(&self, callback: F) {
        let callback = std::rc::Rc::new(callback);
        // Walk through rows to find the trash row and add a right-click controller
        let mut child = self.imp().places_list.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == "place:trash:///" {
                let click = gtk::GestureClick::new();
                click.set_button(3);
                let cb = callback.clone();
                click.connect_pressed(move |gesture, _n, _x, _y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    cb();
                });
                widget.add_controller(click);
                break;
            }
            child = widget.next_sibling();
        }
    }

    fn populate_places(&self) {
        let list = &self.imp().places_list;

        for place in PLACES {
            if let Some(path) = (place.dir_fn)() {
                let icon = gtk::Image::from_icon_name(place.icon);
                icon.set_pixel_size(16);

                let label = gtk::Label::builder()
                    .label(place.name)
                    .xalign(0.0)
                    .hexpand(true)
                    .build();

                let row_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(8)
                    .margin_top(4)
                    .margin_bottom(4)
                    .margin_start(8)
                    .margin_end(8)
                    .build();
                row_box.append(&icon);
                row_box.append(&label);

                let row = gtk::ListBoxRow::new();
                row.set_child(Some(&row_box));
                row.set_widget_name(&format!("place:{}", path.to_string_lossy()));
                row.update_property(&[
                    gtk::accessible::Property::Label(place.name),
                ]);

                list.append(&row);
            }
        }

        // Root directory
        let icon = gtk::Image::from_icon_name("drive-harddisk-symbolic");
        icon.set_pixel_size(16);
        let label = gtk::Label::builder()
            .label("File System")
            .xalign(0.0)
            .hexpand(true)
            .build();
        let row_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();
        row_box.append(&icon);
        row_box.append(&label);
        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&row_box));
        row.set_widget_name("place:/");
        row.update_property(&[
            gtk::accessible::Property::Label("File System"),
        ]);
        list.append(&row);

        // Separator before Bin
        let separator_row = gtk::ListBoxRow::new();
        separator_row.set_child(Some(&gtk::Separator::new(gtk::Orientation::Horizontal)));
        separator_row.set_selectable(false);
        separator_row.set_activatable(false);
        list.append(&separator_row);

        // Bin
        let icon = gtk::Image::from_icon_name("user-trash-symbolic");
        icon.set_pixel_size(16);
        let label = gtk::Label::builder()
            .label("Bin")
            .xalign(0.0)
            .hexpand(true)
            .build();
        let row_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(8)
            .margin_end(8)
            .build();
        row_box.append(&icon);
        row_box.append(&label);
        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&row_box));
        row.set_widget_name("place:trash:///");
        row.update_property(&[
            gtk::accessible::Property::Label("Bin"),
        ]);
        list.append(&row);
    }
}

impl Default for WayfinderSidebar {
    fn default() -> Self {
        Self::new()
    }
}
