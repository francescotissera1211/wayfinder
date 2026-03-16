mod app;
mod clipboard;
mod context_menu;
mod file_model;
mod file_object;
mod file_ops;
mod navigation;
mod properties;
mod search;
mod shortcuts;
mod sidebar;
mod state;
mod views;
mod window;

use gtk::prelude::*;

use app::WayfinderApplication;

fn main() {
    env_logger::init();

    let app = WayfinderApplication::new();
    app.run();
}
