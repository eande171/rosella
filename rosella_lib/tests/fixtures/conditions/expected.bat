@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=2"
set /a "rosella_temp0=x"
if !rosella_temp0! EQU 1 (
    echo(one
) else (
    set /a "rosella_temp1=x + 1"
    if !rosella_temp1! EQU 3 (
        echo(two
    ) else (
        echo(other
    )
)
set "name=Bob"
if not "!name!"=="Alice" (
    echo(not Alice
) else (
    echo(Alice
)
if "!name!"=="Bob" (
    echo(Bob
)
if "apple" LSS "banana" (
    echo(apple first
)
if not exist "missing.txt" (
    echo(no file
)
set /a "rosella_temp2=x"
if !rosella_temp2! GTR 1 (
    echo(explicit
)
