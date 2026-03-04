use thiserror::Error;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Doctype { name: String },
    StartTag { name: String, attrs: Vec<(String, String)>, self_closing: bool },
    EndTag { name: String },
    Comment(String),
    Character(String),
    EOF,
}

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("unexpected end of input")]
    Eof,
}


pub struct Tokenizer<'a> {
    src: &'a str,
    i: usize,
    len: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, i: 0, len: src.len() }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.i >= self.len { return None; }
        let c = self.src[self.i..].chars().next()?;
        self.i += c.len_utf8();
        Some(c)
    }

    fn starts_with_from(&self, pat: &str) -> bool {
        self.src[self.i..].starts_with(pat)
    }

    fn bump_str(&mut self, s: &str) -> bool {
        if self.starts_with_from(s) {
            self.i += s.len();
            true
        } else { false }
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() { self.next_char(); } else { break; }
        }
    }

    fn read_until<'b>(&mut self, stop: char) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if c == stop { break; }
            out.push(c);
            self.next_char();
        }
        out
    }

    fn read_name(&mut self) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' {
                out.push(c.to_ascii_lowercase());
                self.next_char();
            } else { break; }
        }
        out
    }

    fn read_attr_value(&mut self) -> String {
        self.skip_ws();
        match self.peek() {
            Some('"') => {
                self.next_char();
                let v = self.read_until('"');
                let _ = self.next_char(); 
                v
            }
            Some('\'') => {
                self.next_char();
                let v = self.read_until('\'');
                let _ = self.next_char(); 
                v
            }
            Some(_) => {
                
                let mut out = String::new();
                while let Some(c) = self.peek() {
                    if c.is_whitespace() || c == '>' || c == '/' { break; }
                    out.push(c);
                    self.next_char();
                }
                out
            }
            None => String::new(),
        }
    }

    fn read_attributes(&mut self) -> (Vec<(String, String)>, bool) {
        let mut attrs: Vec<(String, String)> = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_ws();
            if self.peek().is_none() { break; }
            if self.starts_with_from("/>") {
                self.i += 2;
                self_closing = true;
                break;
            }
            if self.starts_with_from(">") {
                self.i += 1;
                break;
            }

            let name = self.read_name();
            if name.is_empty() {
                
                let _ = self.next_char();
                continue;
            }

            self.skip_ws();
            let value = if self.bump_str("=") {
                self.skip_ws();
                self.read_attr_value()
            } else {
                
                String::new()
            };

            attrs.push((name, value));
        }

        
        attrs.sort_by(|a, b| a.0.cmp(&b.0));
        (attrs, self_closing)
    }

    fn emit_text(&mut self) -> Option<Token> {
        let start = self.i;
        while let Some(c) = self.peek() {
            if c == '<' { break; }
            self.next_char();
        }
        if self.i > start {
            let s = self.src[start..self.i].to_string();
            
            let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
            if collapsed.is_empty() {
                return self.next(); 
            }
            return Some(Token::Character(collapsed));
        }
        None
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.len { return Some(Token::EOF); }

        if let Some(tok) = self.emit_text() {
            return Some(tok);
        }

        
        if self.bump_str("<!--") {
            
            if let Some(end) = self.src[self.i..].find("-->") {
                let s = &self.src[self.i..self.i + end];
                self.i += end + 3;
                return Some(Token::Comment(s.to_string()));
            } else {
                
                let s = &self.src[self.i..];
                self.i = self.len;
                return Some(Token::Comment(s.to_string()));
            }
        }

        if self.bump_str("<!DOCTYPE") || self.bump_str("<!doctype") {
            
            self.skip_ws();
            let name = self.read_name();
            
            while let Some(c) = self.next_char() { if c == '>' { break; } }
            return Some(Token::Doctype { name });
        }

        if self.bump_str("</") {
            let name = self.read_name();
            
            while let Some(c) = self.next_char() { if c == '>' { break; } }
            return Some(Token::EndTag { name });
        }

        if self.bump_str("<") {
            let name = self.read_name();
            let (attrs, self_closing) = self.read_attributes();
            return Some(Token::StartTag { name, attrs, self_closing });
        }

        
        let _ = self.next_char();
        self.next()
    }
}
