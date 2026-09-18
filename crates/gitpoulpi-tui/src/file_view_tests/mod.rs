mod clip;
mod content;
mod dates;
mod scroll;
mod selection;

use super::*;

use gitpoulpi_core::domain::{BlameLine, BlobSource};

use std::path::PathBuf;

fn oid(byte: u8) -> Oid {
    Oid([byte; 20])
}

fn request(kind: FileInsightKind) -> FileInsightRequest {
    FileInsightRequest {
        path: PathBuf::from("src/lib.rs"),
        at: oid(0xab),
        kind,
        worktree: false,
        staged: false,
    }
}

fn blame_insight(n: usize) -> FileInsight {
    let mut insight = FileInsight::loading(request(FileInsightKind::Blame));
    insight.data = FileInsightData::Blame(Blame {
        path: PathBuf::from("src/lib.rs"),
        at: oid(0xab),
        size: 0,
        content: BlameContent::Lines(
            (0..n)
                .map(|i| BlameLine {
                    commit: oid(i as u8 + 1),
                    author_name: "Ada Lovelace".into(),
                    author_email: "ada@example.com".into(),
                    timestamp: 1_700_000_000,
                    summary: "a commit".into(),
                    final_line: i as u32 + 1,
                    orig_line: i as u32 + 1,
                    committer_time: 0,
                    orig_path: Default::default(),
                    previous: None,
                    content: format!("line {i}"),
                })
                .collect(),
        ),
    });
    insight
}

// ---- Text selection ---------------------------

const BLAME_GUTTER: u16 = (SHA_WIDTH + 1 + AUTHOR_WIDTH + 1 + DATE_WIDTH + 1) as u16;
