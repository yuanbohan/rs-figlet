use crate::shared::{
    load_font_file, parse_font_content, render, FIGcharacter, FIGure, FontData, HeaderLine,
};
use std::collections::HashMap;

/// FIGlet font, which will hold the mapping from u32 code to FIGcharacter
#[derive(Debug, Clone)]
pub struct FIGlet {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

impl FIGlet {
    /// generate FIGlet font from string literal
    pub fn from_content(contents: &str) -> Result<FIGlet, String> {
        Ok(parse_font_content(contents)?.into())
    }

    /// generate FIGlet font from specified file
    pub fn from_file(fontname: &str) -> Result<FIGlet, String> {
        Ok(load_font_file(fontname)?.into())
    }

    /// the standard FIGlet font, which you can find [`fontdb`]
    ///
    /// [`fontdb`]: http://www.figlet.org/fontdb.cgi
    pub fn standard() -> Result<FIGlet, String> {
        Ok(parse_font_content(include_str!("../resources/standard.flf"))?.into())
    }

    /// the small FIGlet font bundled with the crate
    pub fn small() -> Result<FIGlet, String> {
        Ok(parse_font_content(include_str!("../resources/small.flf"))?.into())
    }

    /// the big FIGlet font bundled with the crate
    pub fn big() -> Result<FIGlet, String> {
        Ok(parse_font_content(include_str!("../resources/big.flf"))?.into())
    }

    /// the slant FIGlet font bundled with the crate
    pub fn slant() -> Result<FIGlet, String> {
        Ok(parse_font_content(include_str!("../resources/slant.flf"))?.into())
    }

    /// convert string literal to FIGure
    pub fn convert(&self, message: &str) -> Option<FIGure<'_>> {
        render(&self.header_line, &self.fonts, message)
    }
}

impl From<FontData> for FIGlet {
    fn from(data: FontData) -> Self {
        Self {
            header_line: data.header_line,
            comments: data.comments,
            fonts: data.fonts,
        }
    }
}
