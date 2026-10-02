@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
if defined rosella_exit exit /b !rosella_exit!
set /a "total=rosella_return"
echo(total: !total!
set "rosella_arg1=2"
set "rosella_arg2=3"
call :double_sum
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result1=!rosella_return!"
echo(doubled: !rosella_result1!
set "rosella_arg1=1"
call :label
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result2=!rosella_return!"
set "rosella_arg1=1"
set "rosella_arg2=1"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result3=!rosella_return!"
set "rosella_arg1=!rosella_result3!"
call :label
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result4=!rosella_return!"
echo(!rosella_result2! / !rosella_result4!
set "rosella_arg1=5"
set "rosella_arg2=5"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_arg1=!total!"
set "rosella_arg2=1"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result5=!rosella_return!"
set /a "rosella_temp5=rosella_result5"
if !rosella_temp5! EQU 4 (
    echo(four
)
set /a "n=0"
call :rosella_while7
if defined rosella_exit exit /b !rosella_exit!
echo(n: !n!
set "rosella_arg1=50"
call :first_square_over
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result7=!rosella_return!"
echo(first square over 50: !rosella_result7!
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result8=!rosella_return!"
set "rosella_arg1=3"
set "rosella_arg2=4"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result9=!rosella_return!"
set "rosella_arg1=!rosella_result8!"
set "rosella_arg2=!rosella_result9!"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result10=!rosella_return!"
set /a "nested=rosella_result10 - 1"
echo(nested: !nested!
goto :eof

:add
    set "rosella_add_a=!rosella_arg1!"
    set "rosella_add_b=!rosella_arg2!"
    set /a "rosella_return=rosella_add_a + rosella_add_b"
    goto :eof

:label
    set "rosella_label_n=!rosella_arg1!"
    set /a "rosella_temp0=rosella_label_n"
    if !rosella_temp0! EQU 1 (
        set "rosella_return=one"
        goto :eof
    )
    set "rosella_return=many: !rosella_label_n!"
    goto :eof

:rosella_while4
set /a "rosella_temp1=i"
if !rosella_temp1! LSS 100 goto :rosella_while4_body
goto :eof
:rosella_while4_body
    set /a "rosella_temp2=i * i"
    set /a "rosella_temp3=rosella_first_square_over_limit"
    if !rosella_temp2! GTR !rosella_temp3! (
        set /a "rosella_return=i"
        set "rosella_returning=1"
        goto :eof
    )
    set /a "i=i + 1"
goto :rosella_while4

:first_square_over
    set "rosella_first_square_over_limit=!rosella_arg1!"
    set /a "i=0"
    call :rosella_while4
    if defined rosella_exit exit /b !rosella_exit!
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set /a "rosella_return=(-1)"
    goto :eof

:double_sum
    set "rosella_double_sum_a=!rosella_arg1!"
    set "rosella_double_sum_b=!rosella_arg2!"
    set "rosella_arg1=!rosella_double_sum_a!"
    set "rosella_arg2=!rosella_double_sum_b!"
    call :add
    if defined rosella_exit exit /b !rosella_exit!
    set "rosella_result0=!rosella_return!"
    set /a "rosella_return=rosella_result0 * 2"
    goto :eof

:rosella_while7
set "rosella_arg1=!n!"
set "rosella_arg2=0"
call :add
if defined rosella_exit exit /b !rosella_exit!
set "rosella_result6=!rosella_return!"
set /a "rosella_temp6=rosella_result6"
if !rosella_temp6! LSS 3 goto :rosella_while7_body
goto :eof
:rosella_while7_body
    set /a "n=n + 1"
goto :rosella_while7

