@echo off

title Android Runner
echo Compiling and Running Project...

cargo apk run

if %errorlevel% neq 0 exit /b %errorlevel%

@REM rem Clear the screen once compilation is done
cls

timeout 2 > nul

echo Opening Logcat...

rem Get the PID (retry, since process may not be up immediately)
set pid=
set retries=0
:retry_pid
FOR /F %%i IN ('adb shell pidof rust.mediacodec 2^>nul') DO set pid=%%i
if "%pid%"=="" (
    set /a retries+=1
    if %retries% lss 10 (
        timeout 1 > nul
        goto retry_pid
    )
)

if "%pid%"=="" (
    echo Could not find PID for rust.mediacodec. Falling back to all logs.
    adb logcat
) else (
    echo PID: %pid%
    adb logcat --pid=%pid%
)

@REM adb
