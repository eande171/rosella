@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=(1 + 2) * 3"
set /a "y=x - (-4)"
echo(!x! !y!
set /a "rosella_arg1=x * 3"
call :show
if defined rosella_exit exit /b !rosella_exit!
set /a "rosella_arg1=(x + 1) * (-2)"
call :show
if defined rosella_exit exit /b !rosella_exit!
goto :eof

:show
    set "rosella_show_n=!rosella_arg1!"
    echo(n=!rosella_show_n!
    goto :eof

