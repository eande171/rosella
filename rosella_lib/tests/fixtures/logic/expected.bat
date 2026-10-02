@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set /a "x=2"
set "name=Bob"
set "rosella_condition0="
set "rosella_condition1="
if !x! GTR 0 set "rosella_condition1=1"
if defined rosella_condition1 (
    if "!name!"=="Bob" set "rosella_condition0=1"
)
if defined rosella_condition0 (
    echo(and: yes
)
set "rosella_condition2="
if !x! GTR 5 set "rosella_condition2=1"
if not defined rosella_condition2 (
    if "!name!"=="Bob" set "rosella_condition2=1"
)
if defined rosella_condition2 (
    echo(or: yes
)
set "rosella_condition3="
set "rosella_condition4="
if !x! EQU 1 set "rosella_condition4=1"
if not defined rosella_condition4 (
    if !x! EQU 3 set "rosella_condition4=1"
)
if not defined rosella_condition4 set "rosella_condition3=1"
if defined rosella_condition3 (
    echo(not: yes
)
set "rosella_condition5="
set "rosella_condition6="
if not exist "missing.txt" set "rosella_condition6=1"
if defined rosella_condition6 (
    if !x! EQU 2 set "rosella_condition5=1"
)
if defined rosella_condition5 (
    echo(file and int: yes
)
set "rosella_condition7="
if !x! EQU 9 set "rosella_condition7=1"
if not defined rosella_condition7 (
    set "rosella_condition8="
    if !x! EQU 2 set "rosella_condition8=1"
    if defined rosella_condition8 (
        if "!name!"=="Bob" set "rosella_condition7=1"
    )
)
if defined rosella_condition7 (
    echo(and binds tighter
)
if !x! EQU 2 (
    echo(double not
)
set "rosella_condition9="
set "rosella_condition10="
if !x! EQU 9 set "rosella_condition10=1"
if defined rosella_condition10 (
    set "rosella_arg1=1"
    call :noisy
    if !rosella_return! EQU 1 set "rosella_condition9=1"
)
if defined rosella_condition9 (
    echo(SHOULD NOT PRINT
)
set "rosella_condition11="
if !x! EQU 2 set "rosella_condition11=1"
if not defined rosella_condition11 (
    set "rosella_arg1=2"
    call :noisy
    if !rosella_return! EQU 2 set "rosella_condition11=1"
)
if defined rosella_condition11 (
    echo(or stopped early
)
set "rosella_condition12="
set "rosella_condition13="
if !x! EQU 2 set "rosella_condition13=1"
if defined rosella_condition13 (
    set "rosella_arg1=3"
    call :noisy
    if !rosella_return! EQU 3 set "rosella_condition12=1"
)
if defined rosella_condition12 (
    echo(and ran both
)
set /a "rosella_result0=10 %% 3"
set /a "rosella_result1=17 %% 5"
echo(10 %% 3 = !rosella_result0!, 17 %% 5 = !rosella_result1!
set /a "r=x %% 2"
echo(remainder: !r!
set /a "i=0"
call :rosella_while14
set /a "outer=0"
call :rosella_while16
set /a "n=0"
call :rosella_while19
echo(stopped at !n!
goto :eof

:noisy
    set "rosella_noisy.value=!rosella_arg1!"
    echo(noisy ran with !rosella_noisy.value!
    set /a "rosella_return=rosella_noisy.value"
    goto :eof

:rosella_while14
if !i! LSS 10 goto :rosella_while14_body
goto :eof
:rosella_while14_body
    set /a "i=i + 1"
    set /a "rosella_temp15=i %% 2"
    if !rosella_temp15! EQU 0 (
        goto :rosella_while14
    )
    if !i! EQU 7 (
        goto :eof
    )
    echo(odd: !i!
goto :rosella_while14

:rosella_while17
if 1 EQU 1 goto :rosella_while17_body
goto :eof
:rosella_while17_body
    set /a "inner=inner + 1"
    set /a "rosella_temp18=inner"
    if !rosella_temp18! GTR 2 (
        goto :eof
    )
goto :rosella_while17

:rosella_while16
if !outer! LSS 2 goto :rosella_while16_body
goto :eof
:rosella_while16_body
    set /a "inner=0"
    call :rosella_while17
    echo(outer !outer! inner !inner!
    set /a "outer=outer + 1"
goto :rosella_while16

:rosella_while19
set "rosella_condition20="
set "rosella_condition21="
if !n! LSS 5 set "rosella_condition21=1"
if defined rosella_condition21 (
    set "rosella_arg1=!n!"
    call :noisy
    if !rosella_return! NEQ 3 set "rosella_condition20=1"
)
if defined rosella_condition20 goto :rosella_while19_body
goto :eof
:rosella_while19_body
    set /a "n=n + 1"
goto :rosella_while19

