@echo off
title Eye Remote Admin - Cross Platform Build
color 0A

echo.
echo ===============================================
echo    Eye Remote Admin Bot - Cross Build
echo ===============================================
echo.

echo 🔨 Начинаем кросс-платформенную сборку...
echo.

REM Создаем папку для сборок
if not exist "builds" mkdir "builds"

REM Список целей для сборки
set TARGETS=x86_64-pc-windows-msvc i686-pc-windows-msvc x86_64-pc-windows-gnu i686-pc-windows-gnu

echo 📦 Установка целевых архитектур...
for %%t in (%TARGETS%) do (
    echo Устанавливаем %%t...
    rustup target add %%t
)

echo.
echo 🏗️ Компиляция для различных архитектур...
echo.

REM x86_64 MSVC (64-bit Windows)
echo [1/4] Компилируем для x86_64-pc-windows-msvc (Windows 64-bit MSVC)...
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% EQU 0 (
    if not exist "builds\windows-x64" mkdir "builds\windows-x64"
    copy "target\x86_64-pc-windows-msvc\release\eye.exe" "builds\windows-x64\eye.exe" >nul
    echo ✅ Windows 64-bit MSVC - готово!
) else (
    echo ❌ Ошибка сборки для Windows 64-bit MSVC
)
echo.

REM i686 MSVC (32-bit Windows)
echo [2/4] Компилируем для i686-pc-windows-msvc (Windows 32-bit MSVC)...
cargo build --release --target i686-pc-windows-msvc
if %ERRORLEVEL% EQU 0 (
    if not exist "builds\windows-x86" mkdir "builds\windows-x86"
    copy "target\i686-pc-windows-msvc\release\eye.exe" "builds\windows-x86\eye.exe" >nul
    echo ✅ Windows 32-bit MSVC - готово!
) else (
    echo ❌ Ошибка сборки для Windows 32-bit MSVC
)
echo.

REM x86_64 GNU (64-bit Windows GNU)
echo [3/4] Компилируем для x86_64-pc-windows-gnu (Windows 64-bit GNU)...
cargo build --release --target x86_64-pc-windows-gnu
if %ERRORLEVEL% EQU 0 (
    if not exist "builds\windows-x64-gnu" mkdir "builds\windows-x64-gnu"
    copy "target\x86_64-pc-windows-gnu\release\eye.exe" "builds\windows-x64-gnu\eye.exe" >nul
    echo ✅ Windows 64-bit GNU - готово!
) else (
    echo ❌ Ошибка сборки для Windows 64-bit GNU
)
echo.

REM i686 GNU (32-bit Windows GNU)
echo [4/4] Компилируем для i686-pc-windows-gnu (Windows 32-bit GNU)...
cargo build --release --target i686-pc-windows-gnu
if %ERRORLEVEL% EQU 0 (
    if not exist "builds\windows-x86-gnu" mkdir "builds\windows-x86-gnu"
    copy "target\i686-pc-windows-gnu\release\eye.exe" "builds\windows-x86-gnu\eye.exe" >nul
    echo ✅ Windows 32-bit GNU - готово!
) else (
    echo ❌ Ошибка сборки для Windows 32-bit GNU
)
echo.

echo 🎉 Сборка завершена!
echo.
echo 📊 Результаты сборки:
echo.

REM Показываем результаты
for /r "builds" %%f in (*.exe) do (
    echo    %%f
)

echo.
echo 💡 Все готовые exe файлы находятся в папке builds\
echo.
echo Нажмите любую клавишу для завершения...
pause >nul