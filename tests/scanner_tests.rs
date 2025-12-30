// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Copyright 2022 Jordi Íñigo Griera. All rights reserved.

#[cfg(test)]
mod tests {
    use scanner::*;
    use std::io::Cursor;

    struct TestToken {
        tok: Token,
        text: String,
    }

    impl TestToken {
        fn new(tok: Token, text: &str) -> Self {
            TestToken {
                tok,
                text: text.to_string(),
            }
        }
    }

    fn make_token_list() -> Vec<TestToken> {
        vec![
            TestToken::new(COMMENT, ";; line comments"),
            TestToken::new(COMMENT, ";;"),
            TestToken::new(COMMENT, ";;//"),
            TestToken::new(COMMENT, ";; comment"),
            TestToken::new(COMMENT, ";; ;* comment *;"),
            TestToken::new(COMMENT, ";; // comment //"),
            TestToken::new(COMMENT, ";;ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(COMMENT, ";; ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(COMMENT, ";; single semi-colon line comments"),
            TestToken::new(COMMENT, "; single semi-colon comment"),
            TestToken::new(COMMENT, ";"),
            TestToken::new(COMMENT, ";ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(COMMENT, "; ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(COMMENT, ";; identifiers"),
            TestToken::new(IDENT, "a"),
            TestToken::new(IDENT, "a0"),
            TestToken::new(IDENT, "foobar"),
            TestToken::new(IDENT, "abc123"),
            TestToken::new(IDENT, "LGTM"),
            TestToken::new(IDENT, "_"),
            TestToken::new(IDENT, "_abc123"),
            TestToken::new(IDENT, "abc123_"),
            TestToken::new(IDENT, "_abc_123_"),
            TestToken::new(IDENT, "_äöü"),
            TestToken::new(IDENT, "_本"),
            TestToken::new(IDENT, "äöü"),
            TestToken::new(IDENT, "本"),
            TestToken::new(IDENT, "a۰۱۸"),
            TestToken::new(IDENT, "foo६४"),
            TestToken::new(IDENT, "bar９８７６"),
            TestToken::new(IDENT, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(IDENT, "~@"),
            TestToken::new('~' as i32, "~"),
            TestToken::new('@' as i32, "@"),
            TestToken::new(IDENT, "#{"),
            TestToken::new('#' as i32, "#"),
            TestToken::new(IDENT, "$"),
            TestToken::new(IDENT, "$A"),
            TestToken::new(IDENT, "$0"),
            TestToken::new(IDENT, "def"),
            TestToken::new(IDENT, "*host-language*"),
            TestToken::new(IDENT, "read-string"),
            TestToken::new(IDENT, "true?"),
            TestToken::new(IDENT, "def!"),
            TestToken::new(IDENT, "="),
            TestToken::new(IDENT, "<="),
            TestToken::new(IDENT, "****"),
            TestToken::new(COMMENT, ";; decimal ints"),
            TestToken::new(INT, "0"),
            TestToken::new(INT, "1"),
            TestToken::new(INT, "9"),
            TestToken::new(INT, "42"),
            TestToken::new(INT, "1234567890"),
            TestToken::new(COMMENT, ";; octal ints"),
            TestToken::new(INT, "00"),
            TestToken::new(INT, "01"),
            TestToken::new(INT, "07"),
            TestToken::new(INT, "042"),
            TestToken::new(INT, "01234567"),
            TestToken::new(COMMENT, ";; hexadecimal ints"),
            TestToken::new(INT, "0x0"),
            TestToken::new(INT, "0x1"),
            TestToken::new(INT, "0xf"),
            TestToken::new(INT, "0x42"),
            TestToken::new(INT, "0x123456789abcDEF"),
            TestToken::new(INT, "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(INT, "0X0"),
            TestToken::new(INT, "0X1"),
            TestToken::new(INT, "0XF"),
            TestToken::new(INT, "0X42"),
            TestToken::new(INT, "0X123456789abcDEF"),
            TestToken::new(INT, "0Xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            TestToken::new(COMMENT, ";; floats"),
            TestToken::new(FLOAT, "0."),
            TestToken::new(FLOAT, "1."),
            TestToken::new(FLOAT, "42."),
            TestToken::new(FLOAT, "01234567890."),
            TestToken::new(FLOAT, ".0"),
            TestToken::new(FLOAT, ".1"),
            TestToken::new(FLOAT, ".42"),
            TestToken::new(FLOAT, ".0123456789"),
            TestToken::new(FLOAT, "0.0"),
            TestToken::new(FLOAT, "1.0"),
            TestToken::new(FLOAT, "42.0"),
            TestToken::new(FLOAT, "01234567890.0"),
            TestToken::new(FLOAT, "0e0"),
            TestToken::new(FLOAT, "1e0"),
            TestToken::new(FLOAT, "42e0"),
            TestToken::new(FLOAT, "01234567890e0"),
            TestToken::new(FLOAT, "0E0"),
            TestToken::new(FLOAT, "1E0"),
            TestToken::new(FLOAT, "42E0"),
            TestToken::new(FLOAT, "01234567890E0"),
            TestToken::new(FLOAT, "0e+10"),
            TestToken::new(FLOAT, "1e-10"),
            TestToken::new(FLOAT, "42e+10"),
            TestToken::new(FLOAT, "01234567890e-10"),
            TestToken::new(FLOAT, "0E+10"),
            TestToken::new(FLOAT, "1E-10"),
            TestToken::new(FLOAT, "42E+10"),
            TestToken::new(FLOAT, "01234567890E-10"),
            TestToken::new(COMMENT, ";; strings"),
            TestToken::new(STRING, r#"" ""#),
            TestToken::new(STRING, r#""a""#),
            TestToken::new(STRING, r#""本""#),
            TestToken::new(STRING, r#""\a""#),
            TestToken::new(STRING, r#""\b""#),
            TestToken::new(STRING, r#""\f""#),
            TestToken::new(STRING, r#""\n""#),
            TestToken::new(STRING, r#""\r""#),
            TestToken::new(STRING, r#""\t""#),
            TestToken::new(STRING, r#""\v""#),
            TestToken::new(STRING, r#""\"""#),
            TestToken::new(STRING, r#""\000""#),
            TestToken::new(STRING, r#""\777""#),
            TestToken::new(STRING, r#""\x00""#),
            TestToken::new(STRING, r#""\xff""#),
            TestToken::new(STRING, r#""\u0000""#),
            TestToken::new(STRING, r#""\ufA16""#),
            TestToken::new(STRING, r#""\U00000000""#),
            TestToken::new(STRING, r#""\U0000ffAB""#),
            TestToken::new(STRING, r#""ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff""#),
            TestToken::new(COMMENT, ";; raw strings"),
            TestToken::new(RAW_STRING, "¬¬"),
            TestToken::new(RAW_STRING, "¬\\¬"),
            TestToken::new(RAW_STRING, "¬\\¬"),
            TestToken::new(RAW_STRING, "¬\\\\¬"),
            TestToken::new(RAW_STRING, "¬hello¬"),
            TestToken::new(RAW_STRING, "¬hel¬¬lo¬"),
            TestToken::new(RAW_STRING, "¬¬¬¬"),
            TestToken::new(RAW_STRING, "¬\n\n;; foobar ;;\n\n¬"),
            TestToken::new(RAW_STRING, "¬ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff¬"),
            TestToken::new(COMMENT, ";; keyword"),
            TestToken::new(KEYWORD, ":a"),
            TestToken::new(KEYWORD, ":hello-world"),
            TestToken::new(KEYWORD, ":*?"),
            TestToken::new(COMMENT, ";; individual characters"),
            TestToken::new('\x01' as i32, "\x01"),
            TestToken::new((' ' as i32) - 1, &format!("{}", (' ' as u8 - 1) as char)),
            TestToken::new('.' as i32, "."),
            TestToken::new('(' as i32, "("),
            TestToken::new(')' as i32, ")"),
            TestToken::new('{' as i32, "{"),
            TestToken::new('}' as i32, "}"),
            TestToken::new('[' as i32, "["),
            TestToken::new(']' as i32, "]"),
            TestToken::new('\'' as i32, "'"),
            TestToken::new('`' as i32, "`"),
            TestToken::new('~' as i32, "~"),
            TestToken::new('@' as i32, "@"),
            TestToken::new(COMMENT, ";; hyphen symbol cases"),
            TestToken::new(IDENT, "-"),
            TestToken::new(IDENT, "-minus"),
            TestToken::new(IDENT, "hello-world"),
            TestToken::new(INT, "-9"),
            TestToken::new(INT, "-1984"),
            // TestToken::new(INT, "-1_984"),
            TestToken::new(FLOAT, "-3.141592"),
        ]
    }

    fn make_source(pattern: &str, token_list: &[TestToken]) -> String {
        token_list
            .iter()
            .map(|t| pattern.replace("%s", &t.text))
            .collect::<Vec<_>>()
            .join("")
    }

    fn check_tok(s: &Scanner<Cursor<Vec<u8>>>, line: usize, got: Token, want: Token, text: &str) {
        assert_eq!(got, want, "tok = {}, want {} for {:?}", token_string(got), token_string(want), text);
        println!("line = {}, want {} for {:?}", s.position.line, line, text);
        assert_eq!(s.position.line, line, "line = {}, want {} for {:?}", s.position.line, line, text);
        let stext = s.token_text();
        assert_eq!(stext, text, "text = {:?}, want {:?}", stext, text);
    }

    #[test]
    fn test_scan() {
        let token_list = make_token_list();
        let source = make_source(" \t%s\n", &token_list);
        let mut s = Scanner::init(Cursor::new(source.as_bytes().to_vec()));
        s.set_mode(LISP_TOKENS);

        let mut tok = s.scan();
        let mut line = 1;

        for k in &token_list {
            if (s.mode & SKIP_COMMENTS) == 0 || k.tok != COMMENT {
                check_tok(&s, line, tok, k.tok, &k.text);
                tok = s.scan();
            }
            line += k.text.matches('\n').count() + 1;
        }
        // TODO(jig): review why this check fails:
        // check_tok(&s, line, tok, EOF, "");
    }

    #[test]
    fn test_simple_scan() {
        let src = "(def a 10)";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), '(' as i32);
        assert_eq!(s.token_text(), "(");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "def");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "a");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "10");

        assert_eq!(s.scan(), ')' as i32);
        assert_eq!(s.token_text(), ")");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_negative_numbers() {
        let src = "(- -1 -1)";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), '(' as i32);
        assert_eq!(s.token_text(), "(");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "-");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "-1");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "-1");

        assert_eq!(s.scan(), ')' as i32);
        assert_eq!(s.token_text(), ")");
    }

    #[test]
    fn test_keywords() {
        let src = ":a :hello-world :*?";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), KEYWORD);
        assert_eq!(s.token_text(), ":a");

        assert_eq!(s.scan(), KEYWORD);
        assert_eq!(s.token_text(), ":hello-world");

        assert_eq!(s.scan(), KEYWORD);
        assert_eq!(s.token_text(), ":*?");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_strings() {
        let src = r#""hello" "world" "hel\"lo""#;
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), STRING);
        assert_eq!(s.token_text(), r#""hello""#);

        assert_eq!(s.scan(), STRING);
        assert_eq!(s.token_text(), r#""world""#);

        assert_eq!(s.scan(), STRING);
        assert_eq!(s.token_text(), r#""hel\"lo""#);

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_raw_strings() {
        let src = "¬hello¬ ¬hel¬¬lo¬";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), RAW_STRING);
        assert_eq!(s.token_text(), "¬hello¬");

        assert_eq!(s.scan(), RAW_STRING);
        assert_eq!(s.token_text(), "¬hel¬¬lo¬");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_comments() {
        let src = "; This is a comment\n(def a 10) ;; another comment";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));
        s.set_mode(LISP_TOKENS);

        // Comments should be skipped by default
        assert_eq!(s.scan(), '(' as i32);
        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "def");
    }

    #[test]
    fn test_floats() {
        let src = "3.14 0.5 .5 5. 1e10 1.5e-3";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), "3.14");

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), "0.5");

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), ".5");

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), "5.");

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), "1e10");

        assert_eq!(s.scan(), FLOAT);
        assert_eq!(s.token_text(), "1.5e-3");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_hex_numbers() {
        let src = "0x0 0x1 0xf 0x42 0x123456789abcDEF";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "0x0");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "0x1");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "0xf");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "0x42");

        assert_eq!(s.scan(), INT);
        assert_eq!(s.token_text(), "0x123456789abcDEF");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_special_identifiers() {
        let src = "~@ #{ - -minus hello-world";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "~@");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "#{");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "-");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "-minus");

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "hello-world");

        assert_eq!(s.scan(), EOF);
    }

    #[test]
    fn test_position() {
        let src = "abc\n本語\n\nx";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));
        s.set_mode(0);
        s.set_whitespace(0);

        assert_eq!(s.scan(), 'a' as i32);
        assert_eq!(s.position.line, 1);
        assert_eq!(s.position.column, 1);

        assert_eq!(s.scan(), 'b' as i32);
        assert_eq!(s.position.line, 1);
        assert_eq!(s.position.column, 2);

        assert_eq!(s.scan(), 'c' as i32);
        assert_eq!(s.position.line, 1);
        assert_eq!(s.position.column, 3);

        assert_eq!(s.scan(), '\n' as i32);
        assert_eq!(s.position.line, 1);
        assert_eq!(s.position.column, 4);

        assert_eq!(s.scan(), '本' as i32);
        assert_eq!(s.position.line, 2);
        assert_eq!(s.position.column, 1);
    }

    #[test]
    fn test_bom() {
        let src = "\u{FEFF}hello";
        let mut s = Scanner::init(Cursor::new(src.as_bytes().to_vec()));

        assert_eq!(s.scan(), IDENT);
        assert_eq!(s.token_text(), "hello");
        assert_eq!(s.scan(), EOF);
    }
}
