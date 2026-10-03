@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=2"
if !x! EQU 1 (
    echo(one
) else (
    set /a "rosella_temp0=x + 1"
    if !rosella_temp0! EQU 3 (
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
exit /b 0
