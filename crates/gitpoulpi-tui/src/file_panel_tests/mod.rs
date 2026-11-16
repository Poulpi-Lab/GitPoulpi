mod open_view;
mod render;
mod scroll;
mod wip_list;

use super::*;

use crate::test_support::*;

use gitpoulpi_core::domain::FileDiff;

fn outcome(changed: &[&str]) -> CommitFilesOutcome {
    CommitFilesOutcome {
        commit: oid(7),
        files: Ok(changed
            .iter()
            .map(|p| FileDiff {
                renamed_from: None,
                similarity: None,
                path: PathBuf::from(p),
                status: FileStatus::Modified,
                hunks: Vec::new(),
            })
            .collect()),
        tree: None,
        hunks_pending: false,
    }
}

fn paths(ps: &[&str]) -> Vec<PathBuf> {
    ps.iter().map(PathBuf::from).collect()
}

fn buffer_text(buf: &ratatui::buffer::Buffer) -> Vec<String> {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect()
}
