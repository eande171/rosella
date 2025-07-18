use rosella::Lexer;

fn main() {
    let mut lexer = Lexer::new(
        r#"
        fn add(x, y) {
            let result = x + y;
            |> "echo" result;
        }
        "#
    );

    let tokens = lexer.tokenise().unwrap();

    println!("{:?}", tokens);
}