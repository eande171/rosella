use rosella::Lexer;
use rosella::Parser;
use rosella::Compiler;
use rosella::Shell;
use rosella::OS;

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

    /*let mut lexer = Lexer::new(
        r#"
        fn add(x, y) {
            let result = x + y;
        }

        if int(x > 0) {
            add(1, 2);
        }
        else if int(x < 0) {
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
    );*/
    let mut lexer = Lexer::new(
        r#"
        let int x = 10;
        let int y = 20;

        let int result = x + y;
        let str string = "this is a string";

        let str appended_string = string + string2;

        with windows {
            if int(x > 0) {
                let int x = 10;
                let int y = 20;
            }
            else if int(x < 0) {
                let int result = x + y;
                let str string = "this is a string";
            }
            else {
                let str string2 = "this is another string";
                let str appended_string = string + string2;
            }
        }

        with linux {
            while int(x < 100) {
                let int x = x + 1;
            }

            fn add(x, y) {
                let int result = x + y;
            }

            add(1, 2);
        }

        |> echo "Hello World";
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

    let output = match Compiler::new(ast, OS::Linux, Shell::Bash)
        .compile() {
            Ok(result) => result,
            Err(e) => panic!("{}", e)
        };

    println!("Compiled Output:\n{}", output);
}