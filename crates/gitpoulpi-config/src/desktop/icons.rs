//! Container formats for the shortcut icon, built from PNGs the caller
//! rendered (the TUI draws the pixel octopus in the user's color): `.ico`
//! for Windows shortcuts, `.icns` for the macOS bundle. Both formats can
//! carry PNG data as is, so there is no pixel work here.

/// The icon as PNGs, `(side in pixels, PNG bytes)`, smallest first. The
/// largest one should be 256 px: it is the Linux hicolor icon.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IconSet {
    pub pngs: Vec<(u32, Vec<u8>)>,
}

impl IconSet {
    /// The biggest PNG (the 256 px one).
    pub fn largest(&self) -> Option<&[u8]> {
        self.pngs
            .iter()
            .max_by_key(|(side, _)| *side)
            .map(|(_, png)| png.as_slice())
    }

    /// A Windows `.ico` holding every PNG (Vista and later read PNG entries).
    pub fn ico(&self) -> Vec<u8> {
        let mut out = vec![0, 0, 1, 0];
        out.extend_from_slice(&(self.pngs.len() as u16).to_le_bytes());
        let mut offset = 6 + 16 * self.pngs.len() as u32;
        for (side, png) in &self.pngs {
            // 0 stands for 256.
            let byte = if *side >= 256 { 0 } else { *side as u8 };
            out.extend_from_slice(&[byte, byte, 0, 0]);
            out.extend_from_slice(&1u16.to_le_bytes()); // planes
            out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
            out.extend_from_slice(&(png.len() as u32).to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            offset += png.len() as u32;
        }
        for (_, png) in &self.pngs {
            out.extend_from_slice(png);
        }
        out
    }

    /// A macOS `.icns` with the largest PNG as its 256 px entry (`ic08`).
    pub fn icns(&self) -> Vec<u8> {
        let png = self.largest().unwrap_or_default();
        let entry = 8 + png.len() as u32;
        let mut out = b"icns".to_vec();
        out.extend_from_slice(&(8 + entry).to_be_bytes());
        out.extend_from_slice(b"ic08");
        out.extend_from_slice(&entry.to_be_bytes());
        out.extend_from_slice(png);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set() -> IconSet {
        IconSet {
            pngs: vec![(16, vec![1; 5]), (256, vec![2; 7])],
        }
    }

    #[test]
    fn ico_has_a_directory_entry_per_png_with_offsets_into_the_file() {
        let ico = set().ico();
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 2, 0]);
        assert_eq!(ico[6], 16);
        assert_eq!(ico[22], 0, "256 is stored as 0");
        let offset = u32::from_le_bytes(ico[18..22].try_into().unwrap()) as usize;
        assert_eq!(&ico[offset..offset + 5], &[1; 5]);
        let offset = u32::from_le_bytes(ico[34..38].try_into().unwrap()) as usize;
        assert_eq!(&ico[offset..], &[2; 7]);
    }

    #[test]
    fn icns_wraps_the_largest_png() {
        let icns = set().icns();
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
            icns.len()
        );
        assert_eq!(&icns[8..12], b"ic08");
        assert_eq!(&icns[16..], &[2; 7]);
    }
}
