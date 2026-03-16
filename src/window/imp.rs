use std::cell::{Cell, RefCell};

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::AccessibleAnnouncementPriority;

use crate::clipboard::ClipboardState;
use crate::file_model::DirectoryModel;
use crate::file_object::FileObject;
use crate::navigation::NavigationState;
use crate::sidebar::WayfinderSidebar;
use crate::views::{WayfinderGridView, WayfinderListView};

const COLUMN_NAMES: &[&str] = &["Name", "Size", "Date Modified", "Kind"];

pub struct WayfinderWindowInner {
    pub model: DirectoryModel,
    pub list_view: WayfinderListView,
    pub grid_view: WayfinderGridView,
    pub selection: gtk::SingleSelection,
    pub nav: RefCell<NavigationState>,
    pub location_entry: gtk::Entry,
    pub status_label: gtk::Label,
    pub back_button: gtk::Button,
    pub forward_button: gtk::Button,
    pub up_button: gtk::Button,
    pub current_column: Cell<usize>,
    pub view_stack: gtk::Stack,
    pub sidebar: WayfinderSidebar,
    pub sidebar_revealer: gtk::Revealer,
    pub search_bar: gtk::SearchBar,
    pub search_entry: gtk::SearchEntry,
    pub clipboard: RefCell<Option<ClipboardState>>,
    pub current_view: Cell<ViewMode>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

impl Default for WayfinderWindowInner {
    fn default() -> Self {
        let model = DirectoryModel::new();
        let selection = gtk::SingleSelection::new(Some(model.filter_model.clone()));
        selection.set_autoselect(true);

        let list_view = WayfinderListView::new();
        list_view.set_model(&selection);

        // Grid view starts without a model — only the active view gets one
        let grid_view = WayfinderGridView::new();

        let initial_dir = crate::state::load_last_directory()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| "/".into());
        let nav = RefCell::new(NavigationState::new(initial_dir));

        let location_entry = gtk::Entry::builder()
            .hexpand(true)
            .placeholder_text("Path")
            .build();
        location_entry.update_property(&[
            gtk::accessible::Property::Label("Location"),
            gtk::accessible::Property::Description(
                "Current directory path. Press Enter to navigate.",
            ),
        ]);

        let status_label = gtk::Label::builder()
            .xalign(0.0)
            .margin_start(6)
            .margin_end(6)
            .margin_top(2)
            .margin_bottom(2)
            .build();
        status_label
            .update_property(&[gtk::accessible::Property::Label("Status bar")]);

        let back_button = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Go back (Alt+Left)")
            .sensitive(false)
            .build();
        back_button
            .update_property(&[gtk::accessible::Property::Label("Go back")]);

        let forward_button = gtk::Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text("Go forward (Alt+Right)")
            .sensitive(false)
            .build();
        forward_button
            .update_property(&[gtk::accessible::Property::Label("Go forward")]);

        let up_button = gtk::Button::builder()
            .icon_name("go-up-symbolic")
            .tooltip_text("Go to parent directory (Alt+Up)")
            .build();
        up_button.update_property(&[gtk::accessible::Property::Label(
            "Go to parent directory",
        )]);

        // View stack
        let view_stack = gtk::Stack::new();
        view_stack.set_transition_type(gtk::StackTransitionType::None);
        view_stack.add_named(grid_view.widget(), Some("grid"));
        view_stack.add_named(list_view.widget(), Some("list"));

        // Sidebar
        let sidebar = WayfinderSidebar::new();
        let sidebar_revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideRight)
            .reveal_child(true)
            .child(sidebar.widget())
            .build();

        // Search
        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Filter files..."));
        search_entry.update_property(&[
            gtk::accessible::Property::Label("Filter files"),
            gtk::accessible::Property::Description(
                "Type to filter files in the current directory",
            ),
        ]);

        let search_bar = gtk::SearchBar::builder()
            .show_close_button(true)
            .child(&search_entry)
            .build();
        search_bar.connect_entry(&search_entry);

        Self {
            model,
            list_view,
            grid_view,
            selection,
            nav,
            location_entry,
            status_label,
            back_button,
            forward_button,
            up_button,
            current_column: Cell::new(0),
            view_stack,
            sidebar,
            sidebar_revealer,
            search_bar,
            search_entry,
            clipboard: RefCell::new(None),
            current_view: Cell::new(ViewMode::List),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for WayfinderWindowInner {
    const NAME: &'static str = "WayfinderWindow";
    type Type = super::WayfinderWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for WayfinderWindowInner {
    fn constructed(&self) {
        self.parent_constructed();
        let window = self.obj();

        window.set_title(Some("Wayfinder"));
        let (w, h) = crate::state::load_window_size();
        window.set_default_size(w, h);

        // Header bar
        let header = gtk::HeaderBar::new();
        let nav_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        nav_box.add_css_class("linked");
        nav_box.append(&self.back_button);
        nav_box.append(&self.forward_button);
        header.pack_start(&nav_box);
        header.pack_start(&self.up_button);
        header.set_title_widget(Some(&self.location_entry));

        // Sidebar toggle button — restore saved state
        let sidebar_visible = crate::state::load_sidebar_visible();
        self.sidebar_revealer.set_reveal_child(sidebar_visible);
        let sidebar_toggle = gtk::ToggleButton::builder()
            .icon_name("sidebar-show-symbolic")
            .active(sidebar_visible)
            .tooltip_text("Toggle sidebar (Ctrl+Shift+S)")
            .build();
        sidebar_toggle
            .update_property(&[gtk::accessible::Property::Label("Toggle sidebar")]);
        let revealer = self.sidebar_revealer.clone();
        sidebar_toggle.connect_toggled(move |btn| {
            let visible = btn.is_active();
            revealer.set_reveal_child(visible);
            crate::state::save_sidebar_visible(visible);
        });
        header.pack_end(&sidebar_toggle);

        window.set_titlebar(Some(&header));

        // Content area with sidebar
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.append(&self.search_bar);
        content_box.append(&self.view_stack);
        content_box.append(&self.status_label);

        let paned = gtk::Paned::builder()
            .orientation(gtk::Orientation::Horizontal)
            .start_child(&self.sidebar_revealer)
            .end_child(&content_box)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .position(180)
            .build();

        window.set_child(Some(&paned));

        // Restore saved view mode
        let saved_view = crate::state::load_view_mode();
        let initial_mode = match saved_view.as_deref() {
            Some("grid") => {
                // Detach list, attach grid
                self.list_view.column_view().set_model(gtk::SelectionModel::NONE);
                self.grid_view.set_model(&self.selection);
                self.view_stack.set_visible_child_name("grid");
                ViewMode::Grid
            }
            _ => {
                self.view_stack.set_visible_child_name("list");
                ViewMode::List
            }
        };
        self.current_view.set(initial_mode);

        // Restore hidden files setting
        if crate::state::load_show_hidden() {
            self.model.toggle_hidden();
        }

        // Register actions
        self.setup_actions();

        // Register keyboard shortcuts
        crate::shortcuts::register_shortcuts(&window);

        // Connect list view activation (Enter / double-click)
        self.connect_activation();

        // Connect grid view activation
        self.connect_grid_activation();

        // Connect location entry
        self.connect_location_entry();

        // Connect location entry tab override
        self.connect_location_tab();

        // Connect nav buttons
        self.connect_nav_buttons();

        // Connect column navigation (Left/Right) and Tab escape on the list view
        self.connect_list_key_navigation();

        // Connect grid key navigation
        self.connect_grid_key_navigation();

        // Connect search
        self.connect_search();

        // Connect sidebar
        self.connect_sidebar();

        // Register context menu actions (Open, Open With, Properties)
        crate::context_menu::register_open_with_actions(&window);

        // Connect right-click and Shift+F10 for context menu
        self.connect_context_menu();

        // When the window regains focus (e.g. returning from an external app),
        // restore focus to the selected file item
        let w = window.clone();
        window.connect_is_active_notify(move |win| {
            if win.is_active() {
                let win2 = w.clone();
                glib::idle_add_local_once(move || {
                    win2.restore_focus_to_selected();
                });
            }
        });

        // Restore saved sort state and wire ColumnView's sorter to the model
        let (sort_col_idx, sort_asc) = crate::state::load_sort_state();
        let columns = self.list_view.column_view().columns();
        let sort_order = if sort_asc { gtk::SortType::Ascending } else { gtk::SortType::Descending };
        if let Some(col) = columns.item(sort_col_idx).and_downcast::<gtk::ColumnViewColumn>() {
            self.list_view.column_view().sort_by_column(Some(&col), sort_order);
        } else if let Some(first_col) = columns.item(0).and_downcast::<gtk::ColumnViewColumn>() {
            self.list_view.column_view().sort_by_column(Some(&first_col), gtk::SortType::Ascending);
        }
        if let Some(cv_sorter) = self.list_view.column_view().sorter() {
            self.model.set_sorter(Some(&cv_sorter));
        }

        // Load initial directory
        let initial_path = self.nav.borrow().current().to_string_lossy().to_string();
        window.navigate_to_path(&initial_path);
    }
}

impl WidgetImpl for WayfinderWindowInner {}
impl WindowImpl for WayfinderWindowInner {
    fn close_request(&self) -> glib::Propagation {
        let window = self.obj();
        let (w, h) = window.default_size();
        crate::state::save_window_size(w, h);
        self.parent_close_request()
    }
}
impl ApplicationWindowImpl for WayfinderWindowInner {}

impl WayfinderWindowInner {
    fn setup_actions(&self) {
        let window = self.obj();

        // Navigation actions
        let w = window.clone();
        let action = gio::SimpleAction::new("home", None);
        action.connect_activate(move |_, _| {
            if let Some(home) = dirs::home_dir() {
                w.navigate_to_path(&home.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("documents", None);
        action.connect_activate(move |_, _| {
            if let Some(dir) = dirs::document_dir() {
                w.navigate_to_path(&dir.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("desktop", None);
        action.connect_activate(move |_, _| {
            if let Some(dir) = dirs::desktop_dir() {
                w.navigate_to_path(&dir.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("downloads", None);
        action.connect_activate(move |_, _| {
            if let Some(dir) = dirs::download_dir() {
                w.navigate_to_path(&dir.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("root", None);
        action.connect_activate(move |_, _| {
            w.navigate_to_path("/");
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("up", None);
        action.connect_activate(move |_, _| {
            let parent = w.imp().nav.borrow().go_up();
            if let Some(parent) = parent {
                w.navigate_to_path(&parent.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("back", None);
        action.connect_activate(move |_, _| {
            let path = w.imp().nav.borrow_mut().go_back().cloned();
            if let Some(path) = path {
                w.load_directory(&path.to_string_lossy());
            }
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("forward", None);
        action.connect_activate(move |_, _| {
            let path = w.imp().nav.borrow_mut().go_forward().cloned();
            if let Some(path) = path {
                w.load_directory(&path.to_string_lossy());
            }
        });
        window.add_action(&action);

        let location_entry = self.location_entry.clone();
        let action = gio::SimpleAction::new("location-bar", None);
        action.connect_activate(move |_, _| {
            location_entry.grab_focus();
            location_entry.select_region(0, -1);
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("toggle-hidden", None);
        action.connect_activate(move |_, _| {
            let showing = w.imp().model.toggle_hidden();
            crate::state::save_show_hidden(showing);
            let msg = if showing {
                "Showing hidden files"
            } else {
                "Hidden files hidden"
            };
            w.announce(msg, AccessibleAnnouncementPriority::Medium);
            w.update_status();
        });
        window.add_action(&action);

        // View switching
        let w = window.clone();
        let action = gio::SimpleAction::new("view-grid", None);
        action.connect_activate(move |_, _| {
            w.switch_view(ViewMode::Grid);
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("view-list", None);
        action.connect_activate(move |_, _| {
            w.switch_view(ViewMode::List);
        });
        window.add_action(&action);

        // Sidebar toggle
        let revealer = self.sidebar_revealer.clone();
        let action = gio::SimpleAction::new("toggle-sidebar", None);
        action.connect_activate(move |_, _| {
            let visible = !revealer.reveals_child();
            revealer.set_reveal_child(visible);
            crate::state::save_sidebar_visible(visible);
        });
        window.add_action(&action);

        // Search
        let search_bar = self.search_bar.clone();
        let search_entry = self.search_entry.clone();
        let action = gio::SimpleAction::new("search", None);
        action.connect_activate(move |_, _| {
            search_bar.set_search_mode(true);
            search_entry.grab_focus();
        });
        window.add_action(&action);

        // File operations
        let w = window.clone();
        let action = gio::SimpleAction::new("copy", None);
        action.connect_activate(move |_, _| {
            w.copy_selected();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("cut", None);
        action.connect_activate(move |_, _| {
            w.cut_selected();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("paste", None);
        action.connect_activate(move |_, _| {
            w.paste();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("trash", None);
        action.connect_activate(move |_, _| {
            w.trash_selected();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("delete", None);
        action.connect_activate(move |_, _| {
            w.delete_selected();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("rename", None);
        action.connect_activate(move |_, _| {
            w.rename_selected();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("new-folder", None);
        action.connect_activate(move |_, _| {
            w.create_new_folder();
        });
        window.add_action(&action);

        let w = window.clone();
        let action = gio::SimpleAction::new("select-all", None);
        action.connect_activate(move |_, _| {
            // Select all not available with SingleSelection — will be added with MultiSelection
            w.announce(
                "Select all not yet available",
                AccessibleAnnouncementPriority::Medium,
            );
        });
        window.add_action(&action);

        // New window
        let w = window.clone();
        let action = gio::SimpleAction::new("new-window", None);
        action.connect_activate(move |_, _| {
            if let Some(app) = w.application() {
                let new_win = super::WayfinderWindow::new(&app);
                new_win.present();
            }
        });
        window.add_action(&action);

        // Go to folder dialog
        let w = window.clone();
        let action = gio::SimpleAction::new("go-to-folder", None);
        action.connect_activate(move |_, _| {
            w.show_go_to_folder();
        });
        window.add_action(&action);
    }

    fn connect_activation(&self) {
        let window = self.obj().clone();
        self.list_view
            .column_view()
            .connect_activate(move |_cv, pos| {
                let imp = window.imp();
                if let Some(item) = imp.selection.item(pos) {
                    let file = item.downcast_ref::<FileObject>().unwrap();
                    if file.is_directory() {
                        window.navigate_to_path(&file.path());
                    } else {
                        window.open_file(file);
                    }
                }
            });
    }

    fn connect_grid_activation(&self) {
        let window = self.obj().clone();
        self.grid_view
            .grid_view()
            .connect_activate(move |_gv, pos| {
                let imp = window.imp();
                if let Some(item) = imp.selection.item(pos) {
                    let file = item.downcast_ref::<FileObject>().unwrap();
                    if file.is_directory() {
                        window.navigate_to_path(&file.path());
                    } else {
                        window.open_file(file);
                    }
                }
            });
    }

    fn connect_location_entry(&self) {
        let window = self.obj().clone();
        self.location_entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            let path = if text.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    text.replacen('~', &home.to_string_lossy(), 1)
                } else {
                    text
                }
            } else {
                text
            };
            window.navigate_to_path(&path);
            window.focus_current_view();
        });
    }

    fn connect_location_tab(&self) {
        let controller = gtk::EventControllerKey::new();
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let sidebar = self.sidebar.clone();
        let sidebar_revealer = self.sidebar_revealer.clone();
        let window = self.obj().clone();

        controller.connect_key_pressed(move |_controller, key, _code, mods| {
            use gtk::gdk;

            let is_tab = key == gdk::Key::Tab && !mods.contains(gdk::ModifierType::SHIFT_MASK);
            let is_shift_tab = key == gdk::Key::ISO_Left_Tab
                || (key == gdk::Key::Tab && mods.contains(gdk::ModifierType::SHIFT_MASK));

            if is_tab || is_shift_tab {
                // Both Tab and Shift+Tab from location bar go to sidebar
                if sidebar_revealer.reveals_child() {
                    sidebar.widget().child_focus(gtk::DirectionType::TabForward);
                } else {
                    // Sidebar hidden — go to the file list
                    window.focus_current_view();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });

        self.location_entry.add_controller(controller);
    }

    fn connect_list_key_navigation(&self) {
        let controller = gtk::EventControllerKey::new();
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let window = self.obj().clone();

        controller.connect_key_pressed(move |_controller, key, _code, mods| {
            use gtk::gdk;
            let imp = window.imp();

            if key == gdk::Key::Tab && !mods.contains(gdk::ModifierType::SHIFT_MASK) {
                if imp.back_button.is_sensitive() {
                    imp.back_button.grab_focus();
                } else if imp.forward_button.is_sensitive() {
                    imp.forward_button.grab_focus();
                } else if imp.up_button.is_sensitive() {
                    imp.up_button.grab_focus();
                } else {
                    imp.location_entry.grab_focus();
                }
                glib::Propagation::Stop
            } else if key == gdk::Key::ISO_Left_Tab
                || (key == gdk::Key::Tab && mods.contains(gdk::ModifierType::SHIFT_MASK))
            {
                imp.location_entry.grab_focus();
                glib::Propagation::Stop
            } else if key == gdk::Key::Left
                && !mods.contains(gdk::ModifierType::ALT_MASK)
            {
                let col = imp.current_column.get();
                if col > 0 {
                    imp.current_column.set(col - 1);
                    imp.announce_current_cell(&window);
                }
                glib::Propagation::Stop
            } else if key == gdk::Key::Right
                && !mods.contains(gdk::ModifierType::ALT_MASK)
            {
                let col = imp.current_column.get();
                if col + 1 < COLUMN_NAMES.len() {
                    imp.current_column.set(col + 1);
                    imp.announce_current_cell(&window);
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });

        self.list_view.column_view().add_controller(controller);

        // Restore focus to selected row when list regains focus
        let focus_controller = gtk::EventControllerFocus::new();
        let selection = self.selection.clone();
        let list_view = self.list_view.clone();
        focus_controller.connect_enter(move |_| {
            let pos = selection.selected();
            if pos != gtk::INVALID_LIST_POSITION {
                list_view.grab_focus_at_selected(pos);
            }
        });
        self.list_view.widget().add_controller(focus_controller);

        // Announce column data on selection change in list view only.
        // Icon view is a native GtkListView — Orca reads it directly.
        let window2 = self.obj().clone();
        self.selection.connect_selected_item_notify(move |_sel| {
            if window2.imp().current_view.get() == ViewMode::List {
                window2.imp().announce_current_cell(&window2);
            }
        });
    }

    fn connect_grid_key_navigation(&self) {
        let controller = gtk::EventControllerKey::new();
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let window = self.obj().clone();

        controller.connect_key_pressed(move |_controller, key, _code, mods| {
            use gtk::gdk;
            let imp = window.imp();

            if key == gdk::Key::Tab && !mods.contains(gdk::ModifierType::SHIFT_MASK) {
                if imp.back_button.is_sensitive() {
                    imp.back_button.grab_focus();
                } else if imp.forward_button.is_sensitive() {
                    imp.forward_button.grab_focus();
                } else if imp.up_button.is_sensitive() {
                    imp.up_button.grab_focus();
                } else {
                    imp.location_entry.grab_focus();
                }
                glib::Propagation::Stop
            } else if key == gdk::Key::ISO_Left_Tab
                || (key == gdk::Key::Tab && mods.contains(gdk::ModifierType::SHIFT_MASK))
            {
                imp.location_entry.grab_focus();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });

        self.grid_view.grid_view().add_controller(controller);
    }

    fn connect_search(&self) {
        let window = self.obj().clone();
        let search_bar = self.search_bar.clone();

        self.search_entry.connect_search_changed(move |entry| {
            let text = entry.text().to_string();
            let imp = window.imp();
            imp.model.search.set_query(&text);
            window.update_status();

            if text.is_empty() {
                let count = imp.model.item_count();
                window.announce(
                    &format!("Filter cleared, {} items", count),
                    AccessibleAnnouncementPriority::Medium,
                );
            } else {
                let count = imp.model.item_count();
                window.announce(
                    &format!("{} items match '{}'", count, text),
                    AccessibleAnnouncementPriority::Medium,
                );
            }
        });

        // When search bar closes, clear the filter
        let window2 = self.obj().clone();
        search_bar.connect_search_mode_enabled_notify(move |bar| {
            if !bar.is_search_mode() {
                window2.imp().model.search.clear();
                window2.imp().search_entry.set_text("");
                window2.update_status();
                window2.focus_current_view();
            }
        });
    }

    fn connect_sidebar(&self) {
        let window = self.obj().clone();
        self.sidebar.connect_place_activated(move |path| {
            if path.starts_with("trash:") {
                window.load_special_uri(&path);
            } else {
                window.navigate_to_path(&path);
            }
        });

        // Right-click on Bin shows "Empty Bin" menu
        let window2 = self.obj().clone();
        self.sidebar.connect_trash_right_click(move || {
            let menu = gio::Menu::new();
            menu.append(Some("Open Bin"), Some("win.open-trash"));
            menu.append(Some("Empty Bin"), Some("win.empty-trash"));

            let popover = gtk::PopoverMenu::from_model(Some(&menu));
            popover.set_parent(window2.upcast_ref::<gtk::Widget>());
            popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(10, 10, 1, 1)));
            popover.set_has_arrow(false);

            let w = window2.clone();
            popover.connect_closed(move |pop| {
                pop.unparent();
                let win = w.clone();
                glib::idle_add_local_once(move || {
                    win.restore_focus_to_selected();
                });
            });

            popover.popup();
        });

        // Register open-trash action
        let w = self.obj().clone();
        let action = gio::SimpleAction::new("open-trash", None);
        action.connect_activate(move |_, _| {
            w.load_special_uri("trash:///");
        });
        self.obj().add_action(&action);
    }

    fn connect_context_menu(&self) {
        // Right-click on list view
        let click_controller = gtk::GestureClick::new();
        click_controller.set_button(3); // right button
        let window = self.obj().clone();
        click_controller.connect_pressed(move |gesture, _n, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            crate::context_menu::show_context_menu(&window, x, y);
        });
        self.list_view.column_view().add_controller(click_controller);

        // Right-click on grid view
        let click_controller2 = gtk::GestureClick::new();
        click_controller2.set_button(3);
        let window2 = self.obj().clone();
        click_controller2.connect_pressed(move |gesture, _n, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            crate::context_menu::show_context_menu(&window2, x, y);
        });
        self.grid_view.grid_view().add_controller(click_controller2);

        // Shift+F10 / Menu key — use a key controller on the window level
        let w = self.obj().clone();
        let action = gio::SimpleAction::new("context-menu", None);
        action.connect_activate(move |_, _| {
            crate::context_menu::show_context_menu(&w, 100.0, 100.0);
        });
        self.obj().add_action(&action);
    }

    fn announce_current_cell(&self, window: &super::WayfinderWindow) {
        let col = self.current_column.get();
        let col_name = COLUMN_NAMES[col];

        if let Some(item) = self.selection.selected_item() {
            let file = item.downcast_ref::<FileObject>().unwrap();
            let value = match col {
                0 => file.name(),
                1 => file.size_display(),
                2 => file.modified_display(),
                3 => file.file_type_name(),
                _ => String::new(),
            };
            let announcement = format!("{}: {}", col_name, value);
            window.announce(&announcement, AccessibleAnnouncementPriority::Medium);
        }
    }

    fn connect_nav_buttons(&self) {
        let window = self.obj().clone();
        self.back_button.connect_clicked(move |_| {
            let path = window.imp().nav.borrow_mut().go_back().cloned();
            if let Some(path) = path {
                window.load_directory(&path.to_string_lossy());
            }
        });

        let window = self.obj().clone();
        self.forward_button.connect_clicked(move |_| {
            let path = window.imp().nav.borrow_mut().go_forward().cloned();
            if let Some(path) = path {
                window.load_directory(&path.to_string_lossy());
            }
        });

        let window = self.obj().clone();
        self.up_button.connect_clicked(move |_| {
            let parent = window.imp().nav.borrow().go_up();
            if let Some(parent) = parent {
                window.navigate_to_path(&parent.to_string_lossy());
            }
        });
    }
}
