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
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!pattern!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "data\!pattern!" set "rosella_condition0=1"
if defined rosella_condition0 (
    echo(SHOULD NOT PRINT
)
set "rosella_condition1="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!pattern!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "data\!pattern!" set "rosella_condition1=1"
if not defined rosella_condition1 (
    echo(a wildcard in a variable matches nothing
)
set /a "found=0"
call :rosella_files2
echo(files^(^) with a wildcard in a variable found !found!
if exist "data\keep.txt" (
    echo(keep.txt survived
)
if exist "data\" rmdir /s /q "data"
exit /b 0

:clean
    set "rosella_clean.file=!rosella_arg1!"
    if not defined rosella_clean.file (>&2 echo(Stopped: 'rosella_clean.file' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!rosella_clean.file!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "data\!rosella_clean.file!" if not exist "data\!rosella_clean.file!\" del /f /q "data\!rosella_clean.file!"
    goto :eof

:rosella_files2
set "rosella_break="
set "rosella_files2_dot=!pattern:~0,1!"
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!pattern!x") do set "rosella_wild=1"
if not defined rosella_wild for %%f in ("data\!pattern!") do (
    set "rosella_name=%%~nxf"
    set "rosella_show=1"
    if "!rosella_name:~0,1!"=="." if not "!rosella_files2_dot!"=="." set "rosella_show="
    if defined rosella_show (
        call :rosella_keep file
        call :rosella_files2_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files2_body
    set /a "found=found + 1"
goto :eof

:rosella_keep
setlocal disabledelayedexpansion
for %%z in (1) do set "rosella_raw=%%f"
set "rosella_raw=%rosella_raw:"=""%"
if "%rosella_raw:!=%"=="%rosella_raw%" goto :rosella_keep_done
set "rosella_raw=%rosella_raw:^=^^%"
set "rosella_raw=%rosella_raw:!=^!%"
:rosella_keep_done
for /f delims^=^ eol^= %%v in ("%rosella_raw%") do endlocal & set "%~1=%%v"
set "%~1=!%~1:""="!"
goto :eof

