use std::collections::HashMap;

/// http://www.jave.de/figlet/figfont.html
pub struct FIGfont<'a> {
    pub header_line: HeaderLine<'a>,
    pub comments: String,
    pub data: HashMap<i32, FIGcharacter>,
}

// impl FIGfont {
//     pub fn new() -> Result<FIGfont, &'static str> {
//         Err("")
//     }

//     pub fn convert(&self, message: &str) -> Result<FIGure, &'static str> {
//         Err("")
//     }
// }

#[derive(Debug)]
pub struct HeaderLine<'a> {
    pub header_line: &'a str,

    pub signature: &'a str,
    pub hardblank: &'a str,
    pub height: u32,
    pub baseline: u32,
    pub max_length: u32,
    pub old_layout: i8, // Legal values -1 to 63
    pub comment_lines: u32,
    pub print_direction: Option<u8>,
    pub full_layout: Option<u16>, // Legal values 0 to 32767
    pub codetag_count: Option<u32>,
}

impl<'a> HeaderLine<'a> {
    pub fn new(header_line: &str) -> Result<HeaderLine, &str> {
        let infos: Vec<&str> = header_line.trim().split(" ").collect();
        let mut iter = infos.iter();

        // 1st: signature with hardblank
        let signature_with_hardblank = iter
            .next()
            .expect("headerline: signature_with_hardblank not found!");
        if signature_with_hardblank.len() < 6 {
            return Err("signature_with_hardblank length is illegal, at least 6.");
        }
        let hardblank_index = &signature_with_hardblank.len() - 1;
        let signature = &signature_with_hardblank[..hardblank_index];
        let hardblank = &signature_with_hardblank[hardblank_index..];

        // 2nd: height
        let height: u32 = iter
            .next()
            .expect("headerline: height not found!")
            .parse()
            .expect("headerline: height is illegal!");
        // 3rd: baseline
        let baseline: u32 = iter
            .next()
            .expect("headerline: baseline not found!")
            .parse()
            .expect("headerline: baseline is illegal!");
        // 4th: max length
        let max_length: u32 = iter
            .next()
            .expect("headerline: max length not found!")
            .parse()
            .expect("headerline: max length is illegal!");
        // 5th: old layout
        let old_layout: i8 = iter
            .next()
            .expect("headerline: old layout not found!")
            .parse()
            .expect("headerline: old layout is illegal!");
        // 6th: comment lines
        let comment_lines: u32 = iter
            .next()
            .expect("headerline: comment lines not found!")
            .parse()
            .expect("headerline: comment lines is illegal!");

        let mut headerline = HeaderLine {
            header_line,
            signature,
            hardblank,
            height,
            baseline,
            max_length,
            old_layout,
            comment_lines,
            print_direction: None,
            full_layout: None,
            codetag_count: None,
        };

        // 7th: print direction
        if let Some(&print_direction) = iter.next() {
            let direction: u8 = print_direction
                .parse()
                .expect("headerline: print direction is illegal!");
            headerline.print_direction = Some(direction);
        }

        // 8th: full layout
        if let Some(&full_layout) = iter.next() {
            let full_layout: u16 = full_layout
                .parse()
                .expect("headerline: full layout is illegal!");
            headerline.full_layout = Some(full_layout);
        }

        // 9th: codetag count
        if let Some(&codetag_count) = iter.next() {
            let codetag_count: u32 = codetag_count
                .parse()
                .expect("headerline: codetag count is illegal!");
            headerline.codetag_count = Some(codetag_count);
        }

        Ok(headerline)
    }
}

pub struct FIGcharacter {
    pub code: i32,
    pub characters: Vec<String>,
    pub width: u32,
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
        let headerline = HeaderLine::new(line).unwrap();
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
}
