use scanner::{Scanner, token_string, EOF};

fn main() {
    // Example 1: Basic scanning
    println!("Example 1: Basic Lisp code scanning");
    println!("====================================");
    let src = "; This is scanned code
(def a '(list 10 3.14 -30 \"hello\" ¬hello¬ \"hel\\\"lo\" ¬hel¬¬lo¬ :a))
";

    let mut s = Scanner::init(src.as_bytes());
    s.position.filename = "example".to_string();

    loop {
        let tok = s.scan();
        if tok == EOF {
            break;
        }
        println!("{}: ({}) {}", s.position, token_string(tok), s.token_text());
    }

    println!("\nExample 2: Actual Lisp code with macros");
    println!("========================================");
    let src2 = "; This is scanned code
(def _iter->
	(fn [acc form]
		(if (list? form)
		`(~(first form) ~acc ~@(rest form))
		(list form acc))))
	";

    let mut s2 = Scanner::init(src2.as_bytes());
    s2.position.filename = "actual-code".to_string();

    loop {
        let tok = s2.scan();
        if tok == EOF {
            break;
        }
        println!("{}: ({}) {}", s2.position, token_string(tok), s2.token_text());
    }
}
