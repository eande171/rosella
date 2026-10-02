@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=1"
set /a "i=0"
set /a "rosella_temp0=x"
if !rosella_temp0! EQU 1 (
    call :rosella_while4
    if defined rosella_exit exit /b !rosella_exit!
    echo(then
) else (
    echo(ELSE SHOULD NOT RUN
)
goto :eof

:rosella_while3
set /a "rosella_temp2=j"
if !rosella_temp2! LSS 2 goto :rosella_while3_body
goto :eof
:rosella_while3_body
    echo(!i!!j!
    set /a "j=j + 1"
goto :rosella_while3

:rosella_while4
set /a "rosella_temp1=i"
if !rosella_temp1! LSS 2 goto :rosella_while4_body
goto :eof
:rosella_while4_body
    set /a "j=0"
    call :rosella_while3
    if defined rosella_exit exit /b !rosella_exit!
    set /a "i=i + 1"
goto :rosella_while4

