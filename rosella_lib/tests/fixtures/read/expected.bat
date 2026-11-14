@echo off
setlocal enabledelayedexpansion
set "rosella_exit="
set "name=Name: "
set "rosella_prompt=!name!"
set "name="
set /p "name=!rosella_prompt!"
echo(Hello !name!^^!
exit /b 0
