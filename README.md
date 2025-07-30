# Rosella
Robust Operating System shELL Adaptor

## Purpose
**Rosella** is designed to be a cross-platform scripting language that transpiles/compiles down into either Batch (for Windows) or Bash (for Linux). It also replaces or abstracts a majority of the syntactic and logical quirks from both Bash and Batch, making it resemble a C-style language.

## Features
- **Nice Looking:** No offense but... `while [[ $x -lt 100]]; do` just doesn't compare to `while int(x < 100) { ... }`
- **Cross Platform Compilation by Default:** Install and compile to Batch or Bash from **ANY** supported machine.
- **Lightweight Compiler:** **Rosella's** compiler is TINY (typically less than 1MB) and requires no dependencies.
- **Path Formatting:** Paths will automatically format between `\` on Windows and `/` on Linux

# Getting Started
## Installation
Installing Rosella is as simple as downloading the latest binary from [releases](https://github.com/eande171/rosella/releases), or alternatively, compiling it yourself with `cargo build`.

## First Rosella Script
Here is a super simple `Hello World` script in the file `hello.rosella`:
```javascript
print("Hello World!")
```

To compile this, run the following. Ensure `rosella` points to the location of the compiler: 
```batch
rosella compile -i hello.rosella
```
This will produce `hello.bat` on Windows and `hello.sh` on Linux by default. 

## Documentation
Given the nature of the transpiler, **Rosella** *does* have some syntactic quirks that are better explained in proper documentation found [here](https://github.com/eande171/rosella/wiki).

## Larger Example
An example of what more complicated **Rosella** code looks like:
```javascript
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
    print("secret_index_", x)
}
```

