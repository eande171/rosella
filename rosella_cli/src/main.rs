use rosella::Lexer;
use rosella::Parser;
use rosella::Compiler;
use rosella::Shell;
use rosella::OS;

use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(
    version = "0.1.0",
    about = "A Command Line Interface for the Rosella programming language."

)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
        #[arg(short, long, value_name = "FILE", value_parser = clap::value_parser!(PathBuf))]
        input: PathBuf,

        #[arg(short, long, value_name = "FILE", value_parser = clap::value_parser!(PathBuf))]
        output: Option<PathBuf>,

        #[arg(long, value_enum)]
        os: Option<TargetOS>,

        #[arg(short, long, value_enum)]
        shell: Option<TargetShell>,
    }
}

#[derive(ValueEnum, Debug, Clone)]
enum TargetOS {
    Windows,
    Linux,
}

#[derive(ValueEnum, Debug, Clone)]
enum TargetShell {
    Batch,
    Bash,
}

fn main() {
    let cli = Cli::parse();
    let current_os = std::env::consts::OS;

    match &cli.command {
        Commands::Compile { 
            input, 
            output, 
            os, 
            shell 
        } => {
            let input_content = match std::fs::read_to_string(input) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading input file: {}", e);
                    return;
                }
            };

            let target_os = match os {
                Some(os) => {
                    match os {
                        TargetOS::Windows => OS::Windows,
                        TargetOS::Linux => OS::Linux,
                    }
                }
                None => {
                    if current_os == "windows" {
                        OS::Windows
                    } else {
                        OS::Linux
                    }
                }
            };

            let target_shell = match shell {
                Some(shell) => {
                    match shell {
                        TargetShell::Batch => Shell::Batch,
                        TargetShell::Bash => Shell::Bash,
                    }
                }
                None => {
                    if current_os == "windows" {
                        Shell::Batch
                    } else {
                        Shell::Bash
                    }
                }
            };

            let output = match output {
                Some(path) => path.clone(),
                None => {
                    let mut output_path = input.clone();
                    output_path.set_extension(match target_shell {
                        Shell::Batch => "bat",
                        Shell::Bash => "sh",
                    });
                    output_path
                }
            };

            if target_os == OS::Linux && target_shell == Shell::Batch {
                eprintln!("Batch shell is not supported on Linux.");
                return;
            }

            println!("Compiling {} for {:?} using {:?} shell", input.display(), target_os, target_shell);

            let mut lexer = Lexer::new(&input_content);
            let tokens = match lexer.tokenise() {
                Ok(tokens) => tokens,
                Err(e) => {
                    eprintln!("Error during tokenization: {}", e);
                    return;
                }
            };

            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("Error during parsing: {}", e);
                    return;
                }
            };

            let output_content = match Compiler::new(ast, target_os, target_shell).compile() {
                Ok(output) => output,
                Err(e) => {
                    eprintln!("Error during compilation: {}", e);
                    return;
                }
            };

            if let Err(e) = std::fs::write(&output, output_content) {
                eprintln!("Error writing output file: {}", e);
            } else {
                println!("Compilation successful! Output written to {}", output.display());
            }
        }
    }

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
    /*let mut lexer = Lexer::new(
        r#"
        fn add(x, y) {
            let int result = x + y;
            print("Result: ", result)
        }

        add(1, 2)
        add(3, 4)
        add(5, 6)

        let int x = 0;

        while int(x < 100) {
            print("Current value of x: ", x)
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

        remove_dir("super", "directory")

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

    println!("Compiled Output:\n{}", output);*/
}