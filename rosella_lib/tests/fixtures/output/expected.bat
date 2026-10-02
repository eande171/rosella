@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=fixture.name=Hello & 50%% (done)"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=fixture.name"
set "name="
for /f usebackq^ delims^=^ eol^= %%l in (`""!rosella_command!" "!rosella_argument1!" "!rosella_argument2!" "!rosella_argument3!" "!rosella_argument4!" "!rosella_argument5!""`) do if not defined name set "name=%%l"
echo(name: !name!
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=a.b=1"
set "rosella_argument3=-c"
set "rosella_argument4=a.c=2"
set "rosella_argument5=config"
set "rosella_argument6=--get-regexp"
set "rosella_argument7=^a[.]"
set "first="
for /f usebackq^ delims^=^ eol^= %%l in (`""!rosella_command!" "!rosella_argument1!" "!rosella_argument2!" "!rosella_argument3!" "!rosella_argument4!" "!rosella_argument5!" "!rosella_argument6!" "!rosella_argument7!""`) do if not defined first set "first=%%l"
echo(first line: !first!
set "rosella_command=git"
set "rosella_argument1=config"
set "rosella_argument2=--file"
set "rosella_argument3=missing.cfg"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
set "rosella_result0="
for /f usebackq^ delims^=^ eol^= %%l in (`""!rosella_command!" "!rosella_argument1!" "!rosella_argument2!" "!rosella_argument3!" "!rosella_argument4!" "!rosella_argument5!""`) do if not defined rosella_result0 set "rosella_result0=%%l"
if "!rosella_result0!"=="" (
    echo(empty output
)
set "rosella_command=git"
set "rosella_argument1=-c"
set "rosella_argument2=x.y=inline"
set "rosella_argument3=config"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
set "rosella_result1="
for /f usebackq^ delims^=^ eol^= %%l in (`""!rosella_command!" "!rosella_argument1!" "!rosella_argument2!" "!rosella_argument3!" "!rosella_argument4!" "!rosella_argument5!""`) do if not defined rosella_result1 set "rosella_result1=%%l"
echo(inline: !rosella_result1!
set "rosella_arg1=demo.key"
call :setting
echo(!rosella_return!
goto :eof

:setting
    set "rosella_setting.key=!rosella_arg1!"
    set "rosella_command=git"
    set "rosella_argument1=-c"
    set "rosella_argument2=!rosella_setting.key!=from a function"
    set "rosella_argument3=config"
    set "rosella_argument4=--get"
    set "rosella_argument5=!rosella_setting.key!"
    set "rosella_return="
    for /f usebackq^ delims^=^ eol^= %%l in (`""!rosella_command!" "!rosella_argument1!" "!rosella_argument2!" "!rosella_argument3!" "!rosella_argument4!" "!rosella_argument5!""`) do if not defined rosella_return set "rosella_return=%%l"
    goto :eof

