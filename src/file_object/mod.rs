mod imp;

use gtk::gio;
use gtk::glib;

glib::wrapper! {
    pub struct FileObject(ObjectSubclass<imp::FileObject>);
}

impl FileObject {
    pub fn from_file_info(parent_path: &str, info: &gio::FileInfo) -> Self {
        let name = info
            .name()
            .to_string_lossy()
            .to_string();

        let path = if parent_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        };

        let file_type = info.file_type();
        let is_directory = file_type == gio::FileType::Directory;
        let hidden = name.starts_with('.');

        let size = info.size() as u64;
        let size_display = if is_directory {
            String::from("--")
        } else {
            format_size(size)
        };

        let modified = info
            .modification_date_time()
            .map(|dt| dt.to_unix())
            .unwrap_or(0);
        let modified_display = info
            .modification_date_time()
            .map(|dt| {
                dt.format("%Y-%m-%d %H:%M")
                    .unwrap_or_else(|_| glib::GString::from(""))
                    .to_string()
            })
            .unwrap_or_default();

        let content_type = info
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let icon = if is_directory {
            String::from("folder")
        } else {
            gio::content_type_get_generic_icon_name(&content_type)
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("text-x-generic"))
        };

        let file_type_name = if is_directory {
            String::from("Folder")
        } else if !content_type.is_empty() {
            gio::content_type_get_description(&content_type).to_string()
        } else {
            String::from("File")
        };

        let search_string = name.to_lowercase();

        let a11y_name = if is_directory {
            format!("{}, Folder", name)
        } else {
            format!("{}, {}, {}", name, size_display, file_type_name)
        };

        glib::Object::builder()
            .property("name", &name)
            .property("path", &path)
            .property("icon", &icon)
            .property("size", size)
            .property("size-display", &size_display)
            .property("modified", modified)
            .property("modified-display", &modified_display)
            .property("mime-type", &content_type)
            .property("file-type-name", &file_type_name)
            .property("is-directory", is_directory)
            .property("hidden", hidden)
            .property("search-string", &search_string)
            .property("a11y-name", &a11y_name)
            .build()
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
