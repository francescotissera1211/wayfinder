mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{ListScrollFlags, SelectionModel};

glib::wrapper! {
    pub struct WayfinderGridView(ObjectSubclass<imp::GridViewInner>);
}

impl WayfinderGridView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_model(&self, model: &impl IsA<SelectionModel>) {
        self.imp().grid_view.set_model(Some(model));
    }

    pub fn widget(&self) -> &gtk::ScrolledWindow {
        &self.imp().scrolled_window
    }

    pub fn grid_view(&self) -> &gtk::GridView {
        &self.imp().grid_view
    }

    pub fn grab_focus_at_selected(&self, selected_pos: u32) {
        self.imp().grid_view.scroll_to(
            selected_pos,
            ListScrollFlags::FOCUS,
            None,
        );
    }

    pub fn grab_focus(&self) {
        self.imp().grid_view.grab_focus();
    }
}

impl Default for WayfinderGridView {
    fn default() -> Self {
        Self::new()
    }
}
