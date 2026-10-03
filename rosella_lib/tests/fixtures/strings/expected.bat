@echo off
setlocal
set "rosella_script_dir=%~dp0"
set "rosella_script_dir=%rosella_script_dir:~0,-1%"
setlocal enabledelayedexpansion
set "rosella_exit="
set "name=Hello World"
call :rosella_length rosella_result0 name
set "rosella_measure="
call :rosella_length rosella_result1 rosella_measure
echo(length: !rosella_result0! empty: !rosella_result1!
set "rosella_result2="
if defined name set "rosella_result2=!name:~0,5!"
set "rosella_result3="
if defined name set "rosella_result3=!name:~6,100!"
set "rosella_result4="
if defined name set "rosella_result4=!name:~-5,3!"
echo(slice: !rosella_result2!^|!rosella_result3!^|!rosella_result4!
set "rosella_measure=abc"
call :rosella_length rosella_result5 rosella_measure
set /a "i=0"
call :rosella_for0
set "rosella_text=!name!"
set "rosella_from=o"
set "rosella_to=0"
call :rosella_replace rosella_result7
echo(replace: !rosella_result7!
set "rosella_text=aAaA"
set "rosella_from=a"
set "rosella_to=x"
call :rosella_replace rosella_result8
echo(case: !rosella_result8!
set "rosella_text=one fish two fish"
set "rosella_from=fish"
set "rosella_to=cat"
call :rosella_replace rosella_result9
echo(words: !rosella_result9!
set "rosella_text=50%% & (x)"
set "rosella_from=&"
set "rosella_to=and"
call :rosella_replace rosella_result10
echo(special: !rosella_result10!
set "rosella_result11=!name!"
if defined rosella_result11 for %%c in (A B C D E F G H I J K L M N O P Q R S T U V W X Y Z) do set "rosella_result11=!rosella_result11:%%c=%%c!"
set "rosella_result12=!name!"
if defined rosella_result12 for %%c in (a b c d e f g h i j k l m n o p q r s t u v w x y z) do set "rosella_result12=!rosella_result12:%%c=%%c!"
echo(upper: !rosella_result11! lower: !rosella_result12!
set "rosella_result13=café"
if defined rosella_result13 for %%c in (A B C D E F G H I J K L M N O P Q R S T U V W X Y Z) do set "rosella_result13=!rosella_result13:%%c=%%c!"
set "rosella_result14=CAFÉ"
if defined rosella_result14 for %%c in (a b c d e f g h i j k l m n o p q r s t u v w x y z) do set "rosella_result14=!rosella_result14:%%c=%%c!"
echo(accents stay: !rosella_result13! !rosella_result14!
set "rosella_text=!name!"
set "rosella_from=World"
call :rosella_contains
if defined rosella_found (
    echo(contains World
)
set "rosella_text=!name!"
set "rosella_from=world"
call :rosella_contains
if not defined rosella_found (
    echo(case matters
)
set "rosella_text=!name!"
set "rosella_from="
call :rosella_contains
if defined rosella_found (
    echo(empty text is contained
)
set /a "roll=1 + !random! %% (6 - 1 + 1)"
set "rosella_condition2="
set "rosella_condition3="
if !roll! GEQ 1 set "rosella_condition3=1"
if defined rosella_condition3 (
    if !roll! LEQ 6 set "rosella_condition2=1"
)
if defined rosella_condition2 (
    echo(roll in range
)
set /a "rosella_result15=4 + !random! %% (4 - 4 + 1)"
echo(fixed: !rosella_result15!
if not exist "folder" mkdir "folder"
>"note.txt" echo(x
set "rosella_condition4="
set "rosella_condition5="
if exist "folder\" set "rosella_condition5=1"
if defined rosella_condition5 (
    set "rosella_condition6="
    if exist "folder" if not exist "folder\" set "rosella_condition6=1"
    if not defined rosella_condition6 set "rosella_condition4=1"
)
if defined rosella_condition4 (
    echo(folder is a folder
)
set "rosella_condition7="
set "rosella_condition8="
set "rosella_condition9="
if exist "note.txt" if not exist "note.txt\" set "rosella_condition9=1"
if defined rosella_condition9 set "rosella_condition8=1"
if defined rosella_condition8 (
    if not exist "note.txt\" set "rosella_condition7=1"
)
if defined rosella_condition7 (
    echo(note is a file
)
set "rosella_condition10="
set "rosella_condition11="
if not exist "missing\" set "rosella_condition11=1"
if defined rosella_condition11 (
    set "rosella_condition12="
    if exist "missing" if not exist "missing\" set "rosella_condition12=1"
    if not defined rosella_condition12 set "rosella_condition10=1"
)
if defined rosella_condition10 (
    echo(missing is neither
)
set "rosella_condition13="
set "rosella_condition14="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!rosella_script_dir!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!rosella_script_dir!\expected.sh" if not exist "!rosella_script_dir!\expected.sh\" set "rosella_condition14=1"
if defined rosella_condition14 set "rosella_condition13=1"
if not defined rosella_condition13 (
    set "rosella_condition15="
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!rosella_script_dir!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "!rosella_script_dir!\expected.bat" if not exist "!rosella_script_dir!\expected.bat\" set "rosella_condition15=1"
    if defined rosella_condition15 set "rosella_condition13=1"
)
if defined rosella_condition13 (
    echo(script_dir found the script
)
set "nothing="
set "rosella_condition16="
set "rosella_condition17="
set "rosella_condition18="
set "rosella_condition19="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!nothing!x") do set "rosella_wild=1"
if not defined rosella_wild if not "!nothing!"=="" if exist "!nothing!\" set "rosella_condition19=1"
if not defined rosella_condition19 set "rosella_condition18=1"
if defined rosella_condition18 (
    set "rosella_condition20="
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!nothing!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "!nothing!" if not exist "!nothing!\" set "rosella_condition20=1"
    if not defined rosella_condition20 set "rosella_condition17=1"
)
if defined rosella_condition17 (
    set "rosella_condition21="
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!nothing!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "!nothing!" set "rosella_condition21=1"
    if not defined rosella_condition21 set "rosella_condition16=1"
)
if defined rosella_condition16 (
    echo(an empty path is nothing
)
set "rosella_condition22="
if exist "missing" if not exist "missing\" set "rosella_condition22=1"
if defined rosella_condition22 (
    echo(SHOULD NOT PRINT
) else (
    echo(is_file keeps its else
)
set /a "rosella_sleep=1 + 1"
ping -n !rosella_sleep! 127.0.0.1 >nul
echo(slept
exit /b 0

:rosella_for0
if !i! LSS !rosella_result5! goto :rosella_for0_body
goto :eof
:rosella_for0_body
    set "rosella_text=abc"
    set /a "rosella_temp1=i"
    set "rosella_result6="
    if defined rosella_text for /f "tokens=1,2" %%a in ("!rosella_temp1! 1") do set "rosella_result6=!rosella_text:~%%a,%%b!"
    echo(char !i!: !rosella_result6!
:rosella_for0_next
set /a "i=i + 1"
goto :rosella_for0

:rosella_contains
set "rosella_found="
set "rosella_original=!rosella_text!"
set "rosella_to="
call :rosella_replace rosella_result
if not defined rosella_from set "rosella_found=1"
if not "!rosella_result!"=="!rosella_original!" set "rosella_found=1"
goto :eof

:rosella_replace
set "rosella_result="
if not defined rosella_from set "rosella_result=!rosella_text!"
if not defined rosella_from goto :rosella_replace_done
call :rosella_length rosella_from_size rosella_from
:rosella_replace_loop
if not defined rosella_text goto :rosella_replace_done
for %%n in (!rosella_from_size!) do set "rosella_head=!rosella_text:~0,%%n!"
if "!rosella_head!"=="!rosella_from!" (
    set "rosella_result=!rosella_result!!rosella_to!"
    for %%n in (!rosella_from_size!) do set "rosella_text=!rosella_text:~%%n!"
) else (
    set "rosella_result=!rosella_result!!rosella_text:~0,1!"
    set "rosella_text=!rosella_text:~1!"
)
goto :rosella_replace_loop
:rosella_replace_done
set "%~1=!rosella_result!"
goto :eof

:rosella_length
set "rosella_scan=!%~2!#"
set "rosella_size=0"
for %%n in (4096 2048 1024 512 256 128 64 32 16 8 4 2 1) do (
    if "!rosella_scan:~%%n,1!" NEQ "" (
        set /a "rosella_size+=%%n"
        set "rosella_scan=!rosella_scan:~%%n!"
    )
)
set "%~1=!rosella_size!"
goto :eof

