//! The Windows "Open with" list, from the registry.
//!
//! For an extension `.pdf`:
//!
//! - the default is the `ProgId` of
//!   `HKCU\...\Explorer\FileExts\.pdf\UserChoice`, else the default value of
//!   `HKCR\.pdf`;
//! - the others are the value names of `HKCR\.pdf\OpenWithProgids` and of
//!   `...\FileExts\.pdf\OpenWithProgids`, the programs of
//!   `...\FileExts\.pdf\OpenWithList` (values `a`, `b`, ... except `MRUList`)
//!   and the sub keys of `HKCR\.pdf\OpenWithList`; a program `x.exe` is the
//!   ProgId `Applications\x.exe`;
//! - a ProgId is run through `HKCR\<ProgId>\shell\open\command`. One without a
//!   command (a packaged app that registers a `DelegateExecute`) can not be
//!   launched from here; when it is the default the list still offers
//!   Windows' own `start`, which does open it.
//!
//! The registry is a parameter ([`Registry`]); the real reader (`reg.exe`)
//! lives in the TUI, behind `cfg(windows)`, and its output parser
//! ([`parse_reg_query`]) is here so Linux tests cover it.

use super::{exe_key, quote_arg, Found, OsApp};

/// What the parser reads of the registry. Key paths are
/// `HKCR\.pdf\OpenWithProgids` style; names compare without regard to case.
pub trait Registry {
    /// The names of the sub keys of `key`.
    fn subkeys(&self, key: &str) -> Vec<String>;
    /// The values of `key` as `(name, data)`; the default value's name is "".
    fn values(&self, key: &str) -> Vec<(String, String)>;
}

const FILE_EXTS: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts";

fn value(reg: &dyn Registry, key: &str, name: &str) -> Option<String> {
    reg.values(key)
        .into_iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
}

/// `%NAME%` replaced from `env` (an unknown variable stays as written).
pub fn expand_percent_vars(text: &str, env: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end)
                if end > 0
                    && after[..end]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '(' || c == ')')
                    && !after[..end].chars().all(|c| c.is_ascii_digit()) =>
            {
                match env(&after[..end]) {
                    Some(v) => out.push_str(&v),
                    None => {
                        out.push('%');
                        out.push_str(&after[..end]);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            _ => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// A `shell\open\command` as an argv template: `%1`, `%L`, `%l` and `%*`
/// become `${file}`, the other positional codes go, `%VAR%` is expanded, and a
/// command with no file code gets the file as its last (quoted) argument.
pub fn command_template(command: &str, env: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let command = expand_percent_vars(command.trim(), env);
    let mut out = String::new();
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('1' | 'L' | 'l' | '*') => {
                chars.next();
                out.push_str("${file}");
            }
            Some(d) if d.is_ascii_digit() => {
                chars.next();
            }
            Some('%') => {
                chars.next();
                out.push('%');
            }
            _ => out.push('%'),
        }
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        return None;
    }
    let out = if out.contains("${file}") {
        out
    } else {
        format!("{out} \"${{file}}\"")
    };
    // The command must split, and name no variable but the file.
    let argv = crate::launch::split_argv(&out).ok()?;
    if argv.is_empty() || argv[0].contains("${") {
        return None;
    }
    if argv.iter().any(|a| a.replace("${file}", "").contains("${")) {
        return None;
    }
    Some(out)
}

/// `Viewer.exe` -> `Viewer`, `acrord32` -> `Acrord32`.
fn exe_title(exe: &str) -> String {
    let base = exe.rsplit(['/', '\\']).next().unwrap_or(exe);
    let stem = base
        .strip_suffix(".exe")
        .or_else(|| base.strip_suffix(".EXE"))
        .unwrap_or(base);
    let mut chars = stem.chars();
    match chars.next() {
        Some(first) if first.is_lowercase() => first.to_uppercase().chain(chars).collect(),
        Some(first) => first.to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// The command key a ProgId runs: `shell\open`, else the default verb.
fn command_of(reg: &dyn Registry, prog_id: &str) -> Option<String> {
    let shell = format!(r"HKCR\{prog_id}\shell");
    if let Some(c) = value(reg, &format!(r"{shell}\open\command"), "") {
        return Some(c);
    }
    let verb = value(reg, &shell, "")?;
    value(reg, &format!(r"{shell}\{verb}\command"), "")
}

/// A friendly name the registry gives the application, if it is a plain string
/// (`@shell32.dll,-1234` references need the resource loader).
fn friendly_name(reg: &dyn Registry, prog_id: &str) -> Option<String> {
    let plain = |v: String| (!v.starts_with('@')).then_some(v);
    let open = format!(r"HKCR\{prog_id}\shell\open");
    value(reg, &format!(r"HKCR\{prog_id}"), "FriendlyAppName")
        .and_then(plain)
        .or_else(|| value(reg, &open, "FriendlyAppName").and_then(plain))
        .or_else(|| value(reg, &open, "MUIVerb").and_then(plain))
}

/// The applications for the extension `ext` (with or without the dot).
pub fn apps_for(ext: &str, reg: &dyn Registry, env: &dyn Fn(&str) -> Option<String>) -> Found {
    let ext = ext.trim_start_matches('.').to_lowercase();
    if ext.is_empty() {
        return Found::default();
    }
    let dot = format!(".{ext}");
    let class = format!(r"HKCR\{dot}");
    let exts = format!(r"{FILE_EXTS}\{dot}");

    let textual = value(reg, &class, "PerceivedType")
        .is_some_and(|t| t.eq_ignore_ascii_case("text"))
        || value(reg, &class, "Content Type").is_some_and(|t| t.starts_with("text/"));

    let chosen =
        value(reg, &format!(r"{exts}\UserChoice"), "ProgId").or_else(|| value(reg, &class, ""));

    let mut ids: Vec<String> = Vec::new();
    ids.extend(chosen.clone());
    for key in [
        format!(r"{class}\OpenWithProgids"),
        format!(r"{exts}\OpenWithProgids"),
    ] {
        ids.extend(reg.values(&key).into_iter().map(|(name, _)| name));
    }
    for (name, data) in reg.values(&format!(r"{exts}\OpenWithList")) {
        if !name.eq_ignore_ascii_case("MRUList") && !data.is_empty() {
            ids.push(format!(r"Applications\{data}"));
        }
    }
    for exe in reg.subkeys(&format!(r"{class}\OpenWithList")) {
        ids.push(format!(r"Applications\{exe}"));
    }

    let mut seen: Vec<String> = Vec::new();
    let mut apps = Vec::new();
    let mut default_resolved = false;
    for id in ids {
        let lower = id.to_lowercase();
        if id.is_empty() || seen.contains(&lower) {
            continue;
        }
        seen.push(lower);
        let is_default = chosen.as_ref().is_some_and(|c| c.eq_ignore_ascii_case(&id));
        let Some(command) = command_of(reg, &id) else {
            continue;
        };
        let Some(template) = command_template(&command, env) else {
            continue;
        };
        let argv = crate::launch::split_argv(&template).unwrap_or_default();
        let exe = argv.first().cloned().unwrap_or_default();
        let name = friendly_name(reg, &id)
            .or_else(|| {
                id.strip_prefix(r"Applications\")
                    .and_then(|app| friendly_name(reg, &format!(r"Applications\{app}")))
            })
            .unwrap_or_else(|| exe_title(&exe));
        default_resolved |= is_default;
        apps.push(OsApp {
            name,
            key: exe_key(&argv),
            command: template,
            default: is_default,
        });
    }
    // The user's choice exists but can not be run from here (a packaged app):
    // Windows' own `start` opens it.
    if chosen.is_some() && !default_resolved {
        apps.push(OsApp {
            name: "Default app".into(), // i18n-english
            command: format!("cmd /C start \"\" {}", quote_arg("${file}")),
            key: "cmd".into(),
            default: true,
        });
    }
    Found { apps, textual }.finish()
}

/// Parses the output of `reg query <key>`: the sub keys (full paths, one per
/// line, starting with `HKEY_`) and the values (`    Name    REG_SZ    Data`;
/// the default value is `(Default)` or `<NO NAME>`). Only string values
/// (`REG_SZ`, `REG_EXPAND_SZ`) carry data; others have an empty one.
pub fn parse_reg_query(output: &str) -> (Vec<String>, Vec<(String, String)>) {
    let mut subkeys = Vec::new();
    let mut values = Vec::new();
    // The first non-empty line is the key itself.
    let mut first = true;
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            if first {
                first = false;
            } else if let Some((_, tail)) = line.trim_end().rsplit_once('\\') {
                subkeys.push(tail.to_string());
            }
            continue;
        }
        first = false;
        let body = line.trim_start();
        // Name and type are separated by four spaces; the data by four more.
        let Some((name, rest)) = split_value_line(body) else {
            continue;
        };
        let (kind, data) = rest.split_once("    ").unwrap_or((rest, ""));
        let data = if kind == "REG_SZ" || kind == "REG_EXPAND_SZ" {
            data.trim().to_string()
        } else {
            String::new()
        };
        // `(Default)` is worded by the system's language.
        let name = if name == "<NO NAME>" || (name.starts_with('(') && name.ends_with(')')) {
            String::new()
        } else {
            name.to_string()
        };
        values.push((name, data));
    }
    (subkeys, values)
}

/// `Name    REG_SZ    Data` -> (`Name`, `REG_SZ    Data`): the name ends at
/// the first `REG_` type word that follows four spaces.
fn split_value_line(body: &str) -> Option<(&str, &str)> {
    let at = body.find("    REG_")?;
    Some((body[..at].trim_end(), body[at + 4..].trim_start()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// A fake registry: keys with their sub keys and values; lookups ignore case.
    #[derive(Default)]
    struct Fake {
        values: BTreeMap<String, Vec<(String, String)>>,
        keys: BTreeMap<String, Vec<String>>,
    }

    impl Fake {
        fn value(mut self, key: &str, name: &str, data: &str) -> Self {
            self.values
                .entry(key.to_lowercase())
                .or_default()
                .push((name.to_string(), data.to_string()));
            self
        }

        fn sub(mut self, key: &str, name: &str) -> Self {
            self.keys
                .entry(key.to_lowercase())
                .or_default()
                .push(name.to_string());
            self
        }
    }

    impl Registry for Fake {
        fn subkeys(&self, key: &str) -> Vec<String> {
            self.keys
                .get(&key.to_lowercase())
                .cloned()
                .unwrap_or_default()
        }

        fn values(&self, key: &str) -> Vec<(String, String)> {
            self.values
                .get(&key.to_lowercase())
                .cloned()
                .unwrap_or_default()
        }
    }

    fn env(k: &str) -> Option<String> {
        match k {
            "ProgramFiles" => Some(r"C:\Program Files".into()),
            "SystemRoot" => Some(r"C:\Windows".into()),
            _ => None,
        }
    }

    fn pdf_machine() -> Fake {
        Fake::default()
            .value(r"HKCR\.pdf", "", "AcroExch.Document.DC")
            .value(r"HKCR\.pdf\OpenWithProgids", "AcroExch.Document.DC", "")
            .value(r"HKCR\.pdf\OpenWithProgids", "MSEdgePDF", "")
            .value(
                &format!(r"{FILE_EXTS}\.pdf\UserChoice"),
                "ProgId",
                "MSEdgePDF",
            )
            .value(
                &format!(r"{FILE_EXTS}\.pdf\OpenWithList"),
                "a",
                "SumatraPDF.exe",
            )
            .value(&format!(r"{FILE_EXTS}\.pdf\OpenWithList"), "MRUList", "a")
            .sub(r"HKCR\.pdf\OpenWithList", "Foxit.exe")
            .value(
                r"HKCR\AcroExch.Document.DC\shell\open\command",
                "",
                r#""C:\Program Files\Adobe\Acrobat\Acrobat.exe" "%1""#,
            )
            .value(
                r"HKCR\MSEdgePDF\shell\open\command",
                "",
                r#""%ProgramFiles%\Edge\msedge.exe" --single-argument %1"#,
            )
            .value(r"HKCR\MSEdgePDF", "FriendlyAppName", "Microsoft Edge")
            .value(
                r"HKCR\Applications\SumatraPDF.exe\shell\open\command",
                "",
                r#""C:\Tools\SumatraPDF.exe" "%1""#,
            )
            .value(
                r"HKCR\Applications\SumatraPDF.exe",
                "FriendlyAppName",
                "Sumatra PDF",
            )
            .value(
                r"HKCR\Applications\Foxit.exe\shell\open\command",
                "",
                r#"C:\Foxit\Foxit.exe /A %1"#,
            )
    }

    #[test]
    fn the_user_choice_is_the_default_and_every_registered_program_is_listed() {
        let found = apps_for(".PDF", &pdf_machine(), &env);
        let names: Vec<_> = found.apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Microsoft Edge", "Acrobat", "Foxit", "Sumatra PDF"]);
        assert!(found.apps[0].default);
        assert!(found.apps[1..].iter().all(|a| !a.default));
        assert_eq!(
            found.apps[0].command,
            r#""C:\Program Files\Edge\msedge.exe" --single-argument ${file}"#
        );
        assert_eq!(found.apps[2].command, r"C:\Foxit\Foxit.exe /A ${file}");
        assert!(!found.textual);
    }

    #[test]
    fn without_a_user_choice_the_class_default_is_the_default() {
        let reg = Fake::default().value(r"HKCR\.png", "", "pngfile").value(
            r"HKCR\pngfile\shell\open\command",
            "",
            r#"mspaint.exe "%1""#,
        );
        let found = apps_for("png", &reg, &env);
        assert_eq!(found.apps.len(), 1);
        assert_eq!(found.apps[0].name, "Mspaint");
        assert!(found.apps[0].default);
    }

    #[test]
    fn a_packaged_default_without_a_command_falls_back_to_start() {
        let reg = Fake::default()
            .value(
                &format!(r"{FILE_EXTS}\.jpg\UserChoice"),
                "ProgId",
                "AppX1234",
            )
            .value(r"HKCR\.jpg\OpenWithProgids", "jpegfile", "")
            .value(
                r"HKCR\jpegfile\shell\open\command",
                "",
                r#""C:\v\view.exe" %1"#,
            );
        let found = apps_for("jpg", &reg, &env);
        assert_eq!(found.apps[0].name, "Default app");
        assert!(found.apps[0].default);
        assert_eq!(found.apps[0].command, "cmd /C start \"\" ${file}");
        assert_eq!(found.apps[1].name, "View");
    }

    #[test]
    fn an_extension_the_registry_does_not_know_has_no_apps() {
        assert!(apps_for("zzz", &pdf_machine(), &env).apps.is_empty());
        assert!(apps_for("", &pdf_machine(), &env).apps.is_empty());
    }

    #[test]
    fn text_types_are_flagged() {
        let reg = Fake::default()
            .value(r"HKCR\.txt", "", "txtfile")
            .value(r"HKCR\.txt", "PerceivedType", "text")
            .value(r"HKCR\txtfile\shell\open\command", "", r"notepad.exe %1");
        assert!(apps_for("txt", &reg, &env).textual);
    }

    #[test]
    fn commands_become_templates() {
        let t = |c: &str| command_template(c, &env);
        assert_eq!(
            t(r#""C:\a b\x.exe" "%1""#).as_deref(),
            Some(r#""C:\a b\x.exe" "${file}""#)
        );
        assert_eq!(t(r"x.exe /open %L").as_deref(), Some("x.exe /open ${file}"));
        assert_eq!(t(r"x.exe").as_deref(), Some(r#"x.exe "${file}""#));
        assert_eq!(t(r"x.exe %1 %2").as_deref(), Some("x.exe ${file}"));
        assert_eq!(
            t(r#"%SystemRoot%\system32\rundll32.exe a.dll,Open %1"#).as_deref(),
            Some(r"C:\Windows\system32\rundll32.exe a.dll,Open ${file}")
        );
        assert_eq!(
            t(r#"%Unknown%\x.exe %1"#).as_deref(),
            Some(r"%Unknown%\x.exe ${file}")
        );
        assert_eq!(t(""), None);
        assert_eq!(t(r#""unterminated %1"#), None);
    }

    #[test]
    fn percent_variables_are_expanded_only_when_they_name_a_variable() {
        assert_eq!(expand_percent_vars("100% sure", &env), "100% sure");
        assert_eq!(
            expand_percent_vars("%1 and %SystemRoot%", &env),
            r"%1 and C:\Windows"
        );
    }

    #[test]
    fn reg_query_output_is_parsed_into_keys_and_values() {
        let out = "\r\nHKEY_CLASSES_ROOT\\.pdf\r\n    (Default)    REG_SZ    AcroExch.Document.DC\r\n    Content Type    REG_SZ    application/pdf\r\n    Flags    REG_DWORD    0x1\r\n\r\nHKEY_CLASSES_ROOT\\.pdf\\OpenWithProgids\r\nHKEY_CLASSES_ROOT\\.pdf\\OpenWithList\r\n";
        let (keys, values) = parse_reg_query(out);
        assert_eq!(keys, ["OpenWithProgids", "OpenWithList"]);
        assert_eq!(
            values,
            [
                ("".to_string(), "AcroExch.Document.DC".to_string()),
                ("Content Type".to_string(), "application/pdf".to_string()),
                ("Flags".to_string(), String::new()),
            ]
        );
        // A command with spaces and quotes keeps them.
        let out = "HKEY_CLASSES_ROOT\\x\\shell\\open\\command\r\n    (Default)    REG_SZ    \"C:\\Program Files\\a b\\x.exe\" \"%1\"\r\n";
        let (_, values) = parse_reg_query(out);
        assert_eq!(values[0].1, r#""C:\Program Files\a b\x.exe" "%1""#);
        // A value-less ProgId list.
        let out = "HKEY_CLASSES_ROOT\\.pdf\\OpenWithProgids\r\n    AcroExch.Document.DC    REG_NONE    \r\n";
        let (_, values) = parse_reg_query(out);
        assert_eq!(
            values,
            [("AcroExch.Document.DC".to_string(), String::new())]
        );
    }
}
