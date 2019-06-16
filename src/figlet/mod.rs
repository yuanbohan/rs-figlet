//! please visit [figfont](http://www.jave.de/figlet/figfont.html) to find detail.

use std::collections::HashMap;
use std::fs;
use std::process;

pub struct FIGfont {
    pub header_line: HeaderLine,
    pub comments: String,
    pub data: HashMap<u32, FIGcharacter>,
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

    fn read_data(lines: &Vec<&str>, headerline: &HeaderLine) -> HashMap<u32, FIGcharacter> {
        let mut map = HashMap::new();
        let character = FIGcharacter {
            code: 1,
            characters: vec![String::from("1")],
            width: 1,
        };
        map.insert(1, character);

        map
    }
}

impl FIGfont {
    pub fn from_content(contents: &str) -> FIGfont {
        let lines: Vec<&str> = contents.lines().collect();

        let header_line = FIGfont::read_header_line(&lines);
        let comments = FIGfont::read_comments(&lines, header_line.comment_lines);
        let data = FIGfont::read_data(&lines, &header_line);

        FIGfont {
            header_line,
            comments,
            data,
        }
    }

    pub fn from_file(fontname: &str) -> FIGfont {
        let contents = FIGfont::read_font_file(fontname);
        FIGfont::from_content(&contents)
    }

    pub fn convert(&self, _message: &str) -> Result<FIGure, &'static str> {
        Err("")
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
    pub code: i32,
    pub characters: Vec<String>,
    pub width: i32,
}

impl FIGcharacter {}

pub struct FIGure {
    pub characters: Vec<FIGcharacter>,
}

impl FIGure {}

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
        println!("header_line: {:?}", font.header_line);
        println!("comments: {}", font.comments);
        println!("one data: {:?}", font.data.get(&1));
    }
}
