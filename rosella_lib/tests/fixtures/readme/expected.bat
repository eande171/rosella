@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
set "rosella_arg1=3"
set "rosella_arg2=4"
call :add
set "rosella_arg1=5"
set "rosella_arg2=6"
call :add
set /a "x=0"
call :rosella_while0
goto :eof

:add
    set "rosella_add.x=!rosella_arg1!"
    set "rosella_add.y=!rosella_arg2!"
    set /a "result=rosella_add.x + rosella_add.y"
    echo(Result: !result!
    goto :eof

:rosella_while0
if !x! LSS 100 goto :rosella_while0_body
goto :eof
:rosella_while0_body
    echo(Current value of x: !x!
    set /a "x=x + 1"
    echo(secret_index_!x!
goto :rosella_while0

