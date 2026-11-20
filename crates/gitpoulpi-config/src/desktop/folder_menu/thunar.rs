//! Thunar keeps the user's custom actions in one shared file, `uca.xml`.
//! Ours is one `<action>` block, inserted right before the last `</actions>`
//! and cut out again byte for byte; no XML library is used on purpose (the
//! file is the user's, so nothing outside our block may be reformatted).

use std::path::Path;

use super::Who;

/// The primary's action id (the legacy one); the others append `-i<8hex>`.
/// Remove finds the block by the id.
pub const UNIQUE_ID: &str = "gitpoulpi-folder-menu";

/// `gitpoulpi-folder-menu[-i<8hex>]`: this installation's action id.
pub(super) fn unique_id(who: &Who) -> String {
    format!("{UNIQUE_ID}{}", who.suffix())
}

const HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n";
const TAIL: &str = "</actions>";

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The `<action>` block for `helper`, ending in a newline.
pub fn block(helper: &Path, label: &str, uid: &str, icon: &str) -> String {
    let command = format!(
        "{} launch --folder %f",
        crate::desktop::sh_quote(&helper.to_string_lossy())
    );
    let l = xml_escape(label);
    format!(
        "<action>\n\
         \t<icon>{icon}</icon>\n\
         \t<name>{l}</name>\n\
         \t<submenu></submenu>\n\
         \t<unique-id>{uid}</unique-id>\n\
         \t<command>{}</command>\n\
         \t<description>{l}</description>\n\
         \t<range></range>\n\
         \t<patterns>*</patterns>\n\
         \t<directories/>\n\
         </action>\n",
        xml_escape(&command)
    )
}

/// The byte range of our block, including the newline after it.
fn span(text: &str, uid: &str) -> Option<(usize, usize)> {
    let id = text.find(&format!("<unique-id>{uid}</unique-id>"))?;
    let start = text[..id].rfind("<action>")?;
    let end = id + text[id..].find("</action>")? + "</action>".len();
    let end = if text[end..].starts_with('\n') {
        end + 1
    } else {
        end
    };
    Some((start, end))
}

/// The file with our block added (or replaced). `None`: there was no file.
pub fn merge(existing: Option<&str>, block: &str, uid: &str) -> Result<String, String> {
    let Some(text) = existing else {
        return Ok(format!("{HEAD}{block}{TAIL}\n"));
    };
    if let Some((start, end)) = span(text, uid) {
        return Ok(format!("{}{block}{}", &text[..start], &text[end..]));
    }
    match text.rfind(TAIL) {
        Some(at) => Ok(format!("{}{block}{}", &text[..at], &text[at..])),
        None => Err("uca.xml is not in the expected format; not touching it".to_string()),
    }
}

/// The file without our block, and whether it was there.
pub fn unmerge(text: &str, uid: &str) -> (String, bool) {
    match span(text, uid) {
        Some((start, end)) => (format!("{}{}", &text[..start], &text[end..]), true),
        None => (text.to_string(), false),
    }
}

/// Is `text` what [`merge`] creates for a missing file, minus our block?
pub fn is_skeleton(text: &str) -> bool {
    text == format!("{HEAD}{TAIL}\n")
}

/// The label our block carries (unescaped), if it is there.
pub fn label_of(text: &str, uid: &str) -> Option<String> {
    let (start, end) = span(text, uid)?;
    let b = &text[start..end];
    let from = b.find("<name>")? + "<name>".len();
    let to = from + b[from..].find("</name>")?;
    Some(xml_unescape(&b[from..to]))
}

/// Our block as it stands in `text`.
pub fn block_of<'a>(text: &'a str, uid: &str) -> Option<&'a str> {
    span(text, uid).map(|(s, e)| &text[s..e])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const USER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n<action>\n\t<icon>utilities-terminal</icon>\n\t<name>Open Terminal Here</name>\n\t<unique-id>1</unique-id>\n\t<command>exo-open --working-directory %f --launch TerminalEmulator</command>\n\t<patterns>*</patterns>\n\t<directories/>\n</action>\n<action>\n\t<name>Two</name>\n\t<unique-id>2</unique-id>\n\t<command>true</command>\n\t<patterns>*</patterns>\n</action>\n</actions>\n";

    fn blk() -> String {
        block(
            &PathBuf::from("/opt/gp/gitpoulpi"),
            "Open in GitPoulpi",
            UNIQUE_ID,
            "gitpoulpi-folder",
        )
    }

    #[test]
    fn merge_then_unmerge_restores_a_user_file_byte_for_byte() {
        let merged = merge(Some(USER), &blk(), UNIQUE_ID).unwrap();
        assert!(merged.contains(UNIQUE_ID));
        assert!(merged.ends_with(&format!("{}</actions>\n", blk())));
        assert_eq!(unmerge(&merged, UNIQUE_ID), (USER.to_string(), true));
    }

    #[test]
    fn a_file_without_a_trailing_newline_or_indentation_is_still_restored_exactly() {
        for user in [
            "<actions><action><unique-id>9</unique-id></action></actions>",
            "<actions>\n</actions>",
            "<actions>\r\n<action><unique-id>9</unique-id></action>\r\n</actions>\r\n",
        ] {
            let merged = merge(Some(user), &blk(), UNIQUE_ID).unwrap();
            assert_eq!(unmerge(&merged, UNIQUE_ID).0, user);
        }
    }

    #[test]
    fn merging_twice_replaces_our_block_instead_of_adding_another() {
        let once = merge(Some(USER), &blk(), UNIQUE_ID).unwrap();
        let other = block(
            &PathBuf::from("/elsewhere/gitpoulpi"),
            "Ouvrir",
            UNIQUE_ID,
            "gitpoulpi-folder",
        );
        let twice = merge(Some(&once), &other, UNIQUE_ID).unwrap();
        assert_eq!(twice.matches(UNIQUE_ID).count(), 1);
        assert_eq!(label_of(&twice, UNIQUE_ID).as_deref(), Some("Ouvrir"));
        assert_eq!(unmerge(&twice, UNIQUE_ID).0, USER);
    }

    #[test]
    fn a_missing_file_is_created_with_the_skeleton_and_reads_as_one_after_removal() {
        let merged = merge(None, &blk(), UNIQUE_ID).unwrap();
        assert!(merged.starts_with("<?xml"));
        let (rest, found) = unmerge(&merged, UNIQUE_ID);
        assert!(found && is_skeleton(&rest));
        assert!(!is_skeleton(USER));
    }

    #[test]
    fn a_file_without_the_closing_tag_is_refused() {
        assert!(merge(Some("<actions>\n<action>"), &blk(), UNIQUE_ID).is_err());
        assert!(merge(Some("not xml"), &blk(), UNIQUE_ID).is_err());
    }

    #[test]
    fn the_label_and_the_command_are_escaped() {
        let b = block(
            &PathBuf::from("/o'p/gitpoulpi"),
            "A & <B>",
            UNIQUE_ID,
            "gitpoulpi-folder",
        );
        assert!(b.contains("<name>A &amp; &lt;B&gt;</name>"));
        assert!(b.contains("<description>A &amp; &lt;B&gt;</description>"));
        let cmd = xml_escape("'/o'\\''p/gitpoulpi' launch --folder %f");
        assert!(b.contains(&format!("<command>{cmd}</command>")), "{b}");
        assert_eq!(
            label_of(&merge(None, &b, UNIQUE_ID).unwrap(), UNIQUE_ID).as_deref(),
            Some("A & <B>")
        );
    }
}
