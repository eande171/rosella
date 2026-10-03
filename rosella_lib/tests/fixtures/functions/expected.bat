@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_arg1=Bob Smith"
set "rosella_arg2=Hi^!"
call :greet
set "n=Ann & Lee (100%%)"
set "rosella_arg1=!n!"
set "rosella_arg2=Hello"
call :greet
set /a "x=5"
set "rosella_arg1=1"
call :bump
echo(outside: !x!
set "rosella_arg1=called before its definition"
call :later
if !x! EQU 99 (
    rem
)
call :only_if
set "rosella_arg1=1"
call :a_b
set "rosella_arg1=99"
call :shadow
call :nothing
echo(nothing returned
exit /b 0

:greet
    set "rosella_greet.name=!rosella_arg1!"
    set "rosella_greet.greeting=!rosella_arg2!"
    echo(!rosella_greet.greeting! !rosella_greet.name!.
    goto :eof

:bump
    set "rosella_bump.x=!rosella_arg1!"
    set /a "rosella_bump.x=rosella_bump.x + 100"
    echo(inside: !rosella_bump.x!
    goto :eof

:only_if
    echo(defined inside an if
    goto :eof

:later
    set "rosella_later.message=!rosella_arg1!"
    echo(!rosella_later.message!
    goto :eof

:a_b
    set "rosella_a_b.c=!rosella_arg1!"
    set "rosella_arg1=5"
    call :a
    echo(a_b still has c = !rosella_a_b.c!
    goto :eof

:a
    set "rosella_a.b_c=!rosella_arg1!"
    echo(a got !rosella_a.b_c!
    goto :eof

:show_x
    echo(show_x sees !x!
    goto :eof

:shadow
    set "rosella_shadow.x=!rosella_arg1!"
    call :show_x
    goto :eof

:nothing
    goto :eof

