use scanner::*;
use std::io::Cursor;

fn main() {
    // Crear exactament els primers tokens fins al string f100
    let f100 = "f".repeat(100);
    
    let mut source = String::new();
    let pattern = " \t%s\n";
    
    // Afegir alguns tokens abans del string f100
    let f100_string = format!(r#""{}""#, f100);
    let tokens_before: Vec<&str> = vec![
        r#"";; line comments""#,
        ";;",
        "a",
        "foobar",
        "0",
        "42",
        r#"" ""#,
        r#""a""#,
        &f100_string,  // Aquest és el problemàtic
    ];
    
    for token in &tokens_before {
        source.push_str(&pattern.replace("%s", token));
    }
    
    println!("Total source length: {}", source.len());
    println!("Last 150 chars: {:?}\n", &source[source.len().saturating_sub(150)..]);
    
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
        println!("Token {}: {} = {:?} (len={})", count, token_string(tok), text, text.len());
        
        if count == tokens_before.len() - 1 {  // El f100 és el penúltim (abans de EOF)
            println!("\n*** This is the f100 string ***");
            let expected = format!(r#""{}""#, f100);
            if text != expected {
                println!("ERROR: Expected {:?} (len={})", expected, expected.len());
                println!("Got {:?} (len={})", text, text.len());
                if text.len() > expected.len() {
                    println!("Extra bytes: {:?}", &text.as_bytes()[expected.len()..]);
                }
            } else {
                println!("OK!");
            }
        }
    }
}
