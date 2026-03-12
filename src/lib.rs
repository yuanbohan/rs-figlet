//! You can visit [`figlet`] and [`figfont`] to find more details.
//! You can visit [`fontdb`] to find more fonts.
//!
//! # Examples
//!
//! Convert string literals using built-in FIGlet or Toilet fonts:
//!
//! ```
//! use figlet_rs::{FIGlet, Toilet};
//!
//! let standard_font = FIGlet::standard().unwrap();
//! assert!(standard_font.convert("FIGlet").is_some());
//!
//! let slant_font = FIGlet::slant().unwrap();
//! assert!(slant_font.convert("FIGlet").is_some());
//!
//! let smblock_font = Toilet::smblock().unwrap();
//! assert!(smblock_font.convert("Toilet").is_some());
//! ```
//!
//! [`figlet`]: http://www.figlet.org
//! [`figfont`]: http://www.jave.de/figlet/figfont.html
//! [`fontdb`]: http://www.figlet.org/fontdb.cgi

mod figlet;
mod shared;
mod toilet;

pub use figlet::FIGlet;
pub use shared::{FIGcharacter, FIGure, HeaderLine};
pub use toilet::Toilet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{
        FontData, SM_BIGX, SM_EQUAL, SM_HARDBLANK, SM_HIERARCHY, SM_KERN, SM_LOWLINE, SM_PAIR,
        SM_SMUSH,
    };
    use std::fs;
    use std::path::Path;

    fn fixture(path: &str) -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        fs::read_to_string(root.join(path)).unwrap()
    }

    fn full_smush_font() -> FIGlet {
        let mut font = FIGlet::standard().unwrap();
        font.header_line.full_layout = Some(
            SM_EQUAL | SM_LOWLINE | SM_HIERARCHY | SM_PAIR | SM_BIGX | SM_HARDBLANK | SM_SMUSH,
        );
        font
    }

    fn assert_golden_fixture_figlet(font: &FIGlet, message: &str, fixture_path: &str) {
        let figure = font.convert(message).unwrap();
        assert_eq!(fixture(fixture_path), figure.as_str());
    }

    fn assert_golden_fixture_toilet(font: &Toilet, message: &str, fixture_path: &str) {
        let figure = font.convert(message).unwrap();
        assert_eq!(fixture(fixture_path), figure.as_str());
    }

    #[test]
    fn test_new_headerline() {
        let line = "flf2a$ 6 5 20 15 3 0 143 229";
        let headerline = HeaderLine::try_from(line).unwrap();

        assert_eq!(line, headerline.header_line);
        assert_eq!("flf2a", headerline.signature);
        assert_eq!('$', headerline.hardblank);
        assert_eq!(6, headerline.height);
        assert_eq!(5, headerline.baseline);
        assert_eq!(20, headerline.max_length);
        assert_eq!(15, headerline.old_layout);
        assert_eq!(3, headerline.comment_lines);
        assert_eq!(Some(0), headerline.print_direction);
        assert_eq!(Some(143), headerline.full_layout);
        assert_eq!(Some(229), headerline.codetag_count);
    }

    #[test]
    fn test_new_figfont() {
        let font = FIGlet::standard().unwrap();

        assert_eq!("flf2a$ 6 5 16 15 11 0 24463", font.header_line.header_line);
        assert_eq!("flf2a", font.header_line.signature);
        assert_eq!('$', font.header_line.hardblank);
        assert_eq!(6, font.header_line.height);
        assert_eq!(5, font.header_line.baseline);
        assert_eq!(16, font.header_line.max_length);
        assert_eq!(15, font.header_line.old_layout);
        assert_eq!(11, font.header_line.comment_lines);
        assert_eq!(Some(0), font.header_line.print_direction);
        assert_eq!(Some(24463), font.header_line.full_layout);
        assert_eq!(None, font.header_line.codetag_count);

        let one_font = font.fonts.get(&('F' as u32)).unwrap();
        assert_eq!(70, one_font.code);
        assert_eq!(8, one_font.width);
        assert_eq!(6, one_font.height);
        assert_eq!(6, one_font.characters.len());
        assert_eq!("  _____ ", one_font.characters.first().unwrap());
    }

    #[test]
    fn test_convert() {
        let standard_font = FIGlet::standard().unwrap();
        let figure = standard_font.convert("FIGlet").unwrap();

        assert_eq!(6, figure.height);
        assert_eq!(6, figure.characters.len());
        assert_eq!("  _____ ", figure.characters[0].characters.first().unwrap());
    }

    #[test]
    fn test_convert_empty_string() {
        let font = FIGlet::standard().unwrap();
        assert!(font.convert("").is_none());
    }

    #[test]
    fn test_convert_single_character() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("A").unwrap();
        assert_eq!(1, figure.characters.len());
        assert_eq!(6, figure.height);
    }

    #[test]
    fn test_convert_all_ascii_printable() {
        let font = FIGlet::standard().unwrap();
        let all_ascii: String = (32..=126).map(|c| char::from_u32(c).unwrap()).collect();
        let figure = font.convert(&all_ascii).unwrap();
        assert_eq!(95, figure.characters.len());
    }

    #[test]
    fn test_convert_with_unknown_characters() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("Hello世界").unwrap();
        assert_eq!(5, figure.characters.len());
    }

    #[test]
    fn test_figure_as_str() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("Hi").unwrap();
        let s = figure.as_str();
        assert!(!s.is_empty());
        assert_eq!(figure.height as usize, s.lines().count());
    }

    #[test]
    fn test_figure_display() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("AB").unwrap();
        let display_output = format!("{}", figure);
        let debug_output = format!("{:?}", figure);

        assert!(!display_output.is_empty());
        assert!(display_output.contains('\n'));
        assert!(!debug_output.is_empty());
    }

    #[test]
    fn test_standard_golden_samples() {
        let font = FIGlet::standard().unwrap();
        assert_golden_fixture_figlet(&font, "Test", "tests/fixtures/figlet_standard_test.txt");
        assert_golden_fixture_figlet(&font, "FIGlet", "tests/fixtures/figlet_standard_figlet.txt");
        assert_golden_fixture_figlet(
            &font,
            "-4.5",
            "tests/fixtures/figlet_standard_negative_float.txt",
        );
        assert_golden_fixture_figlet(
            &font,
            "Hello Rust",
            "tests/fixtures/figlet_standard_hello_rust.txt",
        );
    }

    #[test]
    fn test_small_golden_samples() {
        let font = FIGlet::small().unwrap();
        assert_golden_fixture_figlet(&font, "Test", "tests/fixtures/figlet_small_test.txt");
        assert_golden_fixture_figlet(&font, "FIGlet", "tests/fixtures/figlet_small_figlet.txt");
        assert_golden_fixture_figlet(
            &font,
            "-4.5",
            "tests/fixtures/figlet_small_negative_float.txt",
        );
        assert_golden_fixture_figlet(&font, "Hello Rust", "tests/fixtures/figlet_small_hello_rust.txt");
    }

    #[test]
    fn test_effective_layout_prefers_full_layout() {
        let header = HeaderLine::try_from("flf2a$ 6 5 20 15 3 0 143 229").unwrap();
        assert_eq!(143, header.effective_layout());
    }

    #[test]
    fn test_effective_layout_uses_old_layout_compatibility() {
        let kerning = HeaderLine::try_from("flf2a$ 6 5 20 0 3").unwrap();
        let full_width = HeaderLine::try_from("flf2a$ 6 5 20 -1 3").unwrap();
        let smushing = HeaderLine::try_from("flf2a$ 6 5 20 15 3").unwrap();

        assert_eq!(SM_KERN, kerning.effective_layout());
        assert_eq!(0, full_width.effective_layout());
        assert_eq!(143, smushing.effective_layout());
    }

    #[test]
    fn test_smush_rule_equal() {
        let font = full_smush_font();
        let renderer = crate::shared::render(&font.header_line, &font.fonts, "||").unwrap();
        assert!(renderer.is_not_empty());
    }

    #[test]
    fn test_smush_rule_lowline_and_hierarchy() {
        let font = full_smush_font();
        let figure = font.convert("_/|>").unwrap();
        assert!(figure.is_not_empty());
    }

    #[test]
    fn test_figure_is_not_empty() {
        let font = FIGlet::standard().unwrap();
        assert!(font.convert("Test").unwrap().is_not_empty());
    }

    #[test]
    fn test_figure_with_only_unknown_chars() {
        let font = FIGlet::standard().unwrap();
        assert!(font.convert("\u{4E2D}\u{6587}").is_none());
    }

    #[test]
    fn test_figfont_clone() {
        let font1 = FIGlet::standard().unwrap();
        let font2 = font1.clone();

        assert!(font1.convert("Test").is_some());
        assert!(font2.convert("Test").is_some());
    }

    #[test]
    fn test_standard_font_loading() {
        let font1 = FIGlet::standard().unwrap();
        let font2 = FIGlet::from_file("resources/standard.flf").unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_small_font_loading() {
        let font1 = FIGlet::small().unwrap();
        let font2 = FIGlet::from_file("resources/small.flf").unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_big_golden_samples() {
        let font = FIGlet::big().unwrap();
        assert_golden_fixture_figlet(&font, "Test", "tests/fixtures/figlet_big_test.txt");
        assert_golden_fixture_figlet(&font, "FIGlet", "tests/fixtures/figlet_big_figlet.txt");
        assert_golden_fixture_figlet(
            &font,
            "-4.5",
            "tests/fixtures/figlet_big_negative_float.txt",
        );
        assert_golden_fixture_figlet(&font, "Hello Rust", "tests/fixtures/figlet_big_hello_rust.txt");
    }

    #[test]
    fn test_slant_golden_samples() {
        let font = FIGlet::slant().unwrap();
        assert_golden_fixture_figlet(&font, "Test", "tests/fixtures/figlet_slant_test.txt");
        assert_golden_fixture_figlet(&font, "FIGlet", "tests/fixtures/figlet_slant_figlet.txt");
        assert_golden_fixture_figlet(
            &font,
            "-4.5",
            "tests/fixtures/figlet_slant_negative_float.txt",
        );
        assert_golden_fixture_figlet(&font, "Hello Rust", "tests/fixtures/figlet_slant_hello_rust.txt");
    }

    #[test]
    fn test_big_font_loading() {
        let font1 = FIGlet::big().unwrap();
        let font2 = FIGlet::from_file("resources/big.flf").unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_slant_font_loading() {
        let font1 = FIGlet::slant().unwrap();
        let font2 = FIGlet::from_file("resources/slant.flf").unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_figure_character_width_and_height() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("ABC").unwrap();

        for char in &figure.characters {
            assert_eq!(figure.height, char.height);
            assert!(char.width > 0);
        }
    }

    #[test]
    fn test_latin_characters() {
        let font = FIGlet::standard().unwrap();
        let figure = font.convert("\u{00C4}\u{00D6}\u{00DC}").unwrap();
        assert_eq!(3, figure.characters.len());
    }

    #[test]
    fn test_from_content_invalid() {
        assert!(FIGlet::from_content("").is_err());
        assert!(FIGlet::from_content("invalid").is_err());
    }

    #[test]
    fn test_headerline_invalid() {
        assert!(HeaderLine::try_from("flf2a$ 6").is_err());
    }

    #[test]
    fn test_toilet_header_supports_tlf_signature() {
        let header = HeaderLine::try_from("tlf2a$ 4 3 8 0 16 0 64 0").unwrap();
        assert_eq!("tlf2a", header.signature);
        assert_eq!(4, header.height);
    }

    #[test]
    fn test_toilet_from_content() {
        let content = fixture("resources/smblock.tlf");
        let font = Toilet::from_content(&content).unwrap();
        assert_eq!("tlf2a", font.header_line.signature);
        assert!(font.convert("Test").is_some());
    }

    #[test]
    fn test_toilet_builtin_text_font_loading() {
        let font1 = Toilet::smblock().unwrap();
        let font2 = Toilet::from_file("resources/smblock.tlf").unwrap();
        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_toilet_builtin_zipped_font_loading() {
        let font1 = Toilet::mono12().unwrap();
        let font2 = Toilet::from_file("resources/mono12.tlf").unwrap();
        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_toilet_smblock_golden_samples() {
        let font = Toilet::smblock().unwrap();
        assert_golden_fixture_toilet(&font, "Test", "tests/fixtures/toilet_smblock_test.txt");
        assert_golden_fixture_toilet(&font, "FIGlet", "tests/fixtures/toilet_smblock_figlet.txt");
        assert_golden_fixture_toilet(
            &font,
            "-4.5",
            "tests/fixtures/toilet_smblock_negative_float.txt",
        );
    }

    #[test]
    fn test_toilet_future_golden_samples() {
        let font = Toilet::future().unwrap();
        assert_golden_fixture_toilet(&font, "Test", "tests/fixtures/toilet_future_test.txt");
        assert_golden_fixture_toilet(
            &font,
            "Hello Rust",
            "tests/fixtures/toilet_future_hello_rust.txt",
        );
    }

    #[test]
    fn test_toilet_wideterm_golden_samples() {
        let font = Toilet::wideterm().unwrap();
        assert_golden_fixture_toilet(&font, "FIGlet", "tests/fixtures/toilet_wideterm_figlet.txt");
    }

    #[test]
    fn test_toilet_mono12_golden_samples() {
        let font = Toilet::mono12().unwrap();
        assert_golden_fixture_toilet(&font, "Test", "tests/fixtures/toilet_mono12_test.txt");
    }

    #[test]
    fn test_toilet_mono9_golden_samples() {
        let font = Toilet::mono9().unwrap();
        assert_golden_fixture_toilet(
            &font,
            "Hello Rust",
            "tests/fixtures/toilet_mono9_hello_rust.txt",
        );
    }

    #[test]
    fn test_toilet_external_zipped_font_matches_builtin() {
        let external = Toilet::from_file("resources/mono9.tlf").unwrap();
        let builtin = Toilet::mono9().unwrap();
        assert_eq!(
            builtin.convert("Test").unwrap().as_str(),
            external.convert("Test").unwrap().as_str()
        );
    }

    #[test]
    fn test_toilet_unknown_chars_are_skipped() {
        let font = Toilet::smblock().unwrap();
        let figure = font.convert("Toilet世界").unwrap();
        assert_eq!(6, figure.characters.len());
    }

    #[test]
    fn test_toilet_from_content_invalid() {
        assert!(Toilet::from_content("").is_err());
    }

    #[test]
    fn test_from_font_data_roundtrip() {
        let font = FIGlet::standard().unwrap();
        let data = FontData {
            header_line: font.header_line.clone(),
            comments: font.comments.clone(),
            fonts: font.fonts.clone(),
        };
        let cloned = FIGlet::from(data);
        assert_eq!(
            font.convert("Test").unwrap().as_str(),
            cloned.convert("Test").unwrap().as_str()
        );
    }
}
