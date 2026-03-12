use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{Cursor, Read};

pub(crate) const SM_EQUAL: i32 = 1;
pub(crate) const SM_LOWLINE: i32 = 2;
pub(crate) const SM_HIERARCHY: i32 = 4;
pub(crate) const SM_PAIR: i32 = 8;
pub(crate) const SM_BIGX: i32 = 16;
pub(crate) const SM_HARDBLANK: i32 = 32;
pub(crate) const SM_KERN: i32 = 64;
pub(crate) const SM_SMUSH: i32 = 128;

pub(crate) struct FontData {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

pub(crate) fn load_font_file(filename: &str) -> Result<FontData, String> {
    let bytes = fs::read(filename).map_err(|e| format!("{e:?}"))?;
    parse_font_bytes(&bytes)
}

pub(crate) fn parse_font_bytes(bytes: &[u8]) -> Result<FontData, String> {
    let contents = decode_font_bytes(bytes)?;
    parse_font_content(&contents)
}

pub(crate) fn parse_font_content(contents: &str) -> Result<FontData, String> {
    let lines: Vec<&str> = contents.lines().collect();

    if lines.is_empty() {
        return Err("can not generate FIGlet font from empty string".to_string());
    }

    let header_line = read_header_line(lines.first().unwrap())?;
    let comments = read_comments(&lines, header_line.comment_lines)?;
    let fonts = read_fonts(&lines, &header_line)?;

    Ok(FontData {
        header_line,
        comments,
        fonts,
    })
}

fn decode_font_bytes(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(b"PK\x03\x04") {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("{e:?}"))?;
        if archive.is_empty() {
            return Err("zip font archive is empty".to_string());
        }

        let mut file = archive.by_index(0).map_err(|e| format!("{e:?}"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("{e:?}"))?;
        Ok(contents)
    } else {
        String::from_utf8(bytes.to_vec()).map_err(|e| format!("{e:?}"))
    }
}

fn read_header_line(header_line: &str) -> Result<HeaderLine, String> {
    HeaderLine::try_from(header_line)
}

fn read_comments(lines: &[&str], comment_count: i32) -> Result<String, String> {
    let length = lines.len() as i32;
    if length < comment_count + 1 {
        Err("can't get comments from font".to_string())
    } else {
        Ok(lines[1..(1 + comment_count) as usize].join("\n"))
    }
}

fn extract_one_line(
    lines: &[&str],
    index: usize,
    height: usize,
    is_last_index: bool,
) -> Result<String, String> {
    let line = lines
        .get(index)
        .ok_or_else(|| format!("can't get line at specified index:{index}"))?;

    let trimmed = line.trim_end_matches(' ');
    let mut chars: Vec<char> = trimmed.chars().collect();
    let endmark = chars
        .pop()
        .ok_or_else(|| format!("can't parse endmark at specified index:{index}"))?;
    if is_last_index && height != 1 && chars.last().copied() == Some(endmark) {
        chars.pop();
    }

    Ok(chars.into_iter().collect())
}

fn extract_one_font(
    lines: &[&str],
    code: u32,
    start_index: usize,
    height: usize,
) -> Result<FIGcharacter, String> {
    let mut characters = vec![];
    for i in 0..height {
        let index = start_index + i;
        let is_last_index = i == height - 1;
        characters.push(extract_one_line(lines, index, height, is_last_index)?);
    }

    Ok(FIGcharacter {
        code,
        width: characters[0].chars().count() as u32,
        height: height as u32,
        characters,
    })
}

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

        let font = extract_one_font(lines, code, start_index, height)?;
        map.insert(code, font);
    }

    let offset = offset + 95 * height;
    let required_deutsch_characters_codes: [u32; 7] = [196, 214, 220, 228, 246, 252, 223];
    for (i, code) in required_deutsch_characters_codes.iter().enumerate() {
        let start_index = offset + i * height;
        if start_index >= size {
            break;
        }

        let font = extract_one_font(lines, *code, start_index, height)?;
        map.insert(*code, font);
    }

    Ok(())
}

fn extract_codetag_font_code(lines: &[&str], index: usize) -> Result<Option<u32>, String> {
    let line = lines
        .get(index)
        .ok_or_else(|| "get codetag line error".to_string())?;

    let infos: Vec<&str> = line.split_whitespace().collect();
    if infos.is_empty() {
        return Err("extract code for codetag font error".to_string());
    }

    let code = infos[0].trim();
    let is_negative = code.starts_with('-');
    let unsigned = code.trim_start_matches(['-', '+']);

    let parsed = if let Some(s) = unsigned.strip_prefix("0x") {
        i64::from_str_radix(s, 16)
    } else if let Some(s) = unsigned.strip_prefix("0X") {
        i64::from_str_radix(s, 16)
    } else if unsigned.len() > 1 && unsigned.starts_with('0') {
        i64::from_str_radix(&unsigned[1..], 8)
    } else {
        unsigned.parse()
    }
    .map_err(|e| format!("{e:?}"))?;

    if is_negative {
        Ok(None)
    } else {
        u32::try_from(parsed)
            .map(Some)
            .map_err(|e| format!("{e:?}"))
    }
}

fn read_codetag_font(
    lines: &[&str],
    headerline: &HeaderLine,
    map: &mut HashMap<u32, FIGcharacter>,
) -> Result<(), String> {
    let offset = (1 + headerline.comment_lines + 102 * headerline.height) as usize;
    if offset >= lines.len() {
        return Ok(());
    }

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

        let Some(code) = extract_codetag_font_code(lines, start_index)? else {
            continue;
        };
        let font = extract_one_font(lines, code, start_index + 1, headerline.height as usize)?;
        map.insert(code, font);
    }

    Ok(())
}

fn read_fonts(
    lines: &[&str],
    headerline: &HeaderLine,
) -> Result<HashMap<u32, FIGcharacter>, String> {
    let mut map = HashMap::new();
    read_required_font(lines, headerline, &mut map)?;
    read_codetag_font(lines, headerline, &mut map)?;
    Ok(map)
}

pub(crate) fn render<'a>(
    header_line: &'a HeaderLine,
    fonts: &'a HashMap<u32, FIGcharacter>,
    message: &str,
) -> Option<FIGure<'a>> {
    if message.is_empty() {
        return None;
    }

    let mut characters: Vec<&FIGcharacter> = vec![];
    for ch in message.chars() {
        let code = ch as u32;
        if let Some(character) = fonts.get(&code) {
            characters.push(character);
        }
    }

    if characters.is_empty() {
        return None;
    }

    let rendered_lines = Renderer::new(header_line, fonts).render(&characters);

    Some(FIGure {
        characters,
        height: header_line.height as u32,
        lines: rendered_lines,
    })
}

#[derive(Debug, Clone)]
pub struct HeaderLine {
    pub header_line: String,
    pub signature: String,
    pub hardblank: char,
    pub height: i32,
    pub baseline: i32,
    pub max_length: i32,
    pub old_layout: i32,
    pub comment_lines: i32,
    pub print_direction: Option<i32>,
    pub full_layout: Option<i32>,
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
        let val = infos.get(index).ok_or_else(|| {
            format!(
                "can't get field:{field} index:{index} from {}",
                infos.join(",")
            )
        })?;

        val.parse()
            .map_err(|_| format!("can't parse required field:{field} of {val} to i32"))
    }

    fn extract_optional_info(infos: &[&str], index: usize) -> Option<i32> {
        infos.get(index).and_then(|val| val.parse().ok())
    }

    pub(crate) fn effective_layout(&self) -> i32 {
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
        let infos: Vec<&str> = header_line.split_whitespace().collect();

        if infos.len() < 6 {
            return Err("headerline is illegal".to_string());
        }

        let (signature, hardblank) =
            HeaderLine::extract_signature_with_hardblank(infos.first().unwrap())?;

        Ok(HeaderLine {
            header_line: String::from(header_line),
            signature,
            hardblank,
            height: HeaderLine::extract_required_info(&infos, 1, "height")?,
            baseline: HeaderLine::extract_required_info(&infos, 2, "baseline")?,
            max_length: HeaderLine::extract_required_info(&infos, 3, "max length")?,
            old_layout: HeaderLine::extract_required_info(&infos, 4, "old layout")?,
            comment_lines: HeaderLine::extract_required_info(&infos, 5, "comment lines")?,
            print_direction: HeaderLine::extract_optional_info(&infos, 6),
            full_layout: HeaderLine::extract_optional_info(&infos, 7),
            codetag_count: HeaderLine::extract_optional_info(&infos, 8),
        })
    }
}

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

#[derive(Debug)]
pub struct FIGure<'a> {
    pub characters: Vec<&'a FIGcharacter>,
    pub height: u32,
    lines: Vec<String>,
}

impl<'a> FIGure<'a> {
    pub(crate) fn is_not_empty(&self) -> bool {
        !self.characters.is_empty() && self.height > 0
    }

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
    header_line: &'a HeaderLine,
    prev_char_width: usize,
    cur_char_width: usize,
    max_smush: usize,
}

impl<'a> Renderer<'a> {
    fn new(header_line: &'a HeaderLine, _fonts: &'a HashMap<u32, FIGcharacter>) -> Self {
        Self {
            header_line,
            prev_char_width: 0,
            cur_char_width: 0,
            max_smush: 0,
        }
    }

    fn render(mut self, characters: &[&FIGcharacter]) -> Vec<String> {
        let mut buffer = vec![String::new(); self.header_line.height as usize];
        for character in characters {
            self.cur_char_width = character.width as usize;
            self.max_smush = self.smush_amount(&buffer, character);

            for (row, buffer_row) in buffer.iter_mut().enumerate() {
                self.add_char_row_to_buffer_row(buffer_row, &character.characters[row]);
            }

            self.prev_char_width = self.cur_char_width;
        }

        buffer
            .into_iter()
            .map(|line| line.replace(self.header_line.hardblank, " "))
            .collect()
    }

    fn add_char_row_to_buffer_row(&self, buffer_row: &mut String, char_row: &str) {
        let (mut left, right) = if self.header_line.is_right_to_left() {
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
        let layout = self.header_line.effective_layout();
        if (layout & (SM_SMUSH | SM_KERN)) == 0 {
            return 0;
        }

        let mut max_smush = self.cur_char_width;
        for (row, buffer_row) in buffer
            .iter()
            .enumerate()
            .take(self.header_line.height as usize)
        {
            let (line_left, line_right) = if self.header_line.is_right_to_left() {
                (&character.characters[row], buffer_row)
            } else {
                (buffer_row, &character.characters[row])
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
            if ch1 == '\0' || ch1 == ' ' || (ch2 != '\0' && self.smush_chars(ch1, ch2).is_some()) {
                amount += 1;
            }

            max_smush = max_smush.min(amount.max(0) as usize);
        }

        max_smush
    }

    pub(crate) fn smush_chars(&self, left: char, right: char) -> Option<char> {
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

        let layout = self.header_line.effective_layout();
        if (layout & SM_SMUSH) == 0 {
            return None;
        }

        if (layout & 63) == 0 {
            if left == self.header_line.hardblank {
                return Some(right);
            }
            if right == self.header_line.hardblank {
                return Some(left);
            }

            return if self.header_line.is_right_to_left() {
                Some(left)
            } else {
                Some(right)
            };
        }

        if (layout & SM_HARDBLANK) != 0
            && left == self.header_line.hardblank
            && right == self.header_line.hardblank
        {
            return Some(left);
        }
        if left == self.header_line.hardblank || right == self.header_line.hardblank {
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
