use gtk::gio;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOperation {
    Copy,
    Cut,
}

#[derive(Clone)]
pub struct ClipboardState {
    pub operation: ClipboardOperation,
    pub files: Vec<gio::File>,
}

impl ClipboardState {
    pub fn new(operation: ClipboardOperation, files: Vec<gio::File>) -> Self {
        Self { operation, files }
    }
}
