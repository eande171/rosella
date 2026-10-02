@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=1"
set /a "i=0"
if !x! EQU 1 (
    call :rosella_while0
    echo(then
) else (
    echo(ELSE SHOULD NOT RUN
)
goto :eof

:rosella_while1
set /a "rosella_temp2=j"
if !rosella_temp2! LSS 2 goto :rosella_while1_body
goto :eof
:rosella_while1_body
    echo(!i!!j!
    set /a "j=j + 1"
goto :rosella_while1

:rosella_while0
if !i! LSS 2 goto :rosella_while0_body
goto :eof
:rosella_while0_body
    set /a "j=0"
    call :rosella_while1
    set /a "i=i + 1"
goto :rosella_while0

