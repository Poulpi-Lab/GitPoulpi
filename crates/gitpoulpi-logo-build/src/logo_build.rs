//! Build-time logo selection, shared by the build scripts of the three
//! binary crates and by `xtask build-custom`, so the choices the scripts
//! validate and the ones the build applies cannot disagree.
//!
//! * `GITPOULPI_LOGO_COLORS`: unset or `all` = every colorway of
//!   `colourways.json`; otherwise a comma list of slugs (kept in
//!   the table's order, which is the spin order of the About window);
//! * `GITPOULPI_LOGO_DEFAULT`: the About default and the `ui.logo_color`
//!   fallback; must be one of the compiled colorways (unset = the table's
//!   default if compiled, else the first compiled);
//! * `GITPOULPI_ICON_COLOR`: colorway of the Windows executable icon (any
//!   colorway of the table, compiled or not; unset = the default above).

use serde_json::Value;

/// The colorway table: every colorway with its 16x16 sprite.
pub const TABLE_JSON: &str = include_str!("../colourways.json");

pub const COLORS_ENV: &str = "GITPOULPI_LOGO_COLORS";
pub const DEFAULT_ENV: &str = "GITPOULPI_LOGO_DEFAULT";
pub const ICON_ENV: &str = "GITPOULPI_ICON_COLOR";

/// Every slug of the table, in order.
pub fn slugs(table: &Value) -> Vec<String> {
    table["ways"]
        .as_array()
        .map(|w| {
            w.iter()
                .filter_map(|x| x["slug"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// `""` / `all` -> `None` (every colorway); otherwise the validated slugs in
/// table order. An empty selection is an error: the About window needs one.
pub fn parse_colors(value: &str, all: &[String]) -> Result<Option<Vec<String>>, String> {
    let mut want: Vec<String> = Vec::new();
    for raw in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let s = raw.to_ascii_lowercase();
        if s == "all" {
            return Ok(None);
        }
        if !all.contains(&s) {
            return Err(format!(
                "unknown logo color '{raw}'. Available: all, {}",
                all.join(", ")
            ));
        }
        want.push(s);
    }
    if want.is_empty() {
        return if value.trim().is_empty() {
            Ok(None)
        } else {
            Err("at least one logo color is required".into())
        };
    }
    Ok(Some(
        all.iter().filter(|s| want.contains(s)).cloned().collect(),
    ))
}

/// The default colorway: `requested` (must be compiled in), else the
/// table's default if compiled, else the first compiled one.
pub fn resolve_default(
    table: &Value,
    colors: &Option<Vec<String>>,
    requested: &str,
) -> Result<String, String> {
    let all = slugs(table);
    let compiled: Vec<String> = colors.clone().unwrap_or_else(|| all.clone());
    let requested = requested.trim().to_ascii_lowercase();
    if !requested.is_empty() {
        if !all.contains(&requested) {
            return Err(format!(
                "unknown logo color '{requested}'. Available: {}",
                all.join(", ")
            ));
        }
        if !compiled.contains(&requested) {
            return Err(format!(
                "the default logo color '{requested}' is not among the compiled ones ({})",
                compiled.join(", ")
            ));
        }
        return Ok(requested);
    }
    let file_default = table["default"].as_str().unwrap_or_default();
    if compiled.iter().any(|s| s == file_default) {
        return Ok(file_default.to_string());
    }
    compiled
        .first()
        .cloned()
        .ok_or_else(|| "the colorways table is empty".to_string())
}

/// The icon colorway: `requested` (any colorway) else `default`.
pub fn resolve_icon(table: &Value, requested: &str, default: &str) -> Result<String, String> {
    let requested = requested.trim().to_ascii_lowercase();
    if requested.is_empty() {
        return Ok(default.to_string());
    }
    if slugs(table).contains(&requested) {
        Ok(requested)
    } else {
        Err(format!(
            "unknown icon color '{requested}'. Available: {}",
            slugs(table).join(", ")
        ))
    }
}

/// The table with only the compiled colorways and the chosen default.
pub fn filter_table(table: &Value, colors: &Option<Vec<String>>, default: &str) -> Value {
    let mut out = table.clone();
    if let (Some(list), Some(ways)) = (colors, out["ways"].as_array_mut()) {
        ways.retain(|w| {
            w["slug"]
                .as_str()
                .is_some_and(|s| list.iter().any(|c| c == s))
        });
    }
    out["default"] = Value::String(default.to_string());
    out
}

fn hex(c: &str) -> [u8; 3] {
    let v = u32::from_str_radix(c.trim_start_matches('#'), 16).unwrap_or(0);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// RGBA of one sprite cell, `None` where it is empty.
fn cell(way: &Value, ch: char) -> Option<[u8; 3]> {
    let s = |k: &str| hex(way[k].as_str().unwrap_or("#000000"));
    let tip = |i: usize| hex(way["tips"][i].as_str().unwrap_or("#000000"));
    Some(match ch {
        'O' | 'K' => s("ink"),
        'P' => s("body"),
        'L' => s("light"),
        'W' => [255, 255, 255],
        'C' => s("cheek"),
        'T' => tip(0),
        'R' => tip(1),
        'S' => tip(2),
        'B' => tip(3),
        _ => return None,
    })
}

/// Sizes of the generated `.ico` (the 256 px entry is left out: stored
/// uncompressed it would weigh more than all the others together, and
/// Windows scales the 128 px one for the largest views).
pub const ICON_SIZES: [u32; 5] = [16, 32, 48, 64, 128];

/// A Windows `.ico` of colorway `slug`: 32-bit BMP entries, the 16x16 sprite
/// scaled by whole factors (nearest neighbor, so the pixels stay crisp).
pub fn ico(table: &Value, slug: &str) -> Result<Vec<u8>, String> {
    let way = table["ways"]
        .as_array()
        .and_then(|w| w.iter().find(|x| x["slug"] == slug))
        .ok_or_else(|| format!("unknown colorway '{slug}'"))?;
    let rows: Vec<Vec<char>> = table["grid"]
        .as_array()
        .ok_or("the table has no grid")?
        .iter()
        .map(|r| r.as_str().unwrap_or_default().chars().collect())
        .collect();
    if rows.len() != 16 || rows.iter().any(|r| r.len() != 16) {
        return Err("the logo grid must be 16x16".into());
    }
    let mut images: Vec<Vec<u8>> = Vec::new();
    for &size in &ICON_SIZES {
        let mut img = Vec::new();
        for v in [40u32, size, size * 2, 1 | (32 << 16), 0, 0, 0, 0, 0, 0] {
            img.extend_from_slice(&v.to_le_bytes());
        }
        // BITMAPINFOHEADER: planes (u16) and bpp (u16) share one u32 above.
        for y in (0..size).rev() {
            for x in 0..size {
                let ch = rows[(y * 16 / size) as usize][(x * 16 / size) as usize];
                match cell(way, ch) {
                    Some([r, g, b]) => img.extend_from_slice(&[b, g, r, 255]),
                    None => img.extend_from_slice(&[0, 0, 0, 0]),
                }
            }
        }
        // The AND mask (all zero: the alpha channel does the work).
        img.resize(
            img.len() + ((size as usize).div_ceil(32) * 4) * size as usize,
            0,
        );
        images.push(img);
    }
    let mut out = vec![0u8, 0, 1, 0];
    out.extend_from_slice(&(ICON_SIZES.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * ICON_SIZES.len() as u32;
    for (&size, img) in ICON_SIZES.iter().zip(&images) {
        out.extend_from_slice(&[size as u8, size as u8, 0, 0, 1, 0, 32, 0]);
        out.extend_from_slice(&(img.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        out.extend_from_slice(&img);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Value {
        serde_json::from_str(TABLE_JSON).unwrap()
    }

    fn all() -> Vec<String> {
        slugs(&table())
    }

    #[test]
    fn parsing_keeps_table_order_and_rejects_unknowns() {
        assert_eq!(parse_colors("", &all()).unwrap(), None);
        assert_eq!(parse_colors("all", &all()).unwrap(), None);
        let two = parse_colors("teal, Violet,teal", &all()).unwrap().unwrap();
        assert_eq!(two, ["violet", "teal"]);
        assert!(parse_colors("nope", &all())
            .unwrap_err()
            .contains("unknown logo color 'nope'"));
        assert!(parse_colors("none", &all()).is_err());
    }

    #[test]
    fn the_default_must_be_compiled_and_falls_back_sensibly() {
        let t = table();
        let two = parse_colors("violet,teal", &all()).unwrap();
        assert_eq!(resolve_default(&t, &two, "teal").unwrap(), "teal");
        assert!(resolve_default(&t, &two, "sun")
            .unwrap_err()
            .contains("not among the compiled"));
        // lavender (the table's default) is not compiled: first compiled wins.
        assert_eq!(resolve_default(&t, &two, "").unwrap(), "violet");
        assert_eq!(resolve_default(&t, &None, "").unwrap(), "lavender");
    }

    #[test]
    fn filtering_keeps_only_the_chosen_ways_and_sets_the_default() {
        let t = table();
        let sel = parse_colors("violet,teal", &all()).unwrap();
        let f = filter_table(&t, &sel, "teal");
        assert_eq!(slugs(&f), ["violet", "teal"]);
        assert_eq!(f["default"], "teal");
        assert_eq!(slugs(&filter_table(&t, &None, "sun")).len(), all().len());
    }

    #[test]
    fn the_icon_color_is_any_colorway() {
        let t = table();
        assert_eq!(resolve_icon(&t, "", "violet").unwrap(), "violet");
        assert_eq!(resolve_icon(&t, "sun", "violet").unwrap(), "sun");
        assert!(resolve_icon(&t, "zzz", "violet").is_err());
    }

    #[test]
    fn the_ico_is_well_formed_and_uses_the_colorway() {
        let t = table();
        let bytes = ico(&t, "orange").unwrap();
        assert_eq!(&bytes[..6], [0, 0, 1, 0, ICON_SIZES.len() as u8, 0]);
        // Directory entries point inside the file and the sizes match.
        let mut end = 0;
        for (i, &size) in ICON_SIZES.iter().enumerate() {
            let e = &bytes[6 + 16 * i..22 + 16 * i];
            assert_eq!(e[0] as u32, size);
            let len = u32::from_le_bytes(e[8..12].try_into().unwrap()) as usize;
            let off = u32::from_le_bytes(e[12..16].try_into().unwrap()) as usize;
            assert!(off + len <= bytes.len());
            assert_eq!(
                len,
                40 + (size * size * 4) as usize + (size.div_ceil(32) * 4 * size) as usize
            );
            end = off + len;
        }
        assert_eq!(end, bytes.len());
        // The 16 px image's body color (cell row 3, col 6: 'P') is orange F97316 as BGRA.
        let off = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
        let px = |x: usize, y: usize| {
            let at = off + 40 + ((15 - y) * 16 + x) * 4;
            bytes[at..at + 4].to_vec()
        };
        assert_eq!(px(6, 3), [0x16, 0x73, 0xF9, 255]);
        assert_eq!(px(0, 0)[3], 0, "transparent corner");
        assert_ne!(ico(&t, "teal").unwrap(), bytes);
        assert!(ico(&t, "zzz").is_err());
    }
}
