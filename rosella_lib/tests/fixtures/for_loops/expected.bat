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
call :first_log
if "!rosella_return!"=="data\c.log" (
    echo(returned from a files loop
)
set "rosella_arg1=49"
call :square_root
echo(square root of 49: !rosella_return!
goto :eof

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
        set "file=%%~f"
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
        set "file=%%~f"
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
        set "file=%%~f"
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
        set "file=%%~f"
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
for %%f in ("data\*.log") do (
    set "rosella_name=%%~nxf"
    if not "!rosella_name:~0,1!"=="." (
        set "file=%%~f"
        call :rosella_files7_body
        if defined rosella_break (set "rosella_break=" & goto :eof)
        if defined rosella_returning goto :eof
    )
)
goto :eof

:rosella_files7_body
    set "rosella_return=!file!"
    set "rosella_returning=1"
    goto :eof
goto :eof

:first_log
    call :rosella_files7
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set "rosella_return=none"
    goto :eof

:rosella_for8
if !n! LSS 100 goto :rosella_for8_body
goto :eof
:rosella_for8_body
    set /a "rosella_temp9=n * n"
    if !rosella_temp9! EQU !rosella_square_root.target! (
        set /a "rosella_return=n"
        set "rosella_returning=1"
        goto :eof
    )
:rosella_for8_next
set /a "n=n + 1"
goto :rosella_for8

:square_root
    set "rosella_square_root.target=!rosella_arg1!"
    set /a "n=1"
    call :rosella_for8
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set /a "rosella_return=(-1)"
    goto :eof

