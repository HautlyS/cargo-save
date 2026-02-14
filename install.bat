@echo off
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%

copy target\release\cargo-save.exe %USERPROFILE%\.cargo\bin\cargo-save.exe
if %errorlevel% neq 0 (
    echo Failed to copy cargo-save.exe
    exit /b %errorlevel%
)

echo Installed cargo-save to %USERPROFILE%\.cargo\bin\cargo-save.exe
