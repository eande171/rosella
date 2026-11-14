<img src="./rosella.svg" width="25%">

# Rosella
Robust Operating System shELL Adaptor

[![CI](https://github.com/eande171/rosella/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eande171/rosella/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/eande171/rosella)](https://github.com/eande171/rosella/releases)
[![License](https://img.shields.io/github/license/eande171/rosella)](LICENSE)

[Website and playground](https://eande171.github.io/rosella/) · [Documentation](https://github.com/eande171/rosella/wiki) · [Releases](https://github.com/eande171/rosella/releases)

## Purpose
**Rosella** is a cross-platform scripting language that compiles down into either Batch (for Windows) or Bash (for Linux). It replaces or hides most of the syntactic and logical quirks of both shells, so scripts read like a C-style language.

## Why Rosella?
Batch and Bash solve the same problems in very different ways. Writing a script twice is slow, and each shell has its own traps: `!` vanishing from text in Batch, a `*` in a variable deleting more files than you meant, `08` being read as an invalid number, and much more. Rosella lets you write the script once, checks it before anything runs, and produces Batch and Bash that behave the same way.

## Features
- **Nice Looking:** No offense but... `while [[ $x -lt 100 ]]; do` just doesn't compare to `while x < 100 { ... }`
- **Cross Platform Compilation by Default:** Install and compile to Batch or Bash from **ANY** supported machine.
- **Lightweight Compiler:** **Rosella's** compiler is TINY (less than 1MB) and requires no dependencies.
- **Checked Before It Runs:** Type mistakes and typos are reported with a line and column before any script is written.
- **Same Behaviour on Both Shells:** The Batch and Bash output of a script print the same text and exit with the same code.
- **Path Formatting:** Paths will automatically format between `\` on Windows and `/` on Linux.

# Getting Started
## Installation
Download the zip for your system from the [latest release](https://github.com/eande171/rosella/releases/latest):

- `rosella-windows.zip` for Windows
- `rosella-linux.zip` for Linux

Each zip holds the `rosella` program. Unzip it somewhere on your `PATH`.

You can also build it yourself with `cargo build --release`, which puts the program in `target/release/`.

## First Rosella Script
Here is a super simple `Hello World` script in the file `hello.rosella`:
```csharp
print("Hello World!");
```

To compile it, run:
```batch
rosella compile -i hello.rosella
```
This produces `hello.bat` on Windows and `hello.sh` on Linux. To write the other one, choose the target and shell yourself:
```batch
rosella compile -i hello.rosella -t linux -s bash
rosella compile -i hello.rosella -t windows -s batch
```

The Bash output:
```bash
#!/bin/bash
printf '%s\n' "Hello World!"
exit 0
```

The Batch output:
```batch
@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
echo(Hello World^^!
exit /b 0
```

## Documentation
The [wiki](https://github.com/eande171/rosella/wiki) covers the full language, the command line, worked examples and the known differences between Batch and Bash. You can also try Rosella in your browser on the [website](https://eande171.github.io/rosella/).

## Larger Example
An example of what more complicated **Rosella** code looks like:
```csharp
fn int add(int a, int b) {
    return a + b;
}

let str name = read("What is your name? ");
print("Hello ", name, "!");

let int total = 0;
for int i in range(1, 6) {
    total = add(total, i);
    print("Running total: ", total);
}

make_dir("backup");
write_file(path("backup", "summary.txt"), concat(name, " counted to ", total));

if exists("backup", "summary.txt") && total > 10 {
    print("Saved the summary");
} else {
    print("Something went wrong");
}

with windows { |> echo Raw shell commands still work; }
with linux { |> echo Raw shell commands still work; }
```

Typing `Eden` at the prompt prints the same thing in both shells:
```
What is your name? Hello Eden!
Running total: 1
Running total: 3
Running total: 6
Running total: 10
Running total: 15
Saved the summary
Raw shell commands still work
```

## License
Rosella is licensed under the [Apache License 2.0](LICENSE).
