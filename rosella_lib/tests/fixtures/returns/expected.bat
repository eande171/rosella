@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
set /a "total=rosella_return"
echo(total: !total!
set "rosella_arg1=2"
set "rosella_arg2=3"
call :double_sum
echo(doubled: !rosella_return!
set "rosella_arg1=1"
call :label
set "rosella_result0=!rosella_return!"
set "rosella_arg1=1"
set "rosella_arg2=1"
call :add
set "rosella_result1=!rosella_return!"
set "rosella_arg1=!rosella_result1!"
call :label
set "rosella_result2=!rosella_return!"
echo(!rosella_result0! / !rosella_result2!
set "rosella_arg1=5"
set "rosella_arg2=5"
call :add
set "rosella_arg1=!total!"
set "rosella_arg2=1"
call :add
if !rosella_return! EQU 4 (
    echo(four
)
set /a "n=0"
call :rosella_while3
echo(n: !n!
set "rosella_arg1=50"
call :first_square_over
echo(first square over 50: !rosella_return!
set "rosella_arg1=1"
set "rosella_arg2=2"
call :add
set "rosella_result3=!rosella_return!"
set "rosella_arg1=3"
set "rosella_arg2=4"
call :add
set "rosella_result4=!rosella_return!"
set "rosella_arg1=!rosella_result3!"
set "rosella_arg2=!rosella_result4!"
call :add
set "rosella_result5=!rosella_return!"
set /a "nested=rosella_result5 - 1"
echo(nested: !nested!
goto :eof

:add
    set "rosella_add.a=!rosella_arg1!"
    set "rosella_add.b=!rosella_arg2!"
    set /a "rosella_return=rosella_add.a + rosella_add.b"
    goto :eof

:label
    set "rosella_label.n=!rosella_arg1!"
    if !rosella_label.n! EQU 1 (
        set "rosella_return=one"
        goto :eof
    )
    set "rosella_return=many: !rosella_label.n!"
    goto :eof

:rosella_while0
set /a "rosella_temp1=i"
if !rosella_temp1! LSS 100 goto :rosella_while0_body
goto :eof
:rosella_while0_body
    set /a "rosella_temp2=i * i"
    if !rosella_temp2! GTR !rosella_first_square_over.limit! (
        set /a "rosella_return=i"
        set "rosella_returning=1"
        goto :eof
    )
    set /a "i=i + 1"
goto :rosella_while0

:first_square_over
    set "rosella_first_square_over.limit=!rosella_arg1!"
    set /a "i=0"
    call :rosella_while0
    if defined rosella_returning (set "rosella_returning=" & goto :eof)
    set /a "rosella_return=(-1)"
    goto :eof

:double_sum
    set "rosella_double_sum.a=!rosella_arg1!"
    set "rosella_double_sum.b=!rosella_arg2!"
    set "rosella_arg1=!rosella_double_sum.a!"
    set "rosella_arg2=!rosella_double_sum.b!"
    call :add
    set /a "rosella_return=rosella_return * 2"
    goto :eof

:rosella_while3
set "rosella_arg1=!n!"
set "rosella_arg2=0"
call :add
if !rosella_return! LSS 3 goto :rosella_while3_body
goto :eof
:rosella_while3_body
    set /a "n=n + 1"
goto :rosella_while3

