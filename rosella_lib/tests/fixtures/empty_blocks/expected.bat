@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=2"
if !x! GTR 1 (
    rem
) else (
    echo(no
)
call :nothing
if !x! GTR 1 (
    rem
)
echo(ok
goto :eof

:nothing
    rem
    goto :eof

