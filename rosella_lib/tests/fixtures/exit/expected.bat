@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
echo(before
call :stop
if defined rosella_exit exit /b !rosella_exit!
echo(SHOULD NOT PRINT
exit /b 0

:rosella_while0
if !i! LSS 5 goto :rosella_while0_body
goto :eof
:rosella_while0_body
    if !i! EQU 2 (
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

