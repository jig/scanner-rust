use scanner::*;
use std::io::Cursor;

fn make_token_list() -> Vec<(&'static str, &'static str)> {
    let mut tokens = vec![];
    tokens.push((";; line comments", "comment"));
    tokens.push((";;", "comment"));
    tokens.push((";;//", "comment"));
    tokens.push((";; comment", "comment"));
    // ... afegir més tokens aquí, però per simplicitat, només posaré els crítics

    // Saltar directament prop del f100
    for _ in 0..130 {  // Generar 130 tokens simples abans
        tokens.push(("a", "ident"));
    }

    tokens
}

fn main() {
    let f100 = "f".repeat(100);
    let mut source = String::new();
    let pattern = " \t%s\n";

    // Generar molts tokens abans
    for _ in 0..130 {
        source.push_str(&pattern.replace("%s", "a"));
    }

    // Ara afegir el string f100
    let f100_token = format!(r#""{}""#, f100);
    source.push_str(&pattern.replace("%s", &f100_token));

    println!("Total source length: {}", source.len());
    println!("Number of lines: {}", source.lines().count());

    let mut s = Scanner::init(Cursor::new(source.as_bytes().to_vec()));
    s.set_mode(LISP_TOKENS);

    let mut count = 0;
    loop {
        let tok = s.scan();
        if tok == EOF {
            break;
        }
        count += 1;
        let text = s.token_text();

        if count == 131 {  // El token f100
            println!("\nToken {}: {} = (len={})", count, token_string(tok), text.len());
            let expected = f100_token;
            if text != expected {
                println!("ERROR!");
                println!("Expected: {:?} (len={})", expected, expected.len());
                println!("Got: {:?} (len={})", text, text.len());
                if text.len() > expected.len() {
                    println!("Extra bytes: {:?}", &text.as_bytes()[expected.len()..]);
                }
            } else {
                println!("OK!");
            }
            break;
        }
    }

    println!("Total tokens scanned: {}", count);
}