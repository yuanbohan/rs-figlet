//! you can visit [`figlet`] and [`figfont`] to find more details.
//! you can visit [`fongdb`] to find more font.
//!
//! # Examples
//!
//! download [`small.flf`] and place it to the `resources` folder.
//!
//! convert string literal using standard or specified font:
//!
//! ```
//! use figlet_rs::FIGfont;
//!
//! let standard_font = FIGfont::standand().unwrap();
//! let figure = standard_font.convert("FIGlet");
//! assert!(figure.is_some());
//!
//! let small_font = FIGfont::from_file("resources/small.flf").unwrap();
//! let figure = small_font.convert("FIGlet");
//! assert!(figure.is_some());
//! ```
//! [`figlet`]: http://www.figlet.org
//! [`figfont`]: http://www.jave.de/figlet/figfont.html
//! [`fongdb`]: http://www.figlet.org/fontdb.cgi
//! [`small.flf`]: http://www.figlet.org/fonts/small.flf

use std::collections::HashMap;
use std::{fmt, fs};

/// FIGlet font, which will hold the mapping from u32 code to FIGcharacter
pub struct FIGfont {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

impl FIGfont {
    fn read_font_file(filename: &str) -> Result<String, String> {
        fs::read_to_string(filename).map_err(|e| format!("{:?}", e))
    }

    fn read_header_line(header_line: &str) -> Result<HeaderLine, String> {
        HeaderLine::new(header_line)
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
        hardblank: char,
        is_last_index: bool,
    ) -> Result<String, String> {
        let line = lines
            .get(index)
            .ok_or(format!("can't get line at specified index:{}", index))?;

        if line.len() <= 2 {
            return Err(format!("one line len can't be less than 2. it is:{}", line));
        }

        let mut width = line.len() - 1;
        if is_last_index && height != 1 {
            width -= 1;
        }

        Ok(line[..width].replace(hardblank, " ").to_string())
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
            let index = start_index + i as usize;
            let is_last_index = i == height - 1;
            let one_line_character =
                FIGfont::extract_one_line(lines, index, height, hardblank, is_last_index)?;
            characters.push(one_line_character);
        }
        let width = characters[0].len() as u32;
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

        let code = if code.starts_with("0x") || code.starts_with("0X") {
            u32::from_str_radix(&code[2..], 16)
        } else if code.starts_with('0') {
            u32::from_str_radix(&code[1..], 8)
        } else {
            code.parse()
        };
        code.map_err(|e| format!("{:?}", e))
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

        let header_line = FIGfont::read_header_line(&lines.get(0).unwrap())?;
        let comments = FIGfont::read_comments(&lines, header_line.comment_lines)?;
        let fonts = FIGfont::read_fonts(&lines, &header_line)?;

        Ok(FIGfont {
            header_line,
            comments,
            fonts,
        })
    }

    /// generate FIGlet font from specified file
    ///
    /// All fonts found at [`fontdb`] get shipped with the same name
    ///
    /// [`fontdb`]: http://www.figlet.org/fontdb.cgi
    pub fn from_file(fontname: &str) -> Result<FIGfont, String> {
        let contents = FIGfont::read_font_file(fontname)?;
        FIGfont::from_content(&contents)
    }

    /// the standard FIGlet font, which you can find [`fontdb`]
    ///
    /// [`fontdb`]: http://www.figlet.org/fontdb.cgi
    pub fn standand() -> Result<FIGfont, String> {
        let contents = std::include_str!("./fonts/standard.flf");
        FIGfont::from_content(contents)
    }

    /// convert string literal to FIGure
    pub fn convert(&self, message: &str) -> Option<FIGure> {
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

        Some(FIGure {
            characters,
            height: self.header_line.height as u32,
        })
    }
}

/// the first line in FIGlet font, which you can find the documentation [`headerline`]
///
/// [`headerline`]: http://www.jave.de/figlet/figfont.html#headerline
#[derive(Debug)]
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
                "can't get field:{} index:{} from {}",
                field,
                index,
                infos.join(",")
            )),
        }?;

        val.parse::<i32>()
            .map_err(|_e| format!("can't parse required field:{} of {} to i32", field, val))
    }

    fn extract_optional_info(infos: &[&str], index: usize, _field: &str) -> Option<i32> {
        if let Some(val) = infos.get(index) {
            val.parse().ok()
        } else {
            None
        }
    }

    pub fn new(header_line: &str) -> Result<HeaderLine, String> {
        let infos: Vec<&str> = header_line.trim().split(' ').collect();

        if infos.len() < 6 {
            return Err("headerline is illegal".to_string());
        }

        let signature_with_hardblank =
            HeaderLine::extract_signature_with_hardblank(&infos.get(0).unwrap())?;

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
#[derive(Debug)]
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
pub struct FIGure<'a> {
    pub characters: Vec<&'a FIGcharacter>,
    pub height: u32,
}

impl<'a> FIGure<'a> {
    fn is_not_empty(&self) -> bool {
        !self.characters.is_empty() && self.height > 0
    }
}

impl<'a> fmt::Display for FIGure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_not_empty() {
            let mut rs: Vec<&'a str> = vec![];
            for i in 0..self.height {
                for character in &self.characters {
                    if let Some(line) = character.characters.get(i as usize) {
                        rs.push(line);
                    }
                }
                rs.push("\n");
            }

            write!(f, "{}", rs.join(""))
        } else {
            write!(f, "")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_headerline() {
        let line = "flf2a$ 6 5 20 15 3 0 143 229";
        let headerline = HeaderLine::new(line);
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
        let font = FIGfont::standand();
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
        assert_eq!("  _____ ", one_font.characters.get(0).unwrap());
        assert_eq!(" |  ___|", one_font.characters.get(1).unwrap());
        assert_eq!(" | |_   ", one_font.characters.get(2).unwrap());
        assert_eq!(" |  _|  ", one_font.characters.get(3).unwrap());
        assert_eq!(" |_|    ", one_font.characters.get(4).unwrap());
        assert_eq!("        ", one_font.characters.get(5).unwrap());
    }

    #[test]
    fn test_convert() {
        let standard_font = FIGfont::standand();
        assert!(standard_font.is_ok());
        let standard_font = standard_font.unwrap();

        let figure = standard_font.convert("FIGlet");
        assert!(figure.is_some());

        let figure = figure.unwrap();
        assert_eq!(6, figure.height);
        assert_eq!(6, figure.characters.len());

        let f = figure.characters.get(0).unwrap();
        assert_eq!(figure.height, f.height);
        assert_eq!(8, f.width);
        assert_eq!("  _____ ", f.characters.get(0).unwrap());
        assert_eq!(" |  ___|", f.characters.get(1).unwrap());
        assert_eq!(" | |_   ", f.characters.get(2).unwrap());
        assert_eq!(" |  _|  ", f.characters.get(3).unwrap());
        assert_eq!(" |_|    ", f.characters.get(4).unwrap());
        assert_eq!("        ", f.characters.get(5).unwrap());

        let i = figure.characters.get(1).unwrap();
        assert_eq!(figure.height, i.height);
        assert_eq!(6, i.width);
        assert_eq!("  ___ ", i.characters.get(0).unwrap());
        assert_eq!(" |_ _|", i.characters.get(1).unwrap());
        assert_eq!("  | | ", i.characters.get(2).unwrap());
        assert_eq!("  | | ", i.characters.get(3).unwrap());
        assert_eq!(" |___|", i.characters.get(4).unwrap());
        assert_eq!("      ", i.characters.get(5).unwrap());

        let g = figure.characters.get(2).unwrap();
        assert_eq!(figure.height, g.height);
        assert_eq!(8, g.width);
        assert_eq!(r"   ____ ", g.characters.get(0).unwrap());
        assert_eq!(r"  / ___|", g.characters.get(1).unwrap());
        assert_eq!(r" | |  _ ", g.characters.get(2).unwrap());
        assert_eq!(r" | |_| |", g.characters.get(3).unwrap());
        assert_eq!(r"  \____|", g.characters.get(4).unwrap());
        assert_eq!(r"        ", g.characters.get(5).unwrap());

        let l = figure.characters.get(3).unwrap();
        assert_eq!(figure.height, l.height);
        assert_eq!(4, l.width);
        assert_eq!("  _ ", l.characters.get(0).unwrap());
        assert_eq!(" | |", l.characters.get(1).unwrap());
        assert_eq!(" | |", l.characters.get(2).unwrap());
        assert_eq!(" | |", l.characters.get(3).unwrap());
        assert_eq!(" |_|", l.characters.get(4).unwrap());
        assert_eq!("    ", l.characters.get(5).unwrap());

        let e = figure.characters.get(4).unwrap();
        assert_eq!(figure.height, e.height);
        assert_eq!(7, e.width);
        assert_eq!(r"       ", e.characters.get(0).unwrap());
        assert_eq!(r"   ___ ", e.characters.get(1).unwrap());
        assert_eq!(r"  / _ \", e.characters.get(2).unwrap());
        assert_eq!(r" |  __/", e.characters.get(3).unwrap());
        assert_eq!(r"  \___|", e.characters.get(4).unwrap());
        assert_eq!(r"       ", e.characters.get(5).unwrap());

        let t = figure.characters.get(5).unwrap();
        assert_eq!(figure.height, t.height);
        assert_eq!(6, t.width);
        assert_eq!(r"  _   ", t.characters.get(0).unwrap());
        assert_eq!(r" | |_ ", t.characters.get(1).unwrap());
        assert_eq!(r" | __|", t.characters.get(2).unwrap());
        assert_eq!(r" | |_ ", t.characters.get(3).unwrap());
        assert_eq!(r"  \__|", t.characters.get(4).unwrap());
        assert_eq!(r"      ", t.characters.get(5).unwrap());
    }
}
