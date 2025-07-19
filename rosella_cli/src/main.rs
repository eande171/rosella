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

        if (x > 0) {
            add(1, 2);
        }
        else if (x < 0) {
            add(2, 3);
        }
        else {
            add(3, 4);
        }

        with windows {
            print("Using Windows");
        }
        with linux {
            print("Using Linux");
        }
        with macos {
            print("Using MacOS");
        }

        |> "echo Hello, World!";
        |> "echo This is a test" variable "-t 100";
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