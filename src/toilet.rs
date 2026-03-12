use crate::shared::{
    load_font_file, parse_font_bytes, parse_font_content, render, FIGcharacter, FIGure, FontData,
    HeaderLine,
};
use std::collections::HashMap;

/// Toilet font, which supports loading `.tlf` files, including zip-packaged fonts.
#[derive(Debug, Clone)]
pub struct Toilet {
    pub header_line: HeaderLine,
    pub comments: String,
    pub fonts: HashMap<u32, FIGcharacter>,
}

impl Toilet {
    /// generate Toilet font from string literal
    pub fn from_content(contents: &str) -> Result<Toilet, String> {
        Ok(parse_font_content(contents)?.into())
    }

    /// generate Toilet font from specified file
    pub fn from_file(fontname: &str) -> Result<Toilet, String> {
        Ok(load_font_file(fontname)?.into())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Toilet, String> {
        Ok(parse_font_bytes(bytes)?.into())
    }

    /// the smblock Toilet font bundled with the crate
    pub fn smblock() -> Result<Toilet, String> {
        Toilet::from_bytes(include_bytes!("../resources/smblock.tlf"))
    }

    /// the mono12 Toilet font bundled with the crate
    pub fn mono12() -> Result<Toilet, String> {
        Toilet::from_bytes(include_bytes!("../resources/mono12.tlf"))
    }

    /// the future Toilet font bundled with the crate
    pub fn future() -> Result<Toilet, String> {
        Toilet::from_bytes(include_bytes!("../resources/future.tlf"))
    }

    /// the wideterm Toilet font bundled with the crate
    pub fn wideterm() -> Result<Toilet, String> {
        Toilet::from_bytes(include_bytes!("../resources/wideterm.tlf"))
    }

    /// the mono9 Toilet font bundled with the crate
    pub fn mono9() -> Result<Toilet, String> {
        Toilet::from_bytes(include_bytes!("../resources/mono9.tlf"))
    }

    /// convert string literal to FIGure
    pub fn convert(&self, message: &str) -> Option<FIGure<'_>> {
        render(&self.header_line, &self.fonts, message)
    }
}

impl From<FontData> for Toilet {
    fn from(data: FontData) -> Self {
        Self {
            header_line: data.header_line,
            comments: data.comments,
            fonts: data.fonts,
        }
    }
}
