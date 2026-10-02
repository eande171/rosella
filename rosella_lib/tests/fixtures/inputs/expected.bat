@echo off
set "rosella_argc=0"
:rosella_arguments
if "%~1"=="" goto :rosella_arguments_done
set /a "rosella_argc+=1"
set "rosella_arg_%rosella_argc%=%~1"
shift
goto :rosella_arguments
:rosella_arguments_done
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_result0=!rosella_argc!"
echo(count: !rosella_result0!
set "rosella_result1=!rosella_arg_1!"
echo(first: !rosella_result1!
set "rosella_result2=!rosella_arg_2!"
echo(second: !rosella_result2!
set "rosella_result3=!rosella_arg_3!"
echo(missing: [!rosella_result3!]
set /a "i=1"
call :rosella_while0
set "FIXTURE_MODE=50%% & (fast)^!"
echo(mode: !FIXTURE_MODE!
if "!FIXTURE_UNSET!"=="" (
    echo(unset is empty
)
>"50%% & (more)^!.txt" echo(same
>"same.txt" echo(same
>"other.txt" echo(different
set "rosella_command=git"
set "rosella_argument1=diff"
set "rosella_argument2=--no-index"
set "rosella_argument3=--quiet"
set "rosella_argument4=50%% & (more)^!.txt"
set "rosella_argument5=same.txt"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
set "same=!errorlevel!"
set "rosella_command=git"
set "rosella_argument1=diff"
set "rosella_argument2=--no-index"
set "rosella_argument3=--quiet"
set "rosella_argument4=same.txt"
set "rosella_argument5=other.txt"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
set "different=!errorlevel!"
echo(same: !same! different: !different!
set "rosella_result7=!rosella_arg_2!"
set "rosella_command=git"
set "rosella_argument1=diff"
set "rosella_argument2=--no-index"
set "rosella_argument3=--quiet"
set "rosella_argument4=!rosella_result7!"
set "rosella_argument5=same.txt"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
set "rosella_result6=!errorlevel!"
if !rosella_result6! EQU 0 (
    echo(argument reached git intact
)
set "rosella_command=git"
set "rosella_argument1=diff"
set "rosella_argument2=--no-index"
set "rosella_argument3=--quiet"
set "rosella_argument4=same.txt"
set "rosella_argument5=same.txt"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
set "rosella_command=rosella_missing_command"
setlocal disabledelayedexpansion
call "%%rosella_command%%"
endlocal
set "rosella_result8=!errorlevel!"
if !rosella_result8! NEQ 0 (
    echo(missing command failed
)
set /a "rosella_result9=(same + different) + 1"
echo(sum: !rosella_result9!
goto :eof

:rosella_while0
set "rosella_result4=!rosella_argc!"
if !i! LEQ !rosella_result4! goto :rosella_while0_body
goto :eof
:rosella_while0_body
    set /a "rosella_index=i"
    set "rosella_result5="
    for %%i in (!rosella_index!) do set "rosella_result5=!rosella_arg_%%i!"
    echo(!i!: !rosella_result5!
    set /a "i=i + 1"
goto :rosella_while0

