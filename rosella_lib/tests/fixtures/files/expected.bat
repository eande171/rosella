@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "project=proj"
if not exist "!project!\build\out" mkdir "!project!\build\out"
if not exist "!project!\build\out" mkdir "!project!\build\out"
>"!project!\settings.ini" echo(theme=dark
>>"!project!\settings.ini" echo(name=!project!^^!
>>"!project!\settings.ini" echo(more=100%%
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!!project!x") do set "rosella_wild=1"
if not defined rosella_wild if not exist "!project!\settings.ini\" copy /y "!project!\settings.ini" "!project!\build\copy.ini" >nul
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!!project!x") do set "rosella_wild=1"
if not defined rosella_wild move /y "!project!\build\copy.ini" "!project!\moved.ini" >nul
set "rosella_condition0="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\moved.ini" set "rosella_condition0=1"
if defined rosella_condition0 (
    echo(moved
)
set "rosella_condition1="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\build\copy.ini" set "rosella_condition1=1"
if not defined rosella_condition1 (
    echo(copy gone
)
if not defined project (>&2 echo(Stopped: 'project' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\moved.ini" if not exist "!project!\moved.ini\" del /f /q "!project!\moved.ini"
if not defined project (>&2 echo(Stopped: 'project' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\build\" rmdir /s /q "!project!\build"
set "rosella_condition2="
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\build" set "rosella_condition2=1"
if not defined rosella_condition2 (
    echo(build removed
)
if 1 EQU 1 (
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
    if not defined rosella_wild cd /d "!project!"
    set "here=!CD!"
    set "rosella_condition3="
    set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!here!x") do set "rosella_wild=1"
    if not defined rosella_wild if exist "!here!\settings.ini" set "rosella_condition3=1"
    if defined rosella_condition3 (
        echo(in project
    )
)
type settings.ini
cd /d ".."
if not defined project (>&2 echo(Stopped: 'project' is empty in a path to remove& set "rosella_exit=1"& exit /b 1)
set "rosella_wild=" & for /f "tokens=2 delims=*?" %%w in ("x!project!x") do set "rosella_wild=1"
if not defined rosella_wild if exist "!project!\" rmdir /s /q "!project!"
exit /b 0
