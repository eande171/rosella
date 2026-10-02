@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "empty="
set "rosella_arg1=!empty!"
call :clean
if defined rosella_exit exit /b !rosella_exit!
echo(SHOULD NOT PRINT
goto :eof

:clean
    set "rosella_clean_dir=!rosella_arg1!"
    if not defined rosella_clean_dir (
        >&2 echo(Stopped: 'rosella_clean_dir' is empty in a path to remove
        set /a "rosella_exit=1"
        goto :eof
    )
    if exist "!rosella_clean_dir!\build" rmdir /s /q "!rosella_clean_dir!\build"
    goto :eof

