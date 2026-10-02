@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=Bob Smith"
set "rosella_arg2=Hi^!"
call :greet
if defined rosella_exit exit /b !rosella_exit!
set "n=Ann & Lee (100%%)"
set "rosella_arg1=!n!"
set "rosella_arg2=Hello"
call :greet
if defined rosella_exit exit /b !rosella_exit!
set /a "x=5"
set "rosella_arg1=1"
call :bump
if defined rosella_exit exit /b !rosella_exit!
echo(outside: !x!
goto :eof

:greet
    set "rosella_greet_name=!rosella_arg1!"
    set "rosella_greet_greeting=!rosella_arg2!"
    echo(!rosella_greet_greeting! !rosella_greet_name!.
    goto :eof

:bump
    set "rosella_bump_x=!rosella_arg1!"
    set /a "rosella_bump_x=rosella_bump_x + 100"
    echo(inside: !rosella_bump_x!
    goto :eof

