#![allow(clippy::missing_safety_doc)]

use rosella::{Compiler, Lexer, OS, Parser, Shell};
use std::sync::Mutex;

static OUTPUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

pub fn compile_source(source: &str, shell: Shell) -> Result<String, String> {
    let os = match shell {
        Shell::Batch => OS::Windows,
        Shell::Bash => OS::Linux,
    };
    let tokens = Lexer::new(source).tokenise().map_err(|e| e.to_string())?;
    let ast = Parser::new(tokens).parse().map_err(|e| e.to_string())?;
    Compiler::new(ast, os, shell)
        .compile()
        .map_err(|e| e.to_string())
}

// Browser Interface

#[unsafe(no_mangle)]
pub extern "C" fn alloc(length: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(length);
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(pointer: *mut u8, length: usize) {
    unsafe { drop(Vec::from_raw_parts(pointer, 0, length)) }
}

// Zero Means Batch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile(pointer: *const u8, length: usize, shell: u32) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) };
    let source = String::from_utf8_lossy(bytes);
    let shell = if shell == 0 {
        Shell::Batch
    } else {
        Shell::Bash
    };

    let (succeeded, text) = match compile_source(&source, shell) {
        Ok(output) => (1, output),
        Err(message) => (0, message),
    };
    *OUTPUT.lock().unwrap() = text.into_bytes();
    succeeded
}

#[unsafe(no_mangle)]
pub extern "C" fn output_pointer() -> *const u8 {
    OUTPUT.lock().unwrap().as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn output_length() -> usize {
    OUTPUT.lock().unwrap().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_for_both_shells() {
        let bash = compile_source("print(\"Hi\");", Shell::Bash).unwrap();
        assert!(bash.contains("printf '%s\\n' \"Hi\""));
        let batch = compile_source("print(\"Hi\");", Shell::Batch).unwrap();
        assert!(batch.contains("echo(Hi\r\n"));
    }

    #[test]
    fn errors_come_back_as_text() {
        let message = compile_source("let int x = \"a\";", Shell::Bash).unwrap_err();
        assert_eq!(
            message,
            "line 1, column 1: 'x' is int and cannot take a str value"
        );
    }

    #[test]
    fn source_passes_through_memory() {
        let source = "print(1);";
        unsafe {
            let pointer = alloc(source.len());
            std::ptr::copy_nonoverlapping(source.as_ptr(), pointer, source.len());
            assert_eq!(compile(pointer, source.len(), 1), 1);
            dealloc(pointer, source.len());
        }
        let output = String::from_utf8(OUTPUT.lock().unwrap().clone()).unwrap();
        assert!(output.contains("printf '%s\\n' \"1\""));
    }
}
