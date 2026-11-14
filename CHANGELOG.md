# Changelog

## 0.2.0

This release rewrites the compiler and changes the language. Scripts written for 0.1.0 need small updates, listed under Breaking Changes.

### Breaking Changes

- Every statement ends with a semicolon, including function calls like `print("Hi");`.
- Conditions are written directly: `if x < 10 { }` instead of `if int(x < 10) { }`. The `int()`, `str()` and `file()` wrappers are gone.
- Function parameters need a type: `fn greet(str name, int times) { }`.
- Variables are changed with `x = x + 1;`. `let` is only for declaring a variable once.
- `read()` returns the text that was typed: `let str name = read("Name: ");`.
- The short aliases `echo`, `mkdir`, `rmdir`, `del`, `cp` and `mv` are gone. Use `print`, `make_dir`, `remove_dir`, `remove`, `copy` and `move`.
- Numbers are whole numbers that fit in 32 bits, the range Batch supports.
- The compiler binary is called `rosella` instead of `rosella_cli`.

### Language

- Functions can return a value: `fn int add(int a, int b) { return a + b; }`.
- `for` loops over a range of numbers, `for int i in range(0, 10) { }`, or over files, `for str file in files("logs", "*.txt") { }`.
- `break` and `continue` inside loops.
- `&&`, `||`, `!` and `%` in conditions and arithmetic. The right side of `&&` and `||` only runs when needed.
- `else if` chains.
- Script arguments with `arg(1)` and `arg_count()`.
- Environment variables with `env("HOME")` and `set_env("MODE", "release")`.
- Run programs with `run("git", "pull")`, which returns the exit code, and read the first line they print with `run_output("git", "branch", "--show-current")`.
- Text functions: `length`, `slice`, `replace`, `upper`, `lower` and `contains`.
- `number()` turns text into a whole number and `is_number()` checks whether it is one.
- `is_dir()`, `is_file()`, `random()`, `sleep()` and `script_dir()`.

### Compiler

- Errors show the line and column, and explain what is wrong.
- The compiler checks types before writing any output, so mistakes are caught at compile time.
- Bash and Batch output behave the same way for the same script. Known exceptions are listed on the wiki.
- Batch output keeps characters such as `!`, `%`, `^` and `&` intact in text, file names and program output.
- Batch output is shorter and easier to read.
- `remove()` only removes files and `remove_dir()` only removes folders, in both shells.
- Names that would clash with shell commands or shell variables are rejected with a clear message.
- The release binary is smaller.

## 0.1.0

The first release of Rosella.
