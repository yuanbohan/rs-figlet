//! you can visit [`figlet`] and [`figfont`] to find more details.
//! you can visit [`fongdb`] to find more font.
//!
//! # Examples
//!
//! convert string literal using standard or specified font:
//!
//! ```
//! use figlet_rs::FIGfont;
//!
//! let standard_font = FIGfont::standard().unwrap();
//! let figure = standard_font.convert("FIGlet");
//! assert!(figure.is_some());
//!
//! let small_font = FIGfont::small().unwrap();
//! let figure = small_font.convert("FIGlet");
//! assert!(figure.is_some());
//! ```
//! [`figlet`]: http://www.figlet.org
//! [`figfont`]: http://www.jave.de/figlet/figfont.html
//! [`fongdb`]: http://www.figlet.org/fontdb.cgi
//! [`small.flf`]: http://www.figlet.org/fonts/small.flf

use std::collections::HashMap;
use std::{fmt, fs};

const SM_EQUAL: i32 = 1;
const SM_LOWLINE: i32 = 2;
const SM_HIERARCHY: i32 = 4;
const SM_PAIR: i32 = 8;
const SM_BIGX: i32 = 16;
const SM_HARDBLANK: i32 = 32;
const SM_KERN: i32 = 64;
const SM_SMUSH: i32 = 128;

/// FIGlet font, which will hold the mapping from u32 code to FIGcharacter
#[derive(Debug, Clone)]
pub struct FIGfont {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

impl FIGfont {
    fn read_font_file(filename: &str) -> Result<String, String> {
        fs::read_to_string(filename).map_err(|e| format!("{e:?}"))
    }

    fn read_header_line(header_line: &str) -> Result<HeaderLine, String> {
        HeaderLine::try_from(header_line)
    }

    fn read_comments(lines: &[&str], comment_count: i32) -> Result<String, String> {
        let length = lines.len() as i32;
        if length < comment_count + 1 {
            Err("can't get comments from font".to_string())
        } else {
            let comment = lines[1..(1 + comment_count) as usize].join("\n");
            Ok(comment)
        }
    }

    fn extract_one_line(
        lines: &[&str],
        index: usize,
        height: usize,
        _hardblank: char,
        is_last_index: bool,
    ) -> Result<String, String> {
        let line = lines
            .get(index)
            .ok_or(format!("can't get line at specified index:{index}"))?;

        let mut width = line.len() - 1;
        if is_last_index && height != 1 {
            width -= 1;
        }

        Ok(line[..width].to_string())
    }

    fn extract_one_font(
        lines: &[&str],
        code: u32,
        start_index: usize,
        height: usize,
        hardblank: char,
    ) -> Result<FIGcharacter, String> {
        let mut characters = vec![];
        for i in 0..height {
            let index = start_index + i;
            let is_last_index = i == height - 1;
            let one_line_character =
                FIGfont::extract_one_line(lines, index, height, hardblank, is_last_index)?;
            characters.push(one_line_character);
        }
        let width = characters[0].chars().count() as u32;
        let height = height as u32;

        Ok(FIGcharacter {
            code,
            characters,
            width,
            height,
        })
    }

    // 32-126, 196, 214, 220, 228, 246, 252, 223
    fn read_required_font(
        lines: &[&str],
        headerline: &HeaderLine,
        map: &mut HashMap<u32, FIGcharacter>,
    ) -> Result<(), String> {
        let offset = (1 + headerline.comment_lines) as usize;
        let height = headerline.height as usize;
        let size = lines.len();

        for i in 0..=94 {
            let code = (i + 32) as u32;
            let start_index = offset + i * height;
            if start_index >= size {
                break;
            }

            let font =
                FIGfont::extract_one_font(lines, code, start_index, height, headerline.hardblank)?;
            map.insert(code, font);
        }

        let offset = offset + 95 * height;
        let required_deutsch_characters_codes: [u32; 7] = [196, 214, 220, 228, 246, 252, 223];
        for (i, code) in required_deutsch_characters_codes.iter().enumerate() {
            let start_index = offset + i * height;
            if start_index >= size {
                break;
            }

            let font =
                FIGfont::extract_one_font(lines, *code, start_index, height, headerline.hardblank)?;
            map.insert(*code, font);
        }

        Ok(())
    }

    fn extract_codetag_font_code(lines: &[&str], index: usize) -> Result<u32, String> {
        let line = lines
            .get(index)
            .ok_or_else(|| "get codetag line error".to_string())?;

        let infos: Vec<&str> = line.trim().split(' ').collect();
        if infos.is_empty() {
            return Err("extract code for codetag font error".to_string());
        }

        let code = infos[0].trim();

        let code = if let Some(s) = code.strip_prefix("0x") {
            u32::from_str_radix(s, 16)
        } else if let Some(s) = code.strip_prefix("0X") {
            u32::from_str_radix(s, 16)
        } else if let Some(s) = code.strip_prefix('0') {
            u32::from_str_radix(s, 8)
        } else {
            code.parse()
        };

        code.map_err(|e| format!("{e:?}"))
    }

    fn read_codetag_font(
        lines: &[&str],
        headerline: &HeaderLine,
        map: &mut HashMap<u32, FIGcharacter>,
    ) -> Result<(), String> {
        let offset = (1 + headerline.comment_lines + 102 * headerline.height) as usize;
        let codetag_height = (headerline.height + 1) as usize;
        let codetag_lines = lines.len() - offset;

        if codetag_lines % codetag_height != 0 {
            return Err("codetag font is illegal.".to_string());
        }

        let size = codetag_lines / codetag_height;

        for i in 0..size {
            let start_index = offset + i * codetag_height;
            if start_index >= lines.len() {
                break;
            }

            let code = FIGfont::extract_codetag_font_code(lines, start_index)?;
            let font = FIGfont::extract_one_font(
                lines,
                code,
                start_index + 1,
                headerline.height as usize,
                headerline.hardblank,
            )?;
            map.insert(code, font);
        }

        Ok(())
    }

    fn read_fonts(
        lines: &[&str],
        headerline: &HeaderLine,
    ) -> Result<HashMap<u32, FIGcharacter>, String> {
        let mut map = HashMap::new();
        FIGfont::read_required_font(lines, headerline, &mut map)?;
        FIGfont::read_codetag_font(lines, headerline, &mut map)?;
        Ok(map)
    }

    /// generate FIGlet font from string literal
    pub fn from_content(contents: &str) -> Result<FIGfont, String> {
        let lines: Vec<&str> = contents.lines().collect();

        if lines.is_empty() {
            return Err("can not generate FIGlet font from empty string".to_string());
        }

        let header_line = FIGfont::read_header_line(lines.first().unwrap())?;
        let comments = FIGfont::read_comments(&lines, header_line.comment_lines)?;
        let fonts = FIGfont::read_fonts(&lines, &header_line)?;

        Ok(FIGfont {
            header_line,
            comments,
            fonts,
        })
    }

    /// generate FIGlet font from specified file
    pub fn from_file(fontname: &str) -> Result<FIGfont, String> {
        let contents = FIGfont::read_font_file(fontname)?;
        FIGfont::from_content(&contents)
    }

    /// the standard FIGlet font, which you can find [`fontdb`]
    ///
    /// [`fontdb`]: http://www.figlet.org/fontdb.cgi
    pub fn standard() -> Result<FIGfont, String> {
        let contents = std::include_str!("standard.flf");
        FIGfont::from_content(contents)
    }

    /// the small FIGlet font bundled with the crate
    pub fn small() -> Result<FIGfont, String> {
        let contents = std::include_str!("../resources/small.flf");
        FIGfont::from_content(contents)
    }

    /// convert string literal to FIGure
    pub fn convert(&self, message: &str) -> Option<FIGure<'_>> {
        if message.is_empty() {
            return None;
        }

        let mut characters: Vec<&FIGcharacter> = vec![];
        for ch in message.chars() {
            let code = ch as u32;
            if let Some(character) = self.fonts.get(&code) {
                characters.push(character);
            }
        }

        if characters.is_empty() {
            return None;
        }

        let rendered_lines = Renderer::new(self).render(&characters);

        Some(FIGure {
            characters,
            height: self.header_line.height as u32,
            lines: rendered_lines,
        })
    }
}

/// the first line in FIGlet font, which you can find the documentation [`headerline`]
///
/// [`headerline`]: http://www.jave.de/figlet/figfont.html#headerline
#[derive(Debug, Clone)]
pub struct HeaderLine {
    pub header_line: String,

    // required
    pub signature: String,
    pub hardblank: char,
    pub height: i32,
    pub baseline: i32,
    pub max_length: i32,
    pub old_layout: i32, // Legal values -1 to 63
    pub comment_lines: i32,

    // optional
    pub print_direction: Option<i32>,
    pub full_layout: Option<i32>, // Legal values 0 to 32767
    pub codetag_count: Option<i32>,
}

impl HeaderLine {
    fn extract_signature_with_hardblank(
        signature_with_hardblank: &str,
    ) -> Result<(String, char), String> {
        if signature_with_hardblank.len() < 6 {
            Err("can't get signature with hardblank from first line of font".to_string())
        } else {
            let hardblank_index = signature_with_hardblank.len() - 1;
            let signature = &signature_with_hardblank[..hardblank_index];
            let hardblank = signature_with_hardblank[hardblank_index..]
                .chars()
                .next()
                .unwrap();

            Ok((String::from(signature), hardblank))
        }
    }

    fn extract_required_info(infos: &[&str], index: usize, field: &str) -> Result<i32, String> {
        let val = match infos.get(index) {
            Some(val) => Ok(val),
            None => Err(format!(
                "can't get field:{field} index:{index} from {}",
                infos.join(",")
            )),
        }?;

        val.parse()
            .map_err(|_| format!("can't parse required field:{field} of {val} to i32"))
    }

    fn extract_optional_info(infos: &[&str], index: usize, _field: &str) -> Option<i32> {
        if let Some(val) = infos.get(index) {
            val.parse().ok()
        } else {
            None
        }
    }

    fn effective_layout(&self) -> i32 {
        match self.full_layout {
            Some(layout) => layout,
            None if self.old_layout == 0 => SM_KERN,
            None if self.old_layout < 0 => 0,
            None => (self.old_layout & 31) | SM_SMUSH,
        }
    }

    fn is_right_to_left(&self) -> bool {
        self.print_direction == Some(1)
    }
}

impl TryFrom<&str> for HeaderLine {
    type Error = String;

    fn try_from(header_line: &str) -> Result<Self, Self::Error> {
        let infos: Vec<&str> = header_line.trim().split(' ').collect();

        if infos.len() < 6 {
            return Err("headerline is illegal".to_string());
        }

        let signature_with_hardblank =
            HeaderLine::extract_signature_with_hardblank(infos.first().unwrap())?;

        let height = HeaderLine::extract_required_info(&infos, 1, "height")?;
        let baseline = HeaderLine::extract_required_info(&infos, 2, "baseline")?;
        let max_length = HeaderLine::extract_required_info(&infos, 3, "max length")?;
        let old_layout = HeaderLine::extract_required_info(&infos, 4, "old layout")?;
        let comment_lines = HeaderLine::extract_required_info(&infos, 5, "comment lines")?;

        let print_direction = HeaderLine::extract_optional_info(&infos, 6, "print direction");
        let full_layout = HeaderLine::extract_optional_info(&infos, 7, "full layout");
        let codetag_count = HeaderLine::extract_optional_info(&infos, 8, "codetag count");

        Ok(HeaderLine {
            header_line: String::from(header_line),
            signature: signature_with_hardblank.0,
            hardblank: signature_with_hardblank.1,
            height,
            baseline,
            max_length,
            old_layout,
            comment_lines,
            print_direction,
            full_layout,
            codetag_count,
        })
    }
}

/// the matched ascii art of one character
#[derive(Debug, Clone)]
pub struct FIGcharacter {
    pub code: u32,
    pub characters: Vec<String>,
    pub width: u32,
    pub height: u32,
}

impl fmt::Display for FIGcharacter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.characters.join("\n"))
    }
}

/// the matched ascii arts of string literal
#[derive(Debug)]
pub struct FIGure<'a> {
    pub characters: Vec<&'a FIGcharacter>,
    pub height: u32,
    lines: Vec<String>,
}

impl<'a> FIGure<'a> {
    fn is_not_empty(&self) -> bool {
        !self.characters.is_empty() && self.height > 0
    }

    /// Returns the FIGure as a String
    pub fn as_str(&self) -> String {
        self.to_string()
    }
}

impl<'a> fmt::Display for FIGure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_not_empty() {
            for line in &self.lines {
                writeln!(f, "{}", line)?;
            }
            Ok(())
        } else {
            write!(f, "")
        }
    }
}

struct Renderer<'a> {
    font: &'a FIGfont,
    prev_char_width: usize,
    cur_char_width: usize,
    max_smush: usize,
}

impl<'a> Renderer<'a> {
    fn new(font: &'a FIGfont) -> Self {
        Self {
            font,
            prev_char_width: 0,
            cur_char_width: 0,
            max_smush: 0,
        }
    }

    fn render(mut self, characters: &[&FIGcharacter]) -> Vec<String> {
        let mut buffer = vec![String::new(); self.font.header_line.height as usize];
        for character in characters {
            self.cur_char_width = character.width as usize;
            self.max_smush = self.smush_amount(&buffer, character);

            for row in 0..buffer.len() {
                self.add_char_row_to_buffer_row(&mut buffer[row], &character.characters[row]);
            }

            self.prev_char_width = self.cur_char_width;
        }

        buffer
            .into_iter()
            .map(|line| line.replace(self.font.header_line.hardblank, " "))
            .collect()
    }

    fn add_char_row_to_buffer_row(&self, buffer_row: &mut String, char_row: &str) {
        let (mut left, right) = if self.font.header_line.is_right_to_left() {
            (
                char_row.chars().collect::<Vec<_>>(),
                buffer_row.chars().collect::<Vec<_>>(),
            )
        } else {
            (
                buffer_row.chars().collect::<Vec<_>>(),
                char_row.chars().collect::<Vec<_>>(),
            )
        };

        for i in 0..self.max_smush {
            let idx = left.len() as isize - self.max_smush as isize + i as isize;
            let left_ch = if idx >= 0 {
                left.get(idx as usize).copied().unwrap_or('\0')
            } else {
                '\0'
            };
            let right_ch = right.get(i).copied().unwrap_or('\0');
            if let Some(smushed) = self.smush_chars(left_ch, right_ch) {
                if idx >= 0 {
                    left[idx as usize] = smushed;
                }
            }
        }

        left.extend(right.into_iter().skip(self.max_smush));
        *buffer_row = left.into_iter().collect();
    }

    fn smush_amount(&self, buffer: &[String], character: &FIGcharacter) -> usize {
        let layout = self.font.header_line.effective_layout();
        if (layout & (SM_SMUSH | SM_KERN)) == 0 {
            return 0;
        }

        let mut max_smush = self.cur_char_width;
        for row in 0..self.font.header_line.height as usize {
            let (line_left, line_right) = if self.font.header_line.is_right_to_left() {
                (&character.characters[row], &buffer[row])
            } else {
                (&buffer[row], &character.characters[row])
            };

            let left_chars: Vec<char> = line_left.chars().collect();
            let right_chars: Vec<char> = line_right.chars().collect();

            let trimmed_left_len = left_chars
                .iter()
                .rposition(|ch| *ch != ' ')
                .map_or(0, |idx| idx + 1);
            let linebd = trimmed_left_len.saturating_sub(1);
            let ch1 = if trimmed_left_len == 0 {
                '\0'
            } else {
                left_chars[linebd]
            };

            let charbd = right_chars
                .iter()
                .position(|ch| *ch != ' ')
                .unwrap_or(right_chars.len());
            let ch2 = if charbd < right_chars.len() {
                right_chars[charbd]
            } else {
                '\0'
            };

            let mut amount = charbd as isize + left_chars.len() as isize - 1 - linebd as isize;
            if ch1 == '\0' || ch1 == ' ' {
                amount += 1;
            } else if ch2 != '\0' && self.smush_chars(ch1, ch2).is_some() {
                amount += 1;
            }

            max_smush = max_smush.min(amount.max(0) as usize);
        }

        max_smush
    }

    fn smush_chars(&self, left: char, right: char) -> Option<char> {
        if left == ' ' {
            return Some(right);
        }
        if right == ' ' {
            return Some(left);
        }
        if left == '\0' || right == '\0' {
            return None;
        }
        if self.prev_char_width < 2 || self.cur_char_width < 2 {
            return None;
        }

        let layout = self.font.header_line.effective_layout();
        if (layout & SM_SMUSH) == 0 {
            return None;
        }

        if (layout & 63) == 0 {
            if left == self.font.header_line.hardblank {
                return Some(right);
            }
            if right == self.font.header_line.hardblank {
                return Some(left);
            }

            return if self.font.header_line.is_right_to_left() {
                Some(left)
            } else {
                Some(right)
            };
        }

        if (layout & SM_HARDBLANK) != 0
            && left == self.font.header_line.hardblank
            && right == self.font.header_line.hardblank
        {
            return Some(left);
        }
        if left == self.font.header_line.hardblank || right == self.font.header_line.hardblank {
            return None;
        }
        if (layout & SM_EQUAL) != 0 && left == right {
            return Some(left);
        }
        if (layout & SM_LOWLINE) != 0 {
            if left == '_' && "|/\\[]{}()<>".contains(right) {
                return Some(right);
            }
            if right == '_' && "|/\\[]{}()<>".contains(left) {
                return Some(left);
            }
        }
        if (layout & SM_HIERARCHY) != 0 {
            for (a, b) in [
                ("|", "/\\[]{}()<>"),
                ("/\\", "[]{}()<>"),
                ("[]", "{}()<>"),
                ("{}", "()<>"),
                ("()", "<>"),
            ] {
                if a.contains(left) && b.contains(right) {
                    return Some(right);
                }
                if a.contains(right) && b.contains(left) {
                    return Some(left);
                }
            }
        }
        if (layout & SM_PAIR) != 0 {
            let pair = [left, right];
            let reversed = [right, left];
            if pair == ['[', ']']
                || pair == ['{', '}']
                || pair == ['(', ')']
                || reversed == ['[', ']']
                || reversed == ['{', '}']
                || reversed == ['(', ')']
            {
                return Some('|');
            }
        }
        if (layout & SM_BIGX) != 0 {
            if left == '/' && right == '\\' {
                return Some('|');
            }
            if left == '\\' && right == '/' {
                return Some('Y');
            }
            if left == '>' && right == '<' {
                return Some('X');
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture(path: &str) -> String {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        fs::read_to_string(root.join(path)).unwrap()
    }

    fn full_smush_font() -> FIGfont {
        let mut font = FIGfont::standard().unwrap();
        font.header_line.full_layout = Some(
            SM_EQUAL | SM_LOWLINE | SM_HIERARCHY | SM_PAIR | SM_BIGX | SM_HARDBLANK | SM_SMUSH,
        );
        font
    }

    #[test]
    fn test_new_headerline() {
        let line = "flf2a$ 6 5 20 15 3 0 143 229";
        let headerline = HeaderLine::try_from(line);
        assert!(headerline.is_ok());
        let headerline = headerline.unwrap();

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
        let font = FIGfont::standard();
        assert!(font.is_ok());
        let font = font.unwrap();

        let headerline = font.header_line;
        assert_eq!("flf2a$ 6 5 16 15 11 0 24463", headerline.header_line);
        assert_eq!("flf2a", headerline.signature);
        assert_eq!('$', headerline.hardblank);
        assert_eq!(6, headerline.height);
        assert_eq!(5, headerline.baseline);
        assert_eq!(16, headerline.max_length);
        assert_eq!(15, headerline.old_layout);
        assert_eq!(11, headerline.comment_lines);
        assert_eq!(Some(0), headerline.print_direction);
        assert_eq!(Some(24463), headerline.full_layout);
        assert_eq!(None, headerline.codetag_count);

        assert_eq!(
            "Standard by Glenn Chappell & Ian Chai 3/93 -- based on Frank's .sig
Includes ISO Latin-1
figlet release 2.1 -- 12 Aug 1994
Modified for figlet 2.2 by John Cowan <cowan@ccil.org>
  to add Latin-{2,3,4,5} support (Unicode U+0100-017F).
Permission is hereby given to modify this font, as long as the
modifier's name is placed on a comment line.

Modified by Paul Burton <solution@earthlink.net> 12/96 to include new parameter
supported by FIGlet and FIGWin.  May also be slightly modified for better use
of new full-width/kern/smush alternatives, but default output is NOT changed.",
            font.comments
        );

        let one_font = font.fonts.get(&('F' as u32));
        assert!(one_font.is_some());

        let one_font = one_font.unwrap();
        assert_eq!(70, one_font.code);
        assert_eq!(8, one_font.width);
        assert_eq!(6, one_font.height);

        assert_eq!(6, one_font.characters.len());
        assert_eq!("  _____ ", one_font.characters.first().unwrap());
        assert_eq!(" |  ___|", one_font.characters.get(1).unwrap());
        assert_eq!(" | |_   ", one_font.characters.get(2).unwrap());
        assert_eq!(" |  _|  ", one_font.characters.get(3).unwrap());
        assert_eq!(" |_|    ", one_font.characters.get(4).unwrap());
        assert_eq!("        ", one_font.characters.get(5).unwrap());
    }

    #[test]
    fn test_convert() {
        let standard_font = FIGfont::standard();
        assert!(standard_font.is_ok());
        let standard_font = standard_font.unwrap();

        let figure = standard_font.convert("FIGlet");
        assert!(figure.is_some());

        let figure = figure.unwrap();
        assert_eq!(6, figure.height);
        assert_eq!(6, figure.characters.len());

        let f = figure.characters.first().unwrap();
        assert_eq!(figure.height, f.height);
        assert_eq!(8, f.width);
        assert_eq!("  _____ ", f.characters.first().unwrap());
        assert_eq!(" |  ___|", f.characters.get(1).unwrap());
        assert_eq!(" | |_   ", f.characters.get(2).unwrap());
        assert_eq!(" |  _|  ", f.characters.get(3).unwrap());
        assert_eq!(" |_|    ", f.characters.get(4).unwrap());
        assert_eq!("        ", f.characters.get(5).unwrap());

        let i = figure.characters.get(1).unwrap();
        assert_eq!(figure.height, i.height);
        assert_eq!(6, i.width);
        assert_eq!("  ___ ", i.characters.first().unwrap());
        assert_eq!(" |_ _|", i.characters.get(1).unwrap());
        assert_eq!("  | | ", i.characters.get(2).unwrap());
        assert_eq!("  | | ", i.characters.get(3).unwrap());
        assert_eq!(" |___|", i.characters.get(4).unwrap());
        assert_eq!("      ", i.characters.get(5).unwrap());

        let g = figure.characters.get(2).unwrap();
        assert_eq!(figure.height, g.height);
        assert_eq!(8, g.width);
        assert_eq!(r"   ____ ", g.characters.first().unwrap());
        assert_eq!(r"  / ___|", g.characters.get(1).unwrap());
        assert_eq!(r" | |  _ ", g.characters.get(2).unwrap());
        assert_eq!(r" | |_| |", g.characters.get(3).unwrap());
        assert_eq!(r"  \____|", g.characters.get(4).unwrap());
        assert_eq!(r"        ", g.characters.get(5).unwrap());

        let l = figure.characters.get(3).unwrap();
        assert_eq!(figure.height, l.height);
        assert_eq!(4, l.width);
        assert_eq!("  _ ", l.characters.first().unwrap());
        assert_eq!(" | |", l.characters.get(1).unwrap());
        assert_eq!(" | |", l.characters.get(2).unwrap());
        assert_eq!(" | |", l.characters.get(3).unwrap());
        assert_eq!(" |_|", l.characters.get(4).unwrap());
        assert_eq!("    ", l.characters.get(5).unwrap());

        let e = figure.characters.get(4).unwrap();
        assert_eq!(figure.height, e.height);
        assert_eq!(7, e.width);
        assert_eq!(r"       ", e.characters.first().unwrap());
        assert_eq!(r"   ___ ", e.characters.get(1).unwrap());
        assert_eq!(r"  / _ \", e.characters.get(2).unwrap());
        assert_eq!(r" |  __/", e.characters.get(3).unwrap());
        assert_eq!(r"  \___|", e.characters.get(4).unwrap());
        assert_eq!(r"       ", e.characters.get(5).unwrap());

        let t = figure.characters.get(5).unwrap();
        assert_eq!(figure.height, t.height);
        assert_eq!(6, t.width);
        assert_eq!(r"  _   ", t.characters.first().unwrap());
        assert_eq!(r" | |_ ", t.characters.get(1).unwrap());
        assert_eq!(r" | __|", t.characters.get(2).unwrap());
        assert_eq!(r" | |_ ", t.characters.get(3).unwrap());
        assert_eq!(r"  \__|", t.characters.get(4).unwrap());
        assert_eq!(r"      ", t.characters.get(5).unwrap());
    }

    #[test]
    fn test_convert_empty_string() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("");
        assert!(figure.is_none());
    }

    #[test]
    fn test_convert_single_character() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("A");
        assert!(figure.is_some());

        let figure = figure.unwrap();
        assert_eq!(1, figure.characters.len());
        assert_eq!(6, figure.height);
    }

    #[test]
    fn test_convert_all_ascii_printable() {
        let font = FIGfont::standard().unwrap();
        // All printable ASCII characters (32-126)
        let all_ascii: String = (32..=126).map(|c| char::from_u32(c).unwrap()).collect();
        let figure = font.convert(&all_ascii);
        assert!(figure.is_some());

        let figure = figure.unwrap();
        // All 95 characters should be converted
        assert_eq!(95, figure.characters.len());
    }

    #[test]
    fn test_convert_with_unknown_characters() {
        let font = FIGfont::standard().unwrap();
        // Mix of known and unknown characters (Chinese characters are not in standard font)
        let figure = font.convert("Hello世界");
        assert!(figure.is_some());

        let figure = figure.unwrap();
        // Only "Hello" should be converted (5 characters)
        assert_eq!(5, figure.characters.len());
    }

    #[test]
    fn test_figure_as_str() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("Hi").unwrap();
        let s = figure.as_str();
        assert!(!s.is_empty());
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(figure.height as usize, lines.len());
    }

    #[test]
    fn test_figure_display() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("AB").unwrap();
        let display_output = format!("{}", figure);
        let debug_output = format!("{:?}", figure);

        assert!(!display_output.is_empty());
        assert!(display_output.contains('\n'));
        // Debug output should work without panicking
        assert!(!debug_output.is_empty());
    }

    #[test]
    fn test_standard_golden_test() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("Test").unwrap();
        assert_eq!(fixture("tests/fixtures/standard_test.txt"), figure.as_str());
    }

    #[test]
    fn test_standard_golden_figlet() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("FIGlet").unwrap();
        assert_eq!(
            fixture("tests/fixtures/standard_figlet.txt"),
            figure.as_str()
        );
    }

    #[test]
    fn test_standard_golden_negative_float() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("-4.5").unwrap();
        assert_eq!(
            fixture("tests/fixtures/standard_negative_float.txt"),
            figure.as_str()
        );
    }

    #[test]
    fn test_small_golden_test() {
        let font = FIGfont::from_file("resources/small.flf").unwrap();
        let figure = font.convert("Test").unwrap();
        assert_eq!(fixture("tests/fixtures/small_test.txt"), figure.as_str());
    }

    #[test]
    fn test_small_golden_negative_float() {
        let font = FIGfont::from_file("resources/small.flf").unwrap();
        let figure = font.convert("-4.5").unwrap();
        assert_eq!(
            fixture("tests/fixtures/small_negative_float.txt"),
            figure.as_str()
        );
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
        let renderer = Renderer {
            font: &font,
            prev_char_width: 2,
            cur_char_width: 2,
            max_smush: 0,
        };
        assert_eq!(Some('|'), renderer.smush_chars('|', '|'));
    }

    #[test]
    fn test_smush_rule_lowline_and_hierarchy() {
        let font = full_smush_font();
        let renderer = Renderer {
            font: &font,
            prev_char_width: 2,
            cur_char_width: 2,
            max_smush: 0,
        };
        assert_eq!(Some('/'), renderer.smush_chars('_', '/'));
        assert_eq!(Some('>'), renderer.smush_chars('|', '>'));
    }

    #[test]
    fn test_smush_rule_pair_and_bigx() {
        let font = full_smush_font();
        let renderer = Renderer {
            font: &font,
            prev_char_width: 2,
            cur_char_width: 2,
            max_smush: 0,
        };
        assert_eq!(Some('|'), renderer.smush_chars('[', ']'));
        assert_eq!(Some('|'), renderer.smush_chars('/', '\\'));
        assert_eq!(Some('Y'), renderer.smush_chars('\\', '/'));
        assert_eq!(Some('X'), renderer.smush_chars('>', '<'));
    }

    #[test]
    fn test_smush_rule_hardblank() {
        let font = full_smush_font();
        let renderer = Renderer {
            font: &font,
            prev_char_width: 2,
            cur_char_width: 2,
            max_smush: 0,
        };
        let hb = font.header_line.hardblank;
        assert_eq!(Some(hb), renderer.smush_chars(hb, hb));
    }

    #[test]
    fn test_figure_figure_is_not_empty() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("Test").unwrap();
        assert!(figure.is_not_empty());
    }

    #[test]
    fn test_figure_with_only_unknown_chars() {
        let font = FIGfont::standard().unwrap();
        // Chinese characters are not in standard font
        let figure = font.convert("\u{4E2D}\u{6587}");
        // Should return None because no characters were found
        assert!(figure.is_none());
    }

    #[test]
    fn test_figfont_clone() {
        let font1 = FIGfont::standard().unwrap();
        let font2 = font1.clone();

        // Both should work independently
        let figure1 = font1.convert("Test");
        let figure2 = font2.convert("Test");

        assert!(figure1.is_some());
        assert!(figure2.is_some());
    }

    #[test]
    fn test_standard_font_loading() {
        let font1 = FIGfont::standard().unwrap();
        let font2 = FIGfont::standard().unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_small_font_loading() {
        let font1 = FIGfont::small().unwrap();
        let font2 = FIGfont::from_file("resources/small.flf").unwrap();

        assert_eq!(font1.header_line.header_line, font2.header_line.header_line);
        assert_eq!(font1.comments, font2.comments);
    }

    #[test]
    fn test_figure_character_width_and_height() {
        let font = FIGfont::standard().unwrap();
        let figure = font.convert("ABC").unwrap();

        for char in &figure.characters {
            assert_eq!(figure.height, char.height);
            assert!(char.width > 0);
        }
    }

    #[test]
    fn test_latin_characters() {
        let font = FIGfont::standard().unwrap();
        // German characters that are in the standard font
        let figure = font.convert("\u{00C4}\u{00D6}\u{00DC}"); // ÄÖÜ
        assert!(figure.is_some());

        let figure = figure.unwrap();
        assert_eq!(3, figure.characters.len());
    }

    #[test]
    fn test_from_content_invalid() {
        let result = FIGfont::from_content("");
        assert!(result.is_err());

        let result = FIGfont::from_content("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_headerline_invalid() {
        // Too few fields
        let result = HeaderLine::try_from("flf2a$ 6");
        assert!(result.is_err());
    }
}
