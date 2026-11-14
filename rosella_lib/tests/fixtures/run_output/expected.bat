@echo off
setlocal
set "rosella_capture=%TEMP%\rosella_%RANDOM%%RANDOM%.tmp"
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=fixture.name=Hello & 50%% (done)"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=fixture.name"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "name="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined name call :rosella_keep name
del "!rosella_capture!"
echo(name: !name!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=a.b=1"
set "rosella_argument3=-c"
set "rosella_argument4=a.c=2"
set "rosella_argument5=config"
set "rosella_argument6=--get-regexp"
set "rosella_argument7=^a[.]"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" "%%rosella_argument6%%" "%%rosella_argument7%%" >"%rosella_capture%"
endlocal
set "first="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined first call :rosella_keep first
del "!rosella_capture!"
echo(first line: !first!
set "rosella_command=git"
set "rosella_argument1=config"
set "rosella_argument2=--file"
set "rosella_argument3=missing.cfg"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result0="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result0 call :rosella_keep rosella_result0
del "!rosella_capture!"
if "!rosella_result0!"=="" (
    echo(empty output
)
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=inline"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result1="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result1 call :rosella_keep rosella_result1
del "!rosella_capture!"
echo(inline: !rosella_result1!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=Done^! 100%% ^^ caret"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result2="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result2 call :rosella_keep rosella_result2
del "!rosella_capture!"
echo(special: !rosella_result2!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=a ^ b"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result3="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result3 call :rosella_keep rosella_result3
del "!rosella_capture!"
echo(plain caret: !rosella_result3!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=%%PATH%% stays"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result4="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result4 call :rosella_keep rosella_result4
del "!rosella_capture!"
echo(percent name: !rosella_result4!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=C:\dir\\"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result5="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result5 call :rosella_keep rosella_result5
del "!rosella_capture!"
echo(backslash: !rosella_result5!
set "folder=C:\data\\"
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=!folder!"
if "!rosella_argument2:~-1!"=="\" call :rosella_slashes rosella_argument2
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result6="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result6 call :rosella_keep rosella_result6
del "!rosella_capture!"
echo(backslashes: !rosella_result6!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=run !folder!"
if "!rosella_argument2:~-1!"=="\" call :rosella_slashes rosella_argument2
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
>"q.cfg" echo([x]
>>"q.cfg" echo(	z = plain \^" ^& echo INJECTED ^& rem \^"
>>"q.cfg" echo(	m =
>>"q.cfg" echo(	m = second
set "rosella_command=git"
set "rosella_argument1=config"
set "rosella_argument2=--file"
set "rosella_argument3=q.cfg"
set "rosella_argument4=--get"
set "rosella_argument5=x.z"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result7="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result7 call :rosella_keep rosella_result7
del "!rosella_capture!"
echo(quotes: !rosella_result7!
set "rosella_command=git"
set "rosella_argument1=config"
set "rosella_argument2=--file"
set "rosella_argument3=q.cfg"
set "rosella_argument4=--get-all"
set "rosella_argument5=x.m"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
endlocal
set "rosella_result8="
for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_result8 call :rosella_keep rosella_result8
del "!rosella_capture!"
echo(first line with text: [!rosella_result8!]
set "rosella_arg1=demo.key"
call :setting
echo(!rosella_return!
exit /b 0

:setting
    set "rosella_setting.key=!rosella_arg1!"
    set "rosella_command=git"
    set "rosella_argument1=-c"
    set "rosella_argument2=!rosella_setting.key!=from a function"
    if "!rosella_argument2:~-1!"=="\" call :rosella_slashes rosella_argument2
    set "rosella_argument3=config"
    set "rosella_argument4=--get"
    set "rosella_argument5=!rosella_setting.key!"
    if "!rosella_argument5:~-1!"=="\" call :rosella_slashes rosella_argument5
    setlocal disabledelayedexpansion
    call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%" >"%rosella_capture%"
    endlocal
    set "rosella_return="
    for /f usebackq^ delims^=^ eol^= %%f in ("!rosella_capture!") do if not defined rosella_return call :rosella_keep rosella_return
    del "!rosella_capture!"
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

:rosella_slashes
set "rosella_tail=!%~1!"
set "rosella_extra="
:rosella_slashes_loop
if "!rosella_tail:~-1!"=="\" (
    set "rosella_extra=!rosella_extra!\"
    set "rosella_tail=!rosella_tail:~0,-1!"
    goto :rosella_slashes_loop
)
set "%~1=!%~1!!rosella_extra!"
goto :eof

