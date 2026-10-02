@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "project=proj"
if not exist "!project!\build\out" mkdir "!project!\build\out"
if not exist "!project!\build\out" mkdir "!project!\build\out"
>"!project!\settings.ini" echo(theme=dark
>>"!project!\settings.ini" echo(name=!project!^^!
>>"!project!\settings.ini" echo(more=100%%
copy /y "!project!\settings.ini" "!project!\build\copy.ini" >nul
move /y "!project!\build\copy.ini" "!project!\moved.ini" >nul
if exist "!project!\moved.ini" (
    echo(moved
)
if not exist "!project!\build\copy.ini" (
    echo(copy gone
)
if not defined project (
    >&2 echo(Stopped: 'project' is empty in a path to remove
    exit /b 1
)
if exist "!project!\moved.ini" del /f /q "!project!\moved.ini"
if not defined project (
    >&2 echo(Stopped: 'project' is empty in a path to remove
    exit /b 1
)
if exist "!project!\build" rmdir /s /q "!project!\build"
if not exist "!project!\build" (
    echo(build removed
)
if 1 EQU 1 (
    cd /d "!project!"
    set "here=!CD!"
    if exist "!here!\settings.ini" (
        echo(in project
    )
)
type settings.ini
cd /d ".."
if not defined project (
    >&2 echo(Stopped: 'project' is empty in a path to remove
    exit /b 1
)
if exist "!project!" rmdir /s /q "!project!"
