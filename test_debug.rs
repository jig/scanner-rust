use scanner::*;
use std::io::Cursor;

fn main() {
    let f100 = "f".repeat(100);
    
    // Simular el que fa make_source amb el patró " \t%s\n"
    let pattern = " \t%s\n";
    let token_text = format!(r#""{}""#, f100);  // El text del token ja inclou les cometes
    let source_line = pattern.replace("%s", &token_text);
    
    println!("Token text: {:?}", token_text);
    println!("Source line: {:?}", source_line);
    println!("Source line length: {}", source_line.len());
    
    let mut s = Scanner::init(Cursor::new(source_line.as_bytes().to_vec()));
    s.set_mode(LISP_TOKENS);
    
    let tok = s.scan();
    println!("\nScanned token: {} ({})", token_string(tok), tok);
    let scanned_text = s.token_text();
    println!("Scanned text: {:?}", scanned_text);
    println!("Scanned text length: {}", scanned_text.len());
    println!("Expected: {:?}", token_text);
    println!("Match: {}", scanned_text == token_text);
    
    if scanned_text != token_text {
        println!("\nDifference:");
        println!("Expected length: {}", token_text.len());
        println!("Actual length: {}", scanned_text.len());
        if scanned_text.len() > token_text.len() {
            println!("Extra bytes: {:?}", &scanned_text.as_bytes()[token_text.len()..]);
        }
    }
}
