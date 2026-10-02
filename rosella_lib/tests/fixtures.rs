use std::fs;
use std::path::Path;

use rosella::{Compiler, Lexer, OS, Parser, Shell};

// UPDATE_FIXTURES Regenerates Expected Output
#[test]
fn compiled_output_matches_fixtures() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let update = std::env::var_os("UPDATE_FIXTURES").is_some();
    let mut failures = Vec::new();

    let mut fixtures: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("input.rosella").is_file())
        .collect();
    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "no fixtures found in {}",
        directory.display()
    );

    for fixture in fixtures {
        let source = fs::read_to_string(fixture.join("input.rosella")).unwrap();

        for (os, shell, file) in [
            (OS::Linux, Shell::Bash, "expected.sh"),
            (OS::Windows, Shell::Batch, "expected.bat"),
        ] {
            let expected_path = fixture.join(file);
            let actual = compile(&source, os, shell).unwrap_or_else(|message| {
                panic!("{} failed to compile: {}", fixture.display(), message)
            });

            if update {
                fs::write(&expected_path, &actual).unwrap();
                continue;
            }

            // Ignore Line Endings
            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_default()
                .replace("\r\n", "\n");
            if expected != actual.replace("\r\n", "\n") {
                failures.push(expected_path.display().to_string());
            }
        }
    }

    assert!(
        failures.is_empty(),
        "output differs from fixtures:\n{}",
        failures.join("\n")
    );
}

fn compile(source: &str, os: OS, shell: Shell) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenise().map_err(|e| e.to_string())?;
    let ast = Parser::new(tokens).parse().map_err(|e| e.to_string())?;
    Compiler::new(ast, os, shell)
        .compile()
        .map_err(|e| e.to_string())
}
