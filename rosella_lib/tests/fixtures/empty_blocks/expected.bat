@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=2"
set /a "rosella_temp0=x"
if !rosella_temp0! GTR 1 (
    rem
) else (
    echo(no
)
call :nothing
if defined rosella_exit exit /b !rosella_exit!
set /a "rosella_temp1=x"
if !rosella_temp1! GTR 1 (
    rem
)
echo(ok
goto :eof

:nothing
    rem
    goto :eof

