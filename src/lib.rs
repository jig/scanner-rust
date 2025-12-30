// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Copyright 2022 Jordi Íñigo Griera. All rights reserved.

//! # Scanner
//!
//! A scanner and tokenizer for UTF-8-encoded text.
//! It takes a reader providing the source, which then can be tokenized
//! through repeated calls to the `scan()` function. For compatibility with
//! existing tools, the NUL character is not allowed. If the first character
//! in the source is a UTF-8 encoded byte order mark (BOM), it is discarded.
//!
//! By default, a Scanner skips white space and Lisp comments and recognizes all
//! literals as defined by the Lisp language as specified on the
//! jig/lisp implementation. It may be customized to recognize only a subset of
//! those literals and to recognize different identifier and white
//! space characters.

use std::fmt;
use std::io::Read;

const BUF_LEN: usize = 1024; // at least 4 (utf8 max bytes)

/// Position is a value that represents a source position.
/// A position is valid if line > 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub filename: String,
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    /// Reports whether the position is valid.
    pub fn is_valid(&self) -> bool {
        self.line > 0
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if self.filename.is_empty() {
            "<input>".to_string()
        } else {
            self.filename.clone()
        };
        
        if self.is_valid() {
            write!(f, "{}:{}:{}", s, self.line, self.column)
        } else {
            write!(f, "{}", s)
        }
    }
}

/// Token type
pub type Token = i32;

/// The result of Scan is one of these tokens or a Unicode character.
pub const EOF: Token = -1;
pub const IDENT: Token = -2;
pub const INT: Token = -3;
pub const FLOAT: Token = -4;
pub const STRING: Token = -5;
pub const KEYWORD: Token = -6;
pub const RAW_STRING: Token = -7;
pub const COMMENT: Token = -8;
const SKIP_COMMENT: Token = -9;

/// Predefined mode bits to control recognition of tokens.
pub const SCAN_IDENTS: u32 = 1 << (-IDENT as u32);
pub const SCAN_INTS: u32 = 1 << (-INT as u32);
pub const SCAN_FLOATS: u32 = 1 << (-FLOAT as u32);
pub const SCAN_STRINGS: u32 = 1 << (-STRING as u32);
pub const SCAN_KEYWORDS: u32 = 1 << (-KEYWORD as u32);
pub const SCAN_RAW_STRINGS: u32 = 1 << (-RAW_STRING as u32);
pub const SCAN_COMMENTS: u32 = 1 << (-COMMENT as u32);
pub const SKIP_COMMENTS: u32 = 1 << (-SKIP_COMMENT as u32);

/// Standard Lisp tokens mode
pub const LISP_TOKENS: u32 = SCAN_IDENTS | SCAN_FLOATS | SCAN_STRINGS | SCAN_KEYWORDS | SCAN_RAW_STRINGS | SCAN_COMMENTS | SKIP_COMMENTS;

/// Default whitespace characters
pub const LISP_WHITESPACE: u64 = (1 << b'\t') | (1 << b'\n') | (1 << b'\r') | (1 << b' ');

/// Returns a printable string for a token or Unicode character.
pub fn token_string(tok: Token) -> String {
    match tok {
        EOF => "EOF".to_string(),
        IDENT => "Ident".to_string(),
        INT => "Int".to_string(),
        FLOAT => "Float".to_string(),
        STRING => "String".to_string(),
        KEYWORD => "Keyword".to_string(),
        RAW_STRING => "RawString".to_string(),
        COMMENT => "Comment".to_string(),
        _ => {
            if let Some(ch) = char::from_u32(tok as u32) {
                format!("{:?}", ch.to_string())
            } else {
                format!("Token({})", tok)
            }
        }
    }
}

/// A Scanner implements reading of Unicode characters and tokens from a reader.
pub struct Scanner<R: Read> {
    // Input
    src: R,

    // Source buffer
    src_buf: [u8; BUF_LEN + 1],
    src_pos: usize,
    src_end: usize,

    // Source position
    src_buf_offset: usize,
    line: usize,
    column: usize,
    last_line_len: usize,
    last_char_len: usize,

    // Token text buffer
    tok_buf: Vec<u8>,
    tok_pos: isize,
    tok_end: usize,

    // One character look-ahead
    ch: i32,

    // Error handling
    error_count: usize,
    
    // Configuration
    pub mode: u32,
    pub whitespace: u64,
    is_ident_rune: Option<Box<dyn Fn(char, usize) -> bool>>,

    // Token position
    pub position: Position,
}

impl<R: Read> Scanner<R> {
    /// Initializes a Scanner with a new source and returns it.
    pub fn init(src: R) -> Self {
        let mut scanner = Scanner {
            src,
            src_buf: [0; BUF_LEN + 1],
            src_pos: 0,
            src_end: 0,
            src_buf_offset: 0,
            line: 1,
            column: 0,
            last_line_len: 0,
            last_char_len: 0,
            tok_buf: Vec::new(),
            tok_pos: -1,
            tok_end: 0,
            ch: -2,
            error_count: 0,
            mode: LISP_TOKENS,
            whitespace: LISP_WHITESPACE,
            is_ident_rune: None,
            position: Position {
                filename: String::new(),
                offset: 0,
                line: 0,
                column: 0,
            },
        };
        
        // Set sentinel
        scanner.src_buf[0] = 128; // utf8.RuneSelf equivalent
        scanner
    }

    /// Sets the mode field
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Sets the whitespace field
    pub fn set_whitespace(&mut self, whitespace: u64) {
        self.whitespace = whitespace;
    }

    /// Sets the is_ident_rune predicate
    pub fn set_is_ident_rune<F>(&mut self, f: F)
    where
        F: Fn(char, usize) -> bool + 'static,
    {
        self.is_ident_rune = Some(Box::new(f));
    }

    /// Gets the error count
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    fn error(&mut self, msg: &str) {
        self.tok_end = self.src_pos.saturating_sub(self.last_char_len);
        self.error_count += 1;
        // In production, you might want to call an error callback here
        eprintln!("Scanner error: {}", msg);
    }

    fn char_to_token(&self, ch: char) -> Token {
        if ch == '\u{FFFF}' {
            EOF
        } else {
            ch as i32
        }
    }

    fn is_ident_rune_default(&self, ch: char, i: usize) -> bool {
        ch == '_'
            || ch == '$'
            || ch == '*'
            || ch == '+'
            || ch == '/'
            || ch == '?'
            || ch == '!'
            || ch == '<'
            || ch == '>'
            || ch == '='
            || ch.is_alphabetic()
            || (ch == '-' && i > 0)
            || (ch.is_numeric() && i > 0)
    }

    fn is_ident_rune_check(&self, ch: char, i: usize) -> bool {
        if let Some(ref f) = self.is_ident_rune {
            ch as i32 != EOF && f(ch, i)
        } else {
            self.is_ident_rune_default(ch, i)
        }
    }

    fn next(&mut self) -> char {
        let mut ch: u32;
        let mut width = 1;

        if (self.src_buf[self.src_pos] as u32) < 128 {
            ch = self.src_buf[self.src_pos] as u32;
        } else {
            // Uncommon case: not ASCII or not enough bytes
            loop {
                let remaining = self.src_end - self.src_pos;
                if remaining >= 4 {
                    break;
                }
                
                // Check if we have a complete UTF-8 sequence
                if remaining > 0 {
                    let bytes = &self.src_buf[self.src_pos..self.src_end];
                    if let Ok(s) = std::str::from_utf8(bytes) {
                        if !s.is_empty() {
                            break;
                        }
                    }
                }

                // Save token text if any
                if self.tok_pos >= 0 {
                    self.tok_buf.extend_from_slice(&self.src_buf[self.tok_pos as usize..self.src_pos]);
                    self.tok_pos = 0;
                }

                // Move unread bytes to beginning of buffer
                self.src_buf.copy_within(self.src_pos..self.src_end, 0);
                self.src_buf_offset += self.src_pos;

                // Read more bytes
                let i = self.src_end - self.src_pos;
                match self.src.read(&mut self.src_buf[i..BUF_LEN]) {
                    Ok(0) | Err(_) => {
                        self.src_pos = 0;
                        self.src_end = i;
                        self.src_buf[self.src_end] = 128;
                        
                        if self.src_end == 0 {
                            if self.last_char_len > 0 {
                                self.column += 1;
                            }
                            self.last_char_len = 0;
                            return '\u{FFFF}'; // EOF marker
                        }
                        break;
                    }
                    Ok(n) => {
                        self.src_pos = 0;
                        self.src_end = i + n;
                        self.src_buf[self.src_end] = 128;
                    }
                }
            }

            // Decode UTF-8
            ch = self.src_buf[self.src_pos] as u32;
            if ch >= 128 {
                let bytes = &self.src_buf[self.src_pos..self.src_end];
                if let Ok(s) = std::str::from_utf8(bytes) {
                    if let Some(decoded_ch) = s.chars().next() {
                        ch = decoded_ch as u32;
                        width = decoded_ch.len_utf8();
                    } else {
                        self.src_pos += 1;
                        self.last_char_len = 1;
                        self.column += 1;
                        self.error("invalid UTF-8 encoding");
                        return '\u{FFFD}'; // Replacement character
                    }
                } else {
                    self.src_pos += 1;
                    self.last_char_len = 1;
                    self.column += 1;
                    self.error("invalid UTF-8 encoding");
                    return '\u{FFFD}';
                }
            }
        }

        // Advance
        self.src_pos += width;
        self.last_char_len = width;
        self.column += 1;

        let result = char::from_u32(ch).unwrap_or('\u{FFFD}');

        // Special situations
        if result == '\0' {
            self.error("invalid character NUL");
        } else if result == '\n' {
            self.line += 1;
            self.last_line_len = self.column;
            self.column = 0;
        }

        result
    }

    /// Reads and returns the next Unicode character.
    pub fn next_char(&mut self) -> Token {
        self.tok_pos = -1;
        self.position.line = 0;
        let ch = self.peek();
        if ch != EOF {
            let next_char = self.next();
            if next_char == '\u{FFFF}' {
                self.ch = EOF;
            } else {
                self.ch = next_char as i32;
            }
        }
        ch
    }

    /// Returns the next Unicode character without advancing the scanner.
    pub fn peek(&mut self) -> Token {
        if self.ch == -2 {
            let next_char = self.next();
            if next_char == '\u{FFFF}' {
                self.ch = EOF;
            } else {
                self.ch = next_char as i32;
                if self.ch == 0xFEFF {
                    let bom_next = self.next();
                    if bom_next == '\u{FFFF}' {
                        self.ch = EOF;
                    } else {
                        self.ch = bom_next as i32; // ignore BOM
                    }
                }
            }
        }
        self.ch
    }

    fn scan_identifier(&mut self) -> char {
        let mut ch = self.next();
        let mut i = 1;
        while self.is_ident_rune_check(ch, i) {
            ch = self.next();
            i += 1;
        }
        ch
    }

    fn lower(ch: char) -> char {
        if ch.is_ascii_uppercase() {
            ch.to_ascii_lowercase()
        } else {
            ch
        }
    }

    fn is_decimal(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn is_hex(ch: char) -> bool {
        ch.is_ascii_hexdigit()
    }

    fn digits(&mut self, mut ch: char, base: u32, invalid: &mut Option<char>) -> (char, i32) {
        let mut digsep = 0;

        if base <= 10 {
            let max = char::from_u32('0' as u32 + base).unwrap();
            while Self::is_decimal(ch) || ch == '_' {
                let ds = if ch == '_' { 2 } else { 1 };
                if ch >= max && invalid.is_none() {
                    *invalid = Some(ch);
                }
                digsep |= ds;
                ch = self.next();
            }
        } else {
            while Self::is_hex(ch) || ch == '_' {
                let ds = if ch == '_' { 2 } else { 1 };
                digsep |= ds;
                ch = self.next();
            }
        }

        (ch, digsep)
    }

    fn scan_number(&mut self, mut ch: char, mut seen_dot: bool, negative: bool) -> (Token, char) {
        let mut base = 10;
        let mut prefix = '\0';
        let mut digsep = 0;
        let mut invalid: Option<char> = None;

        let mut tok = INT;

        // Integer part
        if !seen_dot {
            if ch == '0' {
                ch = self.next();
                match Self::lower(ch) {
                    'x' => {
                        ch = self.next();
                        base = 16;
                        prefix = 'x';
                    }
                    'o' => {
                        ch = self.next();
                        base = 8;
                        prefix = 'o';
                    }
                    'b' => {
                        ch = self.next();
                        base = 2;
                        prefix = 'b';
                    }
                    _ => {
                        base = 8;
                        prefix = '0';
                        digsep = 1;
                    }
                }
            } else if ch == '-' {
                ch = self.next();
            }

            let (new_ch, ds) = self.digits(ch, base, &mut invalid);
            ch = new_ch;
            digsep |= ds;

            if ch == '.' && (self.mode & SCAN_FLOATS) != 0 {
                ch = self.next();
                seen_dot = true;
            }
        }

        // Fractional part
        if seen_dot {
            tok = FLOAT;
            if prefix == 'o' || prefix == 'b' {
                self.error(&format!("invalid radix point in {}", Self::litname(prefix)));
            }
            let (new_ch, ds) = self.digits(ch, base, &mut invalid);
            ch = new_ch;
            digsep |= ds;
        }

        if (digsep & 1) == 0 {
            if negative {
                tok = '-' as i32;
            } else {
                self.error(&format!("{} has no digits", Self::litname(prefix)));
            }
        }

        // Exponent
        let e = Self::lower(ch);
        if (e == 'e' || e == 'p') && (self.mode & SCAN_FLOATS) != 0 {
            if e == 'e' && prefix != '\0' && prefix != '0' {
                self.error(&format!("'{}' exponent requires decimal mantissa", ch));
            } else if e == 'p' && prefix != 'x' {
                self.error(&format!("'{}' exponent requires hexadecimal mantissa", ch));
            }

            ch = self.next();
            tok = FLOAT;

            if ch == '+' || ch == '-' {
                ch = self.next();
            }

            let (new_ch, ds) = self.digits(ch, 10, &mut None);
            ch = new_ch;
            digsep |= ds;

            if (ds & 1) == 0 {
                self.error("exponent has no digits");
            }
        } else if prefix == 'x' && tok == FLOAT {
            self.error("hexadecimal mantissa requires a 'p' exponent");
        }

        if tok == INT && invalid.is_some() {
            self.error(&format!("invalid digit '{}' in {}", invalid.unwrap(), Self::litname(prefix)));
        }

        if (digsep & 2) != 0 {
            self.tok_end = self.src_pos - self.last_char_len;
            if let Some(_) = Self::invalid_sep(&self.token_text()) {
                self.error("'_' must separate successive digits");
            }
        }

        (tok, ch)
    }

    fn litname(prefix: char) -> String {
        match prefix {
            'x' => "hexadecimal literal".to_string(),
            'o' | '0' => "octal literal".to_string(),
            'b' => "binary literal".to_string(),
            _ => "decimal literal".to_string(),
        }
    }

    fn invalid_sep(x: &str) -> Option<usize> {
        let bytes = x.as_bytes();
        if bytes.is_empty() {
            return None;
        }

        let mut x1 = ' ';
        let mut d = '.';
        let mut i = 0;

        if bytes.len() >= 2 && bytes[0] == b'0' {
            x1 = Self::lower(bytes[1] as char);
            if x1 == 'x' || x1 == 'o' || x1 == 'b' {
                d = '0';
                i = 2;
            }
        }

        while i < bytes.len() {
            let p = d;
            d = bytes[i] as char;
            
            if d == '_' {
                if p != '0' {
                    return Some(i);
                }
            } else if Self::is_decimal(d) || (x1 == 'x' && Self::is_hex(d)) {
                d = '0';
            } else {
                if p == '_' {
                    return Some(i - 1);
                }
                d = '.';
            }
            i += 1;
        }

        if d == '_' {
            return Some(bytes.len() - 1);
        }

        None
    }

    fn digit_val(ch: char) -> u32 {
        match ch {
            '0'..='9' => (ch as u32) - ('0' as u32),
            'a'..='f' => (ch as u32) - ('a' as u32) + 10,
            'A'..='F' => (ch as u32) - ('A' as u32) + 10,
            _ => 16,
        }
    }

    fn scan_digits(&mut self, mut ch: char, base: u32, mut n: usize) -> char {
        while n > 0 && Self::digit_val(ch) < base {
            ch = self.next();
            n -= 1;
        }
        if n > 0 {
            self.error("invalid char escape");
        }
        ch
    }

    fn scan_escape(&mut self, quote: char) -> char {
        let mut ch = self.next();
        
        match ch {
            'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' => {
                if ch == quote {
                    ch = self.next();
                } else {
                    ch = self.next();
                }
            }
            '0'..='7' => {
                ch = self.scan_digits(ch, 8, 3);
            }
            'x' => {
                let next_ch = self.next();
                ch = self.scan_digits(next_ch, 16, 2);
            }
            'u' => {
                let next_ch = self.next();
                ch = self.scan_digits(next_ch, 16, 4);
            }
            'U' => {
                let next_ch = self.next();
                ch = self.scan_digits(next_ch, 16, 8);
            }
            c if c == quote => {
                ch = self.next();
            }
            _ => {
                self.error("invalid char escape");
            }
        }
        ch
    }

    fn scan_string(&mut self, quote: char) -> usize {
        let mut ch = self.next();
        let mut n = 0;

        while ch != quote {
            if ch == '\n' || ch == '\u{FFFF}' {
                self.error("literal not terminated");
                return n;
            }
            if ch == '\\' {
                ch = self.scan_escape(quote);
            } else {
                ch = self.next();
            }
            n += 1;
        }
        n
    }

    fn scan_raw_string(&mut self) -> char {
        loop {
            let mut ch = self.next();
            while ch != '¬' {
                if ch == '\u{FFFF}' {
                    self.error("literal not terminated");
                    return '\0';
                }
                ch = self.next();
            }
            ch = self.next();
            if ch != '¬' {
                return ch;
            }
        }
    }

    fn scan_comment(&mut self, mut ch: char) -> char {
        if ch != '\n' {
            ch = self.next();
            while ch != '\n' && ch != '\u{FFFF}' {
                ch = self.next();
            }
        }
        ch
    }

    /// Scans and returns the next token or Unicode character.
    pub fn scan(&mut self) -> Token {
        let mut ch = self.peek();
        if ch == EOF {
            return EOF;
        }
        
        let mut ch_char = char::from_u32(ch as u32).unwrap_or('\u{FFFF}');
        if ch_char == '\u{FFFF}' {
            return EOF;
        }

        // Reset token text position
        self.tok_pos = -1;
        self.position.line = 0;

        // Skip white space
        let mut ch_u32 = ch_char as u32;
        while ch_u32 < 64 && (self.whitespace & (1 << ch_u32)) != 0 {
            let next = self.next();
            if next == '\u{FFFF}' {
                return EOF;
            }
            ch_char = next;
            ch_u32 = next as u32;
            ch = next as i32;
        }

        // Start collecting token text
        self.tok_buf.clear();
        self.tok_pos = (self.src_pos - self.last_char_len) as isize;

        // Set token position
        self.position.offset = self.src_buf_offset + (self.tok_pos as usize);
        if self.column > 0 {
            self.position.line = self.line;
            self.position.column = self.column;
        } else {
            self.position.line = self.line - 1;
            self.position.column = self.last_line_len;
        }

        // Determine token value
        let mut tok = ch;

        if self.is_ident_rune_check(ch_char, 0) {
            if (self.mode & SCAN_IDENTS) != 0 {
                tok = IDENT;
                let new_ch = self.scan_identifier();
                self.ch = self.char_to_token(new_ch);
            } else {
                let ch = self.next();
                self.ch = self.char_to_token(ch);
            }
        } else if Self::is_decimal(ch_char) {
            if (self.mode & (SCAN_INTS | SCAN_FLOATS)) != 0 {
                let (new_tok, new_ch) = self.scan_number(ch_char, false, false);
                tok = new_tok;
                self.ch = self.char_to_token(new_ch);
            } else {
                let ch = self.next();
                self.ch = self.char_to_token(ch);
            }
        } else if ch_char == '-' {
            let next_ch = self.next();
            if self.is_ident_rune_check(next_ch, 0) {
                if (self.mode & SCAN_IDENTS) != 0 {
                    tok = IDENT;
                    let new_ch = self.scan_identifier();
                    self.ch = self.char_to_token(new_ch);
                }
            } else if Self::is_decimal(next_ch) {
                if (self.mode & (SCAN_INTS | SCAN_FLOATS)) != 0 {
                    let (new_tok, new_ch) = self.scan_number(next_ch, false, true);
                    tok = new_tok;
                    self.ch = self.char_to_token(new_ch);
                }
            } else {
                // Bare "-" identifier
                if (self.mode & SCAN_IDENTS) != 0 {
                    tok = IDENT;
                }
                self.ch = self.char_to_token(next_ch);
            }
        } else {
            match ch_char {
                '\u{FFFF}' => {
                    // EOF already handled
                }
                '"' => {
                    if (self.mode & SCAN_STRINGS) != 0 {
                        self.scan_string('"');
                        tok = STRING;
                    }
                    let ch = self.next();
                    self.ch = self.char_to_token(ch);
                }
                ':' => {
                    if (self.mode & SCAN_KEYWORDS) != 0 {
                        tok = KEYWORD;
                        let new_ch = self.scan_identifier();
                        self.ch = self.char_to_token(new_ch);
                    } else {
                        let ch = self.next();
                        self.ch = self.char_to_token(ch);
                    }
                }
                '.' => {
                    let next_ch = self.next();
                    if Self::is_decimal(next_ch) && (self.mode & SCAN_FLOATS) != 0 {
                        let (new_tok, new_ch) = self.scan_number(next_ch, true, false);
                        tok = new_tok;
                        self.ch = self.char_to_token(new_ch);
                    } else {
                        self.ch = self.char_to_token(next_ch);
                    }
                }
                ';' => {
                    let next_ch = self.next();
                    if (self.mode & SCAN_COMMENTS) != 0 {
                        if (self.mode & SKIP_COMMENTS) != 0 {
                            self.tok_pos = -1;
                            let new_ch = self.scan_comment(next_ch);
                            self.ch = self.char_to_token(new_ch);
                            return self.scan(); // redo
                        }
                        let new_ch = self.scan_comment(next_ch);
                        self.ch = self.char_to_token(new_ch);
                        tok = COMMENT;
                    } else {
                        self.ch = self.char_to_token(next_ch);
                    }
                }
                '¬' => {
                    if (self.mode & SCAN_RAW_STRINGS) != 0 {
                        let new_ch = self.scan_raw_string();
                        self.ch = self.char_to_token(new_ch);
                        tok = RAW_STRING;
                    } else {
                        let ch = self.next();
                        self.ch = self.char_to_token(ch);
                    }
                }
                '~' => {
                    let next_ch = self.next();
                    if (self.mode & SCAN_IDENTS) != 0 {
                        if next_ch == '@' {
                            let ch = self.next();
                            self.ch = self.char_to_token(ch);
                            tok = IDENT;
                        } else {
                            self.ch = self.char_to_token(next_ch);
                        }
                    } else {
                        self.ch = self.char_to_token(next_ch);
                    }
                }
                '#' => {
                    let next_ch = self.next();
                    if (self.mode & SCAN_IDENTS) != 0 {
                        if next_ch == '{' {
                            let ch = self.next();
                            self.ch = self.char_to_token(ch);
                            tok = IDENT;
                        } else {
                            self.ch = self.char_to_token(next_ch);
                        }
                    } else {
                        self.ch = self.char_to_token(next_ch);
                    }
                }
                _ => {
                    let ch = self.next();
                    self.ch = self.char_to_token(ch);
                }
            }
        }

        // End of token text
        self.tok_end = self.src_pos - self.last_char_len;

        tok
    }

    /// Returns the position of the character immediately after
    /// the character or token returned by the last call to next or scan.
    pub fn pos(&self) -> Position {
        let mut pos = Position {
            filename: self.position.filename.clone(),
            offset: self.src_buf_offset + self.src_pos - self.last_char_len,
            line: 0,
            column: 0,
        };

        if self.column > 0 {
            pos.line = self.line;
            pos.column = self.column;
        } else if self.last_line_len > 0 {
            pos.line = self.line - 1;
            pos.column = self.last_line_len;
        } else {
            pos.line = 1;
            pos.column = 1;
        }

        pos
    }

    /// Returns the string corresponding to the most recently scanned token.
    pub fn token_text(&self) -> String {
        if self.tok_pos < 0 {
            return String::new();
        }

        let tok_pos = self.tok_pos as usize;
        let tok_end = if self.tok_end < tok_pos {
            tok_pos
        } else {
            self.tok_end
        };

        if self.tok_buf.is_empty() {
            String::from_utf8_lossy(&self.src_buf[tok_pos..tok_end]).to_string()
        } else {
            let mut result = self.tok_buf.clone();
            result.extend_from_slice(&self.src_buf[tok_pos..tok_end]);
            String::from_utf8_lossy(&result).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_is_valid() {
        let pos = Position {
            filename: "test.lisp".to_string(),
            offset: 0,
            line: 1,
            column: 1,
        };
        assert!(pos.is_valid());

        let invalid_pos = Position {
            filename: "test.lisp".to_string(),
            offset: 0,
            line: 0,
            column: 0,
        };
        assert!(!invalid_pos.is_valid());
    }

    #[test]
    fn test_token_string() {
        assert_eq!(token_string(EOF), "EOF");
        assert_eq!(token_string(IDENT), "Ident");
        assert_eq!(token_string(INT), "Int");
        assert_eq!(token_string(FLOAT), "Float");
        assert_eq!(token_string(STRING), "String");
    }
}
