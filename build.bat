@echo off
setlocal
cd /d "%~dp0"
if not defined CARGO (
	if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
	if exist "C:\Program Files\Rust stable MSVC 1.96\bin\cargo.exe" set "PATH=C:\Program Files\Rust stable MSVC 1.96\bin;%PATH%"
)
cd native
cargo build %*
if errorlevel 1 exit /b 1
cd ..
call gradlew.bat build
endlocal
