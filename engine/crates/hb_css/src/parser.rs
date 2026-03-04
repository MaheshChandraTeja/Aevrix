use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::BTreeMap;

use crate::selectors::{parse_selector_list, Selector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub fn rgba(r:u8,g:u8,b:u8,a:u8)->Self{Self(r,g,b,a)}
    pub fn to_rgba(self)->[u8;4]{[self.0,self.1,self.2,self.3]}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Color(Color),
    LengthPx(f32),
    Ident(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub source_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected EOF")]
    Eof,
    #[error("invalid token near {0}")]
    Invalid(String),
}

pub fn parse_css(input: &str) -> Stylesheet {
    let mut p = Parser::new(input);
    p.parse_stylesheet()
}


pub fn parse_inline_decls(input: &str) -> Vec<Declaration> {
    let mut p = Parser::new(input);
    p.parse_declaration_list()
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    order: u32,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { s: src.as_bytes(), i: 0, order: 0 }
    }

    fn eof(&self) -> bool { self.i >= self.s.len() }
    fn cur(&self) -> Option<u8> { self.s.get(self.i).cloned() }
    fn bump(&mut self) -> Option<u8> { let c = self.cur()?; self.i += 1; Some(c) }

    fn skip_ws(&mut self) {
        while let Some(c) = self.cur() {
            if c.is_ascii_whitespace() { self.i += 1; }
            else if self.starts_with(b"/*") {
                self.i += 2;
                while !self.eof() && !self.starts_with(b"*/") { self.i += 1; }
                if self.starts_with(b"*/") { self.i += 2; }
            } else { break; }
        }
    }

    fn starts_with(&self, pat: &[u8]) -> bool {
        self.s.get(self.i..self.i+pat.len()).map_or(false, |w| w == pat)
    }

    fn read_while<F: Fn(u8)->bool>(&mut self, f: F) -> String {
        let start = self.i;
        while let Some(c) = self.cur() {
            if f(c) { self.i += 1; } else { break; }
        }
        String::from_utf8_lossy(&self.s[start..self.i]).to_lowercase()
    }

    fn read_ident(&mut self) -> String {
        self.read_while(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    }

    fn expect_byte(&mut self, b: u8) {
        self.skip_ws();
        if self.cur() == Some(b) { self.i += 1; } else {  }
    }

    fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        while !self.eof() {
            self.skip_ws();
            if self.eof() { break; }

            let sel_src = self.read_until_byte(b'{');
            let selectors = parse_selector_list(&sel_src);
            self.expect_byte(b'{');

            let decls = self.parse_declaration_block();

            self.expect_byte(b'}');
            self.order += 1;
            rules.push(Rule { selectors, declarations: decls, source_order: self.order });
        }
        Stylesheet { rules }
    }

    fn parse_declaration_block(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_ws();
            if self.eof() || self.cur() == Some(b'}') { break; }
            let name = self.read_ident();
            self.skip_ws();
            if self.cur() == Some(b':') { self.i += 1; }
            self.skip_ws();
            let value = self.read_value();
            decls.push(Declaration { name, value });
            self.skip_ws();
            if self.cur() == Some(b';') { self.i += 1; }
        }
        decls
    }

    fn parse_declaration_list(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_ws();
            if self.eof() { break; }
            let name = self.read_ident();
            self.skip_ws();
            if self.cur() == Some(b':') { self.i += 1; } else { break; }
            self.skip_ws();
            let value = self.read_value();
            decls.push(Declaration { name, value });
            self.skip_ws();
            if self.cur() == Some(b';') { self.i += 1; }
        }
        decls
    }

    fn read_value(&mut self) -> Value {
        
        if self.cur() == Some(b'#') {
            self.i += 1;
            let hex = self.read_while(|c| c.is_ascii_hexdigit());
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                return Value::Color(Color(r,g,b,255));
            }
        }

        
        let mut tmp = String::new();
        while let Some(c) = self.cur() {
            if c.is_ascii_digit() || c == b'.' || c == b'-' { tmp.push(c as char); self.i += 1; }
            else { break; }
        }
        if !tmp.is_empty() && self.read_while(|c| c.is_ascii_alphabetic()) == "px" {
            if let Ok(n) = tmp.parse::<f32>() {
                return Value::LengthPx(n);
            }
        }

        
        let ident = if tmp.is_empty() {
            self.read_ident()
        } else {
            tmp + &self.read_ident()
        };
        Value::Ident(ident)
    }

    fn read_until_byte(&mut self, byte: u8) -> String {
        let start = self.i;
        while let Some(c) = self.cur() {
            if c == byte { break; }
            self.i += 1;
        }
        String::from_utf8_lossy(&self.s[start..self.i]).to_string()
    }
}
