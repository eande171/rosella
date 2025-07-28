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
        fn add(x, y) {
            let int result = x + y;
            |> echo "$x + $y = $result";
        }

        add(1, 2)
        add(3, 4)
        add(5, 6)

        let int x = 0;

        while int(x < 100) {
            |> echo "Current value of x: $x";
            let int x = x + 1;
            print("home and ", x)
        }

        copy(
            path("origin", "path"), 
            path("copy", path)
        )

        move(
            path("source", "file.txt"),
            path("destination", "file.txt")
        )

        remove("super", "directory")

        if file(exists("file.txt")) {
            print("File exists!")
        } else {
            print("File does not exist!")
        }

        read("What is your name?", name)
        print("Hello, ", name, "!")

        exit(0)
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

    let output = match Compiler::new(ast, OS::Windows, Shell::Batch)
        .compile() {
            Ok(result) => result,
            Err(e) => panic!("{}", e)
        };

    println!("Compiled Output:\n{}", output);
}