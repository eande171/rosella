@echo off
setlocal
set "rosella_argc=0"
:rosella_arguments
if "%~1"=="" if [%1]==[] goto :rosella_arguments_done
set /a "rosella_argc+=1"
set "rosella_arg_%rosella_argc%=%~1"
shift
goto :rosella_arguments
:rosella_arguments_done
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_text=5"
call :rosella_number rosella_result0
set /a "rosella_result1=rosella_result0 + 1"
set "rosella_text=-5"
call :rosella_number rosella_result2
set /a "rosella_result3=rosella_result2 * 2"
set "rosella_text=007"
call :rosella_number rosella_result4
set "rosella_text=08"
call :rosella_number rosella_result5
set "rosella_text=09"
call :rosella_number rosella_result6
set /a "rosella_result7=rosella_result5 + rosella_result6"
echo(!rosella_result1! !rosella_result3! !rosella_result4! !rosella_result7!
set "rosella_text=2147483647"
call :rosella_number rosella_result8
set "rosella_text=-2147483647"
call :rosella_number rosella_result9
set "rosella_text=0"
call :rosella_number rosella_result10
set "rosella_text=-0"
call :rosella_number rosella_result11
echo(!rosella_result8! !rosella_result9! !rosella_result10! !rosella_result11!
set "rosella_text=abc"
call :rosella_number rosella_result12
set "rosella_text="
call :rosella_number rosella_result13
set "rosella_text=1.5"
call :rosella_number rosella_result14
set "rosella_text=5 6"
call :rosella_number rosella_result15
set "rosella_text=- 5"
call :rosella_number rosella_result16
set "rosella_text=+5"
call :rosella_number rosella_result17
set "rosella_text=--5"
call :rosella_number rosella_result18
set "rosella_text=-"
call :rosella_number rosella_result19
echo(!rosella_result12!!rosella_result13!!rosella_result14!!rosella_result15!!rosella_result16!!rosella_result17!!rosella_result18!!rosella_result19!
set "rosella_text= 5"
call :rosella_number rosella_result20
set "rosella_text=5 "
call :rosella_number rosella_result21
set "rosella_text=  -5  "
call :rosella_number rosella_result22
set "rosella_text=	7	"
call :rosella_number rosella_result23
set "rosella_text= "
call :rosella_number rosella_result24
echo(!rosella_result20! !rosella_result21! !rosella_result22! !rosella_result23! !rosella_result24!
set "rosella_text=2147483648"
call :rosella_number rosella_result25
set "rosella_text=99999999999"
call :rosella_number rosella_result26
set "rosella_text=12a"
call :rosella_number rosella_result27
set "rosella_text=a (b) & c^!"
call :rosella_number rosella_result28
echo(!rosella_result25!!rosella_result26!!rosella_result27!!rosella_result28!
set "rosella_arg1=42"
call :describe
set "rosella_arg1=-42"
call :describe
set "rosella_arg1=0012"
call :describe
set "rosella_arg1= 42 "
call :describe
set "rosella_arg1=4 2"
call :describe
set "rosella_arg1=four"
call :describe
set "rosella_arg1="
call :describe
set "rosella_arg1=50%%"
call :describe
set "rosella_arg1=^!x^!"
call :describe
set "rosella_arg1=;5"
call :describe
set /a "total=0"
set /a "i=1"
call :rosella_while0
echo(total of the number arguments: !total!
set "answer="
set /p "answer="
set /a "tries=0"
call :rosella_while1
set "rosella_text=!answer!"
call :rosella_number rosella_result34
echo(answer !rosella_result34! after !tries! retry
set "rosella_condition4="
set "rosella_condition5="
set "rosella_condition6="
set "rosella_text=1"
call :rosella_number
if defined rosella_valid set "rosella_condition6=1"
if defined rosella_condition6 (
    set "rosella_text=x"
    call :rosella_number
    if not defined rosella_valid set "rosella_condition5=1"
)
if defined rosella_condition5 (
    set "rosella_text=3"
    call :rosella_number rosella_result35
    if !rosella_result35! EQU 3 set "rosella_condition4=1"
)
if defined rosella_condition4 (
    echo(combined
)
set "rosella_text=3"
call :rosella_number rosella_result36
set /a "n=0"
call :rosella_for7
set "rosella_arg1=21"
call :doubled
set "rosella_result38=!rosella_return!"
set "rosella_arg1=oops"
call :doubled
set "rosella_result39=!rosella_return!"
echo(!rosella_result38! !rosella_result39!
set "rosella_text=4"
call :rosella_number rosella_result40
set /a "rosella_exit=rosella_result40"
exit /b !rosella_exit!
exit /b 0

:describe
    set "rosella_describe.text=!rosella_arg1!"
    set "rosella_text=!rosella_describe.text!"
    call :rosella_number
    if defined rosella_valid (
        set "rosella_text=!rosella_describe.text!"
        call :rosella_number rosella_result29
        echo([!rosella_describe.text!] is a number: !rosella_result29!
    ) else (
        echo([!rosella_describe.text!] is not a number
    )
    goto :eof

:rosella_while0
set "rosella_result30=!rosella_argc!"
if !i! LEQ !rosella_result30! goto :rosella_while0_body
goto :eof
:rosella_while0_body
    set /a "rosella_index=i"
    set "rosella_result31="
    for %%i in (!rosella_index!) do set "rosella_result31=!rosella_arg_%%i!"
    set "rosella_text=!rosella_result31!"
    call :rosella_number
    if defined rosella_valid (
        set /a "rosella_index=i"
        set "rosella_result33="
        for %%i in (!rosella_index!) do set "rosella_result33=!rosella_arg_%%i!"
        set "rosella_text=!rosella_result33!"
        call :rosella_number rosella_result32
        set /a "total=total + rosella_result32"
    )
    set /a "i=i + 1"
goto :rosella_while0

:rosella_while1
set "rosella_condition2="
set "rosella_condition3="
set "rosella_text=!answer!"
call :rosella_number
if not defined rosella_valid set "rosella_condition3=1"
if defined rosella_condition3 (
    if !tries! LSS 3 set "rosella_condition2=1"
)
if defined rosella_condition2 goto :rosella_while1_body
goto :eof
:rosella_while1_body
    set /a "tries=tries + 1"
    set "answer=7"
goto :rosella_while1

:rosella_for7
if !n! LSS !rosella_result36! goto :rosella_for7_body
goto :eof
:rosella_for7_body
    echo(n !n!
:rosella_for7_next
set /a "n=n + 1"
goto :rosella_for7

:doubled
    set "rosella_doubled.text=!rosella_arg1!"
    set "rosella_text=!rosella_doubled.text!"
    call :rosella_number rosella_result37
    set /a "rosella_return=rosella_result37 * 2"
    goto :eof

:rosella_number
set "rosella_valid="
set "rosella_value=0"
set "rosella_sign="
set "rosella_digits=!rosella_text!"
:rosella_number_trim
if not defined rosella_digits goto :rosella_number_done
for %%s in (" " "	") do (
    if "!rosella_digits:~0,1!"==%%s (set "rosella_digits=!rosella_digits:~1!" & goto :rosella_number_trim)
    if "!rosella_digits:~-1!"==%%s (set "rosella_digits=!rosella_digits:~0,-1!" & goto :rosella_number_trim)
)
if "!rosella_digits:~0,1!"=="-" (
    set "rosella_sign=-"
    set "rosella_digits=!rosella_digits:~1!"
)
if not defined rosella_digits goto :rosella_number_done
for /f delims^=0123456789^ eol^= %%a in ("!rosella_digits!") do goto :rosella_number_done
:rosella_number_zeros
if "!rosella_digits:~0,1!"=="0" if not "!rosella_digits!"=="0" (
    set "rosella_digits=!rosella_digits:~1!"
    goto :rosella_number_zeros
)
if not "!rosella_digits:~10,1!"=="" goto :rosella_number_done
if not "!rosella_digits:~9,1!"=="" if "!rosella_digits!" GTR "2147483647" goto :rosella_number_done
set "rosella_valid=1"
set /a "rosella_value=!rosella_sign!!rosella_digits!"
:rosella_number_done
if not "%~1"=="" set "%~1=!rosella_value!"
goto :eof

