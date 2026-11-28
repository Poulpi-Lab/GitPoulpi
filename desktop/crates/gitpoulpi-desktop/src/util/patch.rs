//! `FileDiff` -> unified patch, which is what the kit's `DiffFile::parse` consumes (it then does the
//! word-level inline diff and the syntax highlighting).

use gitpoulpi_core::domain::{FileDiff, FileStatus, LineOrigin};

pub fn to_patch(files: &[FileDiff]) -> String {
    let mut out = String::new();
    for f in files {
        let path = f.path.to_string_lossy();
        let old = f
            .renamed_from
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
        out.push_str(&format!("diff --git a/{old} b/{path}\n"));
        match f.status {
            FileStatus::Added | FileStatus::Untracked => {
                out.push_str("new file mode 100644\n--- /dev/null\n");
                out.push_str(&format!("+++ b/{path}\n"));
            }
            FileStatus::Deleted => {
                out.push_str("deleted file mode 100644\n");
                out.push_str(&format!("--- a/{old}\n+++ /dev/null\n"));
            }
            FileStatus::Renamed => {
                out.push_str(&format!("rename from {old}\nrename to {path}\n"));
                if !f.hunks.is_empty() {
                    out.push_str(&format!("--- a/{old}\n+++ b/{path}\n"));
                }
            }
            _ => out.push_str(&format!("--- a/{old}\n+++ b/{path}\n")),
        }
        for h in &f.hunks {
            out.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                h.old_start, h.old_lines, h.new_start, h.new_lines
            ));
            for l in &h.lines {
                let prefix = match l.origin {
                    LineOrigin::Add => '+',
                    LineOrigin::Delete => '-',
                    LineOrigin::Context => ' ',
                };
                out.push(prefix);
                out.push_str(l.content.trim_end_matches(['\n', '\r']));
                out.push('\n');
            }
        }
    }
    out
}
