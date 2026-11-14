@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
if not exist "full" mkdir "full"
>"full\inside.txt" echo(x
>"plain.txt" echo(x
if exist "full" if not exist "full\" del /f /q "full"
set "rosella_condition0="
if exist "full\inside.txt" if not exist "full\inside.txt\" set "rosella_condition0=1"
if defined rosella_condition0 (
    echo(remove left the folder alone
)
if exist "plain.txt\" rmdir /s /q "plain.txt"
set "rosella_condition1="
if exist "plain.txt" if not exist "plain.txt\" set "rosella_condition1=1"
if defined rosella_condition1 (
    echo(remove_dir left the file alone
)
if not exist "full\" copy /y "full" "full_copy" >nul
if not exist "full_copy" (
    echo(copy skipped the folder
)
move /y "full" "renamed" >nul
set "rosella_condition2="
set "rosella_condition3="
set "rosella_condition4="
if exist "renamed\inside.txt" if not exist "renamed\inside.txt\" set "rosella_condition4=1"
if defined rosella_condition4 set "rosella_condition3=1"
if defined rosella_condition3 (
    if not exist "full" set "rosella_condition2=1"
)
if defined rosella_condition2 (
    echo(move renamed the folder
)
move /y "plain.txt" "renamed" >nul
set "rosella_condition5="
if exist "renamed\plain.txt" if not exist "renamed\plain.txt\" set "rosella_condition5=1"
if defined rosella_condition5 (
    echo(move put the file in the folder
)
if exist "renamed\" rmdir /s /q "renamed"
set "rosella_command=git"
set "rosella_argument1=config"
set "rosella_argument2=--file"
set "rosella_argument3=none.cfg"
set "rosella_argument4=--get"
set "rosella_argument5=x.y"
setlocal disabledelayedexpansion
call "%%rosella_command%%" "%%rosella_argument1%%" "%%rosella_argument2%%" "%%rosella_argument3%%" "%%rosella_argument4%%" "%%rosella_argument5%%"
endlocal
exit /b 0
