use super::*;
use crate::folder_listing::{read_folder, FOLDER_LIMIT};

/// root/ { alpha/ (a repository), beta/ { inner/ }, notes.txt }
fn tree() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("alpha").join(".git")).unwrap();
    std::fs::create_dir_all(tmp.path().join("beta").join("inner")).unwrap();
    std::fs::write(tmp.path().join("notes.txt"), "x").unwrap();
    tmp
}

/// Serves every pending read synchronously - what the lister does on
/// its threads.
fn settle(picker: &mut FolderPicker) {
    for _ in 0..8 {
        let wants = picker.wants();
        if wants.is_empty() {
            return;
        }
        for dir in wants {
            let listing = read_folder(&dir, FOLDER_LIMIT);
            picker.deliver(&dir, listing);
        }
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn picker_at(dir: &Path) -> FolderPicker {
    let mut picker = FolderPicker::new("Open a repository", "Open", dir.to_path_buf());
    settle(&mut picker);
    picker
}

fn draw(picker: &mut FolderPicker, width: u16, height: u16) -> String {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    picker.render(picker_rect(buf.area), &mut buf);
    dialog_kit::buffer_text(&buf)
}

fn click(picker: &mut FolderPicker, hit: PickerHit, double: bool) -> PickerOutcome {
    let (rect, _) = *picker
        .hits()
        .iter()
        .find(|(_, h)| *h == hit)
        .unwrap_or_else(|| panic!("{hit:?} not drawn"));
    picker.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x + 1,
            row: rect.y,
            modifiers: KeyModifiers::NONE,
        },
        double,
    )
}

fn content_index(picker: &FolderPicker, name: &str) -> usize {
    let listing = picker.listing().unwrap();
    picker
        .content_rows()
        .iter()
        .position(|r| matches!(r, ContentRow::Dir(i) if listing.dirs[*i].name == name))
        .unwrap()
}

mod bars;
mod choose;
mod filter;
mod keys;
mod listing;
mod mouse;
mod new_folder;
mod render;
mod siblings;
mod splitter;
