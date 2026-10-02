@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_arg1=3"
set "rosella_arg2=4"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_arg1=5"
set "rosella_arg2=6"
call :add
if defined rosella_exit exit /b !rosella_exit!
set /a "x=0"
call :rosella_while1
if defined rosella_exit exit /b !rosella_exit!
goto :eof

:add
    set "rosella_add_x=!rosella_arg1!"
    set "rosella_add_y=!rosella_arg2!"
    set /a "result=rosella_add_x + rosella_add_y"
    echo(Result: !result!
    goto :eof

:rosella_while1
set /a "rosella_temp0=x"
if !rosella_temp0! LSS 100 goto :rosella_while1_body
goto :eof
:rosella_while1_body
    echo(Current value of x: !x!
    set /a "x=x + 1"
    echo(secret_index_!x!
goto :rosella_while1

