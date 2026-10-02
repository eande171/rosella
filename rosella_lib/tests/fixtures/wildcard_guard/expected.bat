@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
if not exist "data" mkdir "data"
>"data\keep.txt" echo(x
set "pattern=*.txt"
set "rosella_arg1=!pattern!"
call :clean
if defined rosella_exit exit /b !rosella_exit!
set "rosella_condition0="
for /f "delims=*?" %%w in ("x!pattern!x") do if "%%w"=="x!pattern!x" if exist "data\!pattern!" set "rosella_condition0=1"
if defined rosella_condition0 (
    echo(SHOULD NOT PRINT
)
set "rosella_condition1="
for /f "delims=*?" %%w in ("x!pattern!x") do if "%%w"=="x!pattern!x" if exist "data\!pattern!" set "rosella_condition1=1"
if not defined rosella_condition1 (
    echo(a wildcard in a variable matches nothing
)
set /a "found=0"
call :rosella_files2
echo(files^(^) with a wildcard in a variable found !found!
if exist "data\keep.txt" (
    echo(keep.txt survived
)
if exist "data" rmdir /s /q "data"
goto :eof

:clean
    set "rosella_clean.file=!rosella_arg1!"
    if not defined rosella_clean.file (>&2 echo(Stopped: 'rosella_clean.file' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
    for /f "delims=*?" %%w in ("x!rosella_clean.file!x") do if "%%w"=="x!rosella_clean.file!x" if exist "data\!rosella_clean.file!" del /f /q "data\!rosella_clean.file!"
    goto :eof

:rosella_files2
set "rosella_break="
for /f "delims=*?" %%w in ("x!pattern!x") do if "%%w"=="x!pattern!x" for %%f in ("data\!pattern!") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        set "file=%%~f"
        call :rosella_files2_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files2_body
    set /a "found=found + 1"
goto :eof

