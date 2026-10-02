use rosella::{Compiler, Lexer, OS, Parser, Shell};

use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

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

        #[arg(short, long, value_enum)]
        target: Option<TargetOS>,

        #[arg(short, long, value_enum)]
        shell: Option<TargetShell>,
    },
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let current_os = std::env::consts::OS;

    match &cli.command {
        Commands::Compile {
            input,
            output,
            target,
            shell,
        } => {
            let input_content = match std::fs::read_to_string(input) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading input file: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let target_os = match target {
                Some(os) => match os {
                    TargetOS::Windows => OS::Windows,
                    TargetOS::Linux => OS::Linux,
                },
                None => {
                    if current_os == "windows" {
                        OS::Windows
                    } else {
                        OS::Linux
                    }
                }
            };

            let target_shell = match shell {
                Some(shell) => match shell {
                    TargetShell::Batch => Shell::Batch,
                    TargetShell::Bash => Shell::Bash,
                },
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
                return ExitCode::FAILURE;
            }

            println!(
                "Compiling {} for {:?} using {:?} shell",
                input.display(),
                target_os,
                target_shell
            );

            let mut lexer = Lexer::new(&input_content);
            let tokens = match lexer.tokenise() {
                Ok(tokens) => tokens,
                Err(e) => {
                    eprintln!("Error during tokenization: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("Error during parsing: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            let output_content = match Compiler::new(ast, target_os, target_shell).compile() {
                Ok(output) => output,
                Err(e) => {
                    eprintln!("Error during compilation: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = std::fs::write(&output, output_content) {
                eprintln!("Error writing output file: {}", e);
                return ExitCode::FAILURE;
            }

            println!(
                "Compilation successful! Output written to {}",
                output.display()
            );
            ExitCode::SUCCESS
        }
    }
}
