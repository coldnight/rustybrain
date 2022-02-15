use std::fmt::Write;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use tree_sitter::Tree;

pub enum ZettelError {
    IOError(io::Error),
    ParseHeaderError(toml::de::Error),
}

impl From<io::Error> for ZettelError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<toml::de::Error> for ZettelError {
    fn from(e: toml::de::Error) -> Self {
        Self::ParseHeaderError(e)
    }
}

pub struct Zettel {
    path: String,
    header: ZettelHeader,
    content: String,
    link_to: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ZettelHeader {
    title: String,

    #[serde(skip)]
    raw: String,
}

impl ZettelHeader {
    pub fn from_cursor(
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<Self, ZettelError> {
        let raw = Self::read(cursor)?;
        let s = toml::from_str(&raw)?;
        Ok(s)
    }

    fn read(cursor: &mut Cursor<Vec<u8>>) -> Result<String, ZettelError> {
        let mut line_buf: String = String::new();
        let mut header: String = String::new();
        cursor.read_line(&mut line_buf)?;
        if let Some(remain) = line_buf.strip_prefix("+") {
            if remain != "" {
                return Ok(header);
            }
            loop {
                line_buf.clear();
                cursor.read_line(&mut &mut line_buf)?;

                if let Some(remain) = line_buf.strip_prefix("+") {
                    if remain == "" {
                        return Ok(header);
                    }
                }
                header.write_str(&line_buf);
            }
        }
        Ok(header)
    }
}

impl Zettel {
    pub fn from_md(path: &str) -> Result<Self, ZettelError> {
        let mut file = File::open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        let mut cursor = Cursor::new(buf);
        let header = ZettelHeader::from_cursor(&mut cursor)?;
        let mut content: String = String::new();
        cursor.read_to_string(&mut content)?;
        Ok(Zettel {
            path: path.to_string(),
            header,
            content,
            link_to: vec![],
        })
    }

    pub fn path(&self) -> &str {
        todo!()
    }

    pub fn title(&self) -> &str {
        todo!()
    }

    pub fn content(&self) -> &str {
        todo!()
    }

    pub fn contexts(&self) -> &str {
        todo!()
    }

    pub fn tree(&self) -> Tree {
        todo!()
    }

    pub fn set_title(&mut self, title: &str) {}

    pub fn set_content(&mut self, content: &str) {}

    pub fn save() {}
}
