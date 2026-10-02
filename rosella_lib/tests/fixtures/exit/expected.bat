@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
echo(before
call :stop
if defined rosella_exit exit /b !rosella_exit!
echo(SHOULD NOT PRINT
goto :eof

:rosella_while0
set /a "rosella_temp1=i"
if !rosella_temp1! LSS 5 goto :rosella_while0_body
goto :eof
:rosella_while0_body
    set /a "rosella_temp2=i"
    if !rosella_temp2! EQU 2 (
        set /a "rosella_exit=3"
        goto :eof
    )
    set /a "i=i + 1"
goto :rosella_while0

:stop
    set /a "i=0"
    call :rosella_while0
    if defined rosella_exit exit /b !rosella_exit!
    goto :eof

