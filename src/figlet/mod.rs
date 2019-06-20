//! please visit [figfont](http://www.jave.de/figlet/figfont.html) to find detail.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::u32;

pub struct FIGfont {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

impl FIGfont {
    fn read_font_file(filename: &str) -> String {
        let contents = match fs::read_to_string(filename) {
            Ok(contents) => contents,
            Err(err) => panic!("err to read font content: {}", err),
        };

        contents
    }

    fn read_header_line(lines: &Vec<&str>) -> HeaderLine {
        let headerline = lines.get(0).unwrap_or_else(|| {
            panic!("can't read headerline from fontfile");
        });

        HeaderLine::new(headerline)
    }

    fn read_comments(lines: &Vec<&str>, comment_count: i32) -> String {
        let length = lines.len() as i32;
        if length < comment_count + 1 {
            panic!("can't get comments from font");
        };

        lines[1..(1 + comment_count) as usize]
            .join("\n")
            .to_string()
    }

    fn extract_one_line(
        lines: &Vec<&str>,
        index: usize,
        height: usize,
        is_last_index: bool,
    ) -> String {
        let line = lines.get(index).unwrap_or_else(|| {
            panic!("can't get line at specified index:{}", index);
        });

        if line.len() <= 2 {
            panic!("one line len can't be less than 2. it is:{}", line);
        }

        let mut width = line.len() - 1;
        if is_last_index && height != 1 {
            width -= 1;
        }

        String::from(&line[..width])
    }

    fn extract_one_font(
        lines: &Vec<&str>,
        code: u32,
        start_index: usize,
        height: usize,
    ) -> FIGcharacter {
        let mut characters = vec![];
        for i in 0..height {
            let index = start_index + i as usize;
            let is_last_index = i == height - 1;
            let one_line_character = FIGfont::extract_one_line(lines, index, height, is_last_index);
            characters.push(one_line_character);
        }
        let width = characters[0].len() as u32;
        let height = height as u32;

        FIGcharacter {
            code,
            characters,
            width,
            height,
        }
    }

    // 32-126, 196, 214, 220, 228, 246, 252, 223
    fn read_required_font(
        lines: &Vec<&str>,
        headerline: &HeaderLine,
        map: &mut HashMap<u32, FIGcharacter>,
    ) {
        let offset = (1 + headerline.comment_lines) as usize;
        let height = headerline.height as usize;
        let size = lines.len();

        for i in 0..=94 {
            let code = (i + 32) as u32;
            let start_index = offset + i * height;
            if start_index >= size {
                break;
            }

            let font = FIGfont::extract_one_font(lines, code, start_index, height);
            map.insert(code, font);
        }

        let offset = offset + 95 * height;
        let required_deutsch_characters_codes: [u32; 7] = [196, 214, 220, 228, 246, 252, 223];
        for i in 0..=6 {
            let code = required_deutsch_characters_codes[i];
            // let start_index = (offset + (95 + i) * height) as usize;
            let start_index = offset + i * height;
            if start_index >= size {
                break;
            }

            let font = FIGfont::extract_one_font(lines, code, start_index, height);
            map.insert(code, font);
        }
    }

    fn extract_codetag_font_code(lines: &Vec<&str>, index: usize) -> u32 {
        let line = match lines.get(index) {
            Some(line) => line,
            None => panic!("get codetag line error"),
        };

        let infos: Vec<&str> = line.trim().split(" ").collect();
        if infos.len() < 1 {
            panic!("extract code for codetag font error");
        }

        let code = infos[0].trim();

        let parse = if code.starts_with("0x") || code.starts_with("0X") {
            u32::from_str_radix(&code[2..], 16)
        } else if code.starts_with("0") {
            u32::from_str_radix(&code[1..], 8)
        } else {
            code.parse()
        };

        let code: u32 = match parse {
            Ok(code) => code,
            Err(_) => panic!("parse code for codetag font error"),
        };

        code
    }

    fn read_codetag_font(
        lines: &Vec<&str>,
        headerline: &HeaderLine,
        map: &mut HashMap<u32, FIGcharacter>,
    ) {
        let offset = (1 + headerline.comment_lines + 102 * headerline.height) as usize;
        let codetag_height = (headerline.height + 1) as usize;
        let codetag_lines = lines.len() - offset;

        if codetag_lines % codetag_height != 0 {
            panic!("codetag font is illegal.")
        }

        let size = codetag_lines / codetag_height;

        for i in 0..size {
            let start_index = offset + i * codetag_height;
            if start_index >= lines.len() {
                break;
            }

            let code = FIGfont::extract_codetag_font_code(lines, start_index);
            let font =
                FIGfont::extract_one_font(lines, code, start_index + 1, headerline.height as usize);
            map.insert(code, font);
        }
    }

    fn read_fonts(lines: &Vec<&str>, headerline: &HeaderLine) -> HashMap<u32, FIGcharacter> {
        let mut map = HashMap::new();
        FIGfont::read_required_font(lines, headerline, &mut map);
        FIGfont::read_codetag_font(lines, headerline, &mut map);
        map
    }
}

impl FIGfont {
    pub fn from_content(contents: &str) -> FIGfont {
        let lines: Vec<&str> = contents.lines().collect();

        let header_line = FIGfont::read_header_line(&lines);
        let comments = FIGfont::read_comments(&lines, header_line.comment_lines);
        let fonts = FIGfont::read_fonts(&lines, &header_line);

        FIGfont {
            header_line,
            comments,
            fonts,
        }
    }

    pub fn from_file(fontname: &str) -> FIGfont {
        let contents = FIGfont::read_font_file(fontname);
        FIGfont::from_content(&contents)
    }

    pub fn standand() -> FIGfont {
        let fontname = "resources/standard.flf";
        let contents = FIGfont::read_font_file(fontname);
        FIGfont::from_content(&contents)
    }

    pub fn convert(&self, message: &str) -> Option<FIGure> {
        if message.len() == 0 {
            return None;
        }

        let mut characters: Vec<&FIGcharacter> = vec![];
        for ch in message.chars() {
            let code = ch as u32;
            if let Some(character) = self.fonts.get(&code) {
                characters.push(character);
            }
        }
        Some(FIGure {
            characters,
            height: self.header_line.height as u32,
        })
    }
}

#[derive(Debug)]
pub struct HeaderLine {
    pub header_line: String,

    // required
    pub signature: String,
    pub hardblank: String,
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
    fn extract_signature_with_hardblank(infos: &Vec<&str>) -> (String, String) {
        let signature_with_hardblank = infos.get(0).unwrap_or_else(|| {
            panic!("can't get signature with hardblank from first line of font");
        });

        if signature_with_hardblank.len() < 6 {
            panic!("can't get signature with hardblank from first line of font");
        }
        let hardblank_index = &signature_with_hardblank.len() - 1;
        let signature = &signature_with_hardblank[..hardblank_index];
        let hardblank = &signature_with_hardblank[hardblank_index..];

        (String::from(signature), String::from(hardblank))
    }

    fn extract_required_info(infos: &Vec<&str>, index: usize, field: &str) -> i32 {
        let val = infos.get(index).unwrap_or_else(|| {
            panic!(
                "can't get field:{} index:{} from first line of font",
                field, index
            )
        });

        let num: i32 = match val.parse() {
            Ok(num) => num,
            Err(_) => panic!("can't parse required field:{} of {} to i32", field, val),
        };

        num
    }

    fn extract_optional_info(infos: &Vec<&str>, index: usize, field: &str) -> Option<i32> {
        if let Some(val) = infos.get(index) {
            let num: i32 = match val.parse() {
                Ok(num) => num,
                Err(_) => panic!("can't parse optional field:{} of {} to i32", field, val),
            };
            return Some(num);
        }

        None
    }

    pub fn new(header_line: &str) -> HeaderLine {
        let infos: Vec<&str> = header_line.trim().split(" ").collect();

        let (signature, hardblank) = HeaderLine::extract_signature_with_hardblank(&infos);

        let height = HeaderLine::extract_required_info(&infos, 1, "height");
        let baseline = HeaderLine::extract_required_info(&infos, 2, "baseline");
        let max_length = HeaderLine::extract_required_info(&infos, 3, "max length");
        let old_layout = HeaderLine::extract_required_info(&infos, 4, "old layout");
        let comment_lines = HeaderLine::extract_required_info(&infos, 5, "comment lines");

        let print_direction = HeaderLine::extract_optional_info(&infos, 6, "print direction");
        let full_layout = HeaderLine::extract_optional_info(&infos, 7, "full layout");
        let codetag_count = HeaderLine::extract_optional_info(&infos, 8, "codetag count");

        HeaderLine {
            header_line: String::from(header_line),
            signature: String::from(signature),
            hardblank: String::from(hardblank),
            height,
            baseline,
            max_length,
            old_layout,
            comment_lines,
            print_direction,
            full_layout,
            codetag_count,
        }
    }
}

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

pub struct FIGure<'a> {
    pub characters: Vec<&'a FIGcharacter>,
    pub height: u32,
}

impl<'a> FIGure<'a> {
    fn is_not_empty(&self) -> bool {
        self.characters.len() > 0 && self.height > 0
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
            write!(f, "{}", "")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_header_line() {
        let line = "flf2a$ 6 5 20 15 3 0 143 229";
        let headerline = HeaderLine::new(line);
        assert_eq!(line, headerline.header_line);
        assert_eq!("flf2a", headerline.signature);
        assert_eq!("$", headerline.hardblank);
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
    fn new_fig_font() {
        let font = FIGfont::from_file("resources/standard.flf");

        let headerline = font.header_line;
        assert_eq!("flf2a$ 6 5 16 15 11 0 24463", headerline.header_line);
        assert_eq!("flf2a", headerline.signature);
        assert_eq!("$", headerline.hardblank);
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

        let one_font = font.fonts.get(&70); // F
        assert!(one_font.is_some());

        let one_font = one_font.unwrap();
        assert_eq!(70, one_font.code);
        assert_eq!(8, one_font.width);

        assert_eq!(6, one_font.characters.len());
        assert_eq!("  _____ ", one_font.characters.get(0).unwrap());
        assert_eq!(" |  ___|", one_font.characters.get(1).unwrap());
        assert_eq!(" | |_   ", one_font.characters.get(2).unwrap());
        assert_eq!(" |  _|  ", one_font.characters.get(3).unwrap());
        assert_eq!(" |_|    ", one_font.characters.get(4).unwrap());
        assert_eq!("        ", one_font.characters.get(5).unwrap());
    }

    #[test]
    fn convert_message() {
        let standard_font = FIGfont::standand();
        let figure = standard_font.convert("hello");
        assert!(figure.is_some());
    }
}
