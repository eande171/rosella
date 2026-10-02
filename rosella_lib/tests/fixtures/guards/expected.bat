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
    set "rosella_clean.dir=!rosella_arg1!"
    if not defined rosella_clean.dir (>&2 echo(Stopped: 'rosella_clean.dir' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
    for /f "delims=*?" %%w in ("x!rosella_clean.dir!x") do if "%%w"=="x!rosella_clean.dir!x" if exist "!rosella_clean.dir!\build" rmdir /s /q "!rosella_clean.dir!\build"
    goto :eof

