# Scanner

A scanner and tokenizer for UTF-8-encoded text, 100% compatible with the Go implementation on [github.com/jig/scanner](https://github.com/jig/scanner), that is a Lisp-adapted version of the Go standard library's `text/scanner` package.

Translation is done by Copilot AI.

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)

## Features

- **UTF-8 Support**: Full Unicode support for identifiers and strings
- **Lisp Syntax**: Designed for Lisp-like languages
- **Configurable**: Customizable whitespace, identifier rules, and token modes
- **Position Tracking**: Accurate line and column information
- **Error Handling**: Built-in error reporting and counting
- **BOM Support**: Automatically skips UTF-8 BOM if present

## Supported Tokens

- **Identifiers**: `foo`, `hello-world`, `*host-language*`, `read-string`, `true?`, `def!`, etc.
- **Integers**: Decimal (`42`), octal (`0755`), hexadecimal (`0xFF`), binary (`0b1010`)
- **Floats**: `3.14`, `.5`, `5.`, `1e10`, `1.5e-3`, `0x1.fp+3`
- **Strings**: `"hello"`, with escape sequences `\n`, `\t`, `\x00`, `\u0000`, etc.
- **Raw Strings**: `¬hello¬`, `¬hel¬¬lo¬` (double ¬ to escape)
- **Keywords**: `:a`, `:hello-world`, `:*?`
- **Comments**: `;` and `;;` line comments
- **Special Characters**: `(`, `)`, `[`, `]`, `{`, `}`, `'`, `` ` ``, `~`, `@`, etc.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
scanner = "0.1.1"
```

### Basic Example

```rust
use scanner::{Scanner, token_string, EOF, IDENT, INT};
use std::io::Cursor;

fn main() {
    let src = "(def a 10)";
    let mut scanner = Scanner::init(Cursor::new(src.as_bytes().to_vec()));
    scanner.position.filename = "example.lisp".to_string();

    loop {
        let tok = scanner.scan();
        if tok == EOF {
            break;
        }
        println!("{}: ({}) {}",
            scanner.position,
            token_string(tok),
            scanner.token_text()
        );
    }
}
```

### Output

```
example.lisp:1:1: ("(") (
example.lisp:1:2: (Ident) def
example.lisp:1:6: (Ident) a
example.lisp:1:8: (Int) 10
example.lisp:1:10: (")") )
```

### Custom Configuration

```rust
use scanner::{Scanner, SCAN_IDENTS, SCAN_INTS};
use std::io::Cursor;

let src = "foo 123 bar";
let mut scanner = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

// Only scan identifiers and integers
scanner.set_mode(SCAN_IDENTS | SCAN_INTS);

// Custom whitespace (only space and tab)
scanner.set_whitespace((1 << b' ') | (1 << b'\t'));

// Custom identifier predicate
scanner.set_is_ident_rune(|ch, i| {
    if i == 0 {
        ch.is_alphabetic()
    } else {
        ch.is_alphanumeric()
    }
});
```

## Modes

Configure which tokens to recognize:

- `SCAN_IDENTS`: Identifiers
- `SCAN_INTS`: Integer literals
- `SCAN_FLOATS`: Floating-point literals (includes `SCAN_INTS`)
- `SCAN_STRINGS`: String literals
- `SCAN_KEYWORDS`: Keywords (`:keyword`)
- `SCAN_RAW_STRINGS`: Raw string literals (`¬string¬`)
- `SCAN_COMMENTS`: Comments (`;` and `;;`)
- `SKIP_COMMENTS`: Skip comments (treat as whitespace)
- `LISP_TOKENS`: All of the above (default)

## API Reference

### Types

- `Position`: Represents a source position (filename, offset, line, column)
- `Token`: An `i32` representing a token type or Unicode character
- `Scanner<R: Read>`: The main scanner struct

### Constants

Token types:
- `EOF`, `IDENT`, `INT`, `FLOAT`, `STRING`, `KEYWORD`, `RAW_STRING`, `COMMENT`

Mode bits:
- `SCAN_IDENTS`, `SCAN_INTS`, `SCAN_FLOATS`, `SCAN_STRINGS`, `SCAN_KEYWORDS`, `SCAN_RAW_STRINGS`, `SCAN_COMMENTS`, `SKIP_COMMENTS`, `LISP_TOKENS`

Whitespace:
- `LISP_WHITESPACE`: Default whitespace (space, tab, newline, carriage return)

### Main Methods

- `Scanner::init(src: R) -> Scanner<R>`: Create a new scanner
- `scan() -> Token`: Scan and return the next token
- `next_char() -> Token`: Read next Unicode character
- `peek() -> Token`: Peek at next character without advancing
- `token_text() -> String`: Get text of most recently scanned token
- `pos() -> Position`: Get current position
- `error_count() -> usize`: Get number of errors encountered
- `set_mode(mode: u32)`: Set scanning mode
- `set_whitespace(ws: u64)`: Set whitespace characters
- `set_is_ident_rune<F>(f: F)`: Set custom identifier predicate

## Compatibility with Go Version

This Rust implementation is 100% compatible with the Go version:

- ✅ Same token recognition rules
- ✅ Same position tracking
- ✅ Same error handling behavior
- ✅ Same configurability options
- ✅ All test cases from Go version pass

## Running Tests

```bash
cargo test
```

## Running Examples

```bash
cargo run --example basic
```

## License

This project is licensed under the BSD-3-Clause License - see the LICENSE file for details.

Copyright 2009 The Go Authors. All rights reserved.
Copyright 2022 Jordi Íñigo Griera. All rights reserved.

## Credits

Based on the Go `text/scanner` package and adapted for Lisp syntax by Jordi Íñigo Griera.
Ported to Rust with 100% compatibility maintained.
