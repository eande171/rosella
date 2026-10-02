@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
echo(before
call :stop
if defined rosella_exit exit /b !rosella_exit!
echo(SHOULD NOT PRINT
goto :eof

:rosella_while2
set /a "rosella_temp0=i"
if !rosella_temp0! LSS 5 goto :rosella_while2_body
goto :eof
:rosella_while2_body
    set /a "rosella_temp1=i"
    if !rosella_temp1! EQU 2 (
        set /a "rosella_exit=3"
        goto :eof
    )
    set /a "i=i + 1"
goto :rosella_while2

:stop
    set /a "i=0"
    call :rosella_while2
    if defined rosella_exit exit /b !rosella_exit!
    goto :eof

