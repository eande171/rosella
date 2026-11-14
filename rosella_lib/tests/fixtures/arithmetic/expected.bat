@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=(1 + 2) * 3"
set /a "y=x - (-4)"
echo(!x! !y!
set /a "rosella_arg1=x * 3"
call :show
set /a "rosella_arg1=(x + 1) * (-2)"
call :show
exit /b 0

:show
    set "rosella_show.n=!rosella_arg1!"
    echo(n=!rosella_show.n!
    goto :eof

