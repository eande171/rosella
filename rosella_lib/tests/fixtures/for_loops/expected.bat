@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "i=0"
call :rosella_for0
set /a "i=10"
call :rosella_for1
set /a "limit=3"
set /a "rosella_result0=limit"
set /a "k=0"
call :rosella_for2
echo(limit is now !limit!
if not exist "data" mkdir "data"
if not exist "data\folder.txt" mkdir "data\folder.txt"
>"data\b.txt" echo(second
>"data\a.txt" echo(first
>"data\c.log" echo(other
>"data\.hidden.txt" echo(skipped like a hidden file
set /a "count=0"
call :rosella_files3
echo(txt files: !count!
call :rosella_files4
call :rosella_files5
call :rosella_files6
>"data\wow^!.md" echo(x
>"data\a^^b^!c.md" echo(x
>"data\50%% ^ plain.md" echo(x
set /a "kept=0"
call :rosella_files7
echo(names kept: !kept!
set /a "dotted=0"
call :rosella_files9
echo(dot names: !dotted!
>"data\.second" echo(x
set /a "both=0"
call :dot
call :rosella_files10
echo(dot names from a call: !both!
call :first_log
if "!rosella_return!"=="data\c.log" (
    echo(returned from a files loop
)
set "rosella_arg1=49"
call :square_root
echo(square root of 49: !rosella_return!
exit /b 0

:rosella_for0
if !i! LSS 5 goto :rosella_for0_body
goto :eof
:rosella_for0_body
    if !i! EQU 1 (
        goto :rosella_for0_next
    )
    if !i! EQU 4 (
        goto :eof
    )
    echo(up: !i!
:rosella_for0_next
set /a "i=i + 1"
goto :rosella_for0

:rosella_for1
if !i! GTR 0 goto :rosella_for1_body
goto :eof
:rosella_for1_body
    echo(down: !i!
:rosella_for1_next
set /a "i=i - 3"
goto :rosella_for1

:rosella_for2
if !k! LSS !rosella_result0! goto :rosella_for2_body
goto :eof
:rosella_for2_body
    set /a "limit=limit + 10"
    echo(k: !k!
:rosella_for2_next
set /a "k=k + 1"
goto :rosella_for2

:rosella_files3
set "rosella_break="
for %%f in ("data\*.txt") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files3_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files3_body
    set /a "count=count + 1"
    if "!file!"=="data\a.txt" (
        echo(found a.txt
    ) else (
        if "!file!"=="data\b.txt" (
            echo(found b.txt
        ) else (
            echo(UNEXPECTED MATCH
        )
    )
goto :eof

:rosella_files4
set "rosella_break="
for %%f in ("data\*.txt") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files4_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files4_body
    if "!file!"=="data\a.txt" (
        goto :eof
    )
    echo(after continue
goto :eof

:rosella_files5
set "rosella_break="
for %%f in ("data\*.txt") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files5_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files5_body
    echo(before break
    set "rosella_break=1"
    goto :eof
goto :eof

:rosella_files6
set "rosella_break="
for %%f in ("data\*.none") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files6_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files6_body
    echo(SHOULD NOT PRINT
goto :eof

:rosella_files7
set "rosella_break="
for %%f in ("data\*.md") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files7_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files7_body
    set "rosella_condition8="
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!file!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "!file!" if not exist "!file!\" set "rosella_condition8=1"
    if defined rosella_condition8 (
        set /a "kept=kept + 1"
    )
    if "!file!"=="data\wow^!.md" (
        echo(found wow^^!.md
    )
goto :eof

:rosella_files9
set "rosella_break="
for %%f in ("data\.*") do (
    if 1 EQU 1 (
        call :rosella_keep file
        call :rosella_files9_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files9_body
    set /a "dotted=dotted + 1"
goto :eof

:dot
    set "rosella_return=."
    goto :eof

:other
    set "rosella_return=other"
    goto :eof

:rosella_files10
set "rosella_break="
set "rosella_files10_dot=!rosella_return:~0,1!"
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!rosella_return!x") do set "rosella_wild=1"
if not defined rosella_wild for %%f in ("data\!rosella_return!*") do (
    set "rosella_name=%%~nxf"
    set "rosella_show=1"
    if "!rosella_name:~0,1!"=="." if not "!rosella_files10_dot!"=="." set "rosella_show="
    if defined rosella_show (
        call :rosella_keep file
        call :rosella_files10_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
    )
)
goto :eof

:rosella_files10_body
    call :other
    set "changed=!rosella_return!"
    set /a "both=both + 1"
goto :eof

:rosella_files11
set "rosella_break="
for %%f in ("data\*.log") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        call :rosella_keep file
        call :rosella_files11_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
        if defined rosella_returning goto :eof
    )
)
goto :eof

:rosella_files11_body
    set "rosella_return=!file!"
    set "rosella_returning=1"
    goto :eof
goto :eof

:first_log
    call :rosella_files11
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set "rosella_return=none"
    goto :eof

:rosella_for12
if !n! LSS 100 goto :rosella_for12_body
goto :eof
:rosella_for12_body
    set /a "rosella_temp13=n * n"
    if !rosella_temp13! EQU !rosella_square_root.target! (
        set /a "rosella_return=n"
        set "rosella_returning=1"
        goto :eof
    )
:rosella_for12_next
set /a "n=n + 1"
goto :rosella_for12

:square_root
    set "rosella_square_root.target=!rosella_arg1!"
    set /a "n=1"
    call :rosella_for12
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set /a "rosella_return=(-1)"
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

