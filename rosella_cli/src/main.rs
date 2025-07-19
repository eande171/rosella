use rosella::Lexer;
use rosella::Parser;

fn main() {
    /*let mut lexer = Lexer::new(
        r#"
        fn add(x, y) {
            let result = x + y;
            |> "echo" result;
        }
        "#
    );*/

    /*let mut lexer = Lexer::new(
  r#"
        let result = x + y;
        let string = "this is a string";
        "#
    );*/

    let mut lexer = Lexer::new(
        r#"
        fn add(x, y) {
            let result = x + y;
        }

        add(1, 2);
        add(3, 4);
        "#
    );

    let tokens = lexer.tokenise().unwrap();

    println!("{:?}", tokens);

    let mut parser = Parser::new(tokens);

    let ast = match parser.parse() {
        Ok(result) => result,
        Err(e) => panic!("{}", e)
    };

    println!("AST: {:?}", ast);
}