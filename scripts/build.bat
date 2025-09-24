@echo off
title Eye Remote Admin Bot - Quick Build
color 0A

echo.
echo ===============================================
echo    Eye Remote Admin Bot - Quick Build
echo ===============================================
echo.

echo [1] Быстрая сборка (текущая архитектура)
echo [2] Сборка для всех Windows архитектур
echo [3] Портативная версия
echo [4] Кросс-платформенная сборка
echo [5] Очистить кэш и пересобрать
echo [0] Выход
echo.

set /p choice="Выберите вариант (0-5): "

if "%choice%"=="1" goto quick
if "%choice%"=="2" goto all_windows
if "%choice%"=="3" goto portable
if "%choice%"=="4" goto cross
if "%choice%"=="5" goto clean
if "%choice%"=="0" goto exit
goto invalid

:quick
echo.
echo 🔨 Быстрая сборка...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    echo ✅ Сборка завершена успешно!
    echo 📁 EXE файл: target\release\eye.exe
) else (
    echo ❌ Ошибка сборки!
)
goto end

:all_windows
echo.
echo 🔨 Сборка для всех Windows архитектур...
powershell -ExecutionPolicy Bypass -File "build-all.ps1"
goto end

:portable
echo.
echo 📦 Создание портативной версии...
powershell -ExecutionPolicy Bypass -File "build-portable.ps1"
goto end

:cross
echo.
echo 🌐 Кросс-платформенная сборка...
powershell -ExecutionPolicy Bypass -File "build-cross.ps1"
goto end

:clean
echo.
echo 🧹 Очистка кэша...
cargo clean
echo ✅ Кэш очищен!
echo.
echo 🔨 Пересборка...
cargo build --release
if %ERRORLEVEL% EQU 0 (
    echo ✅ Пересборка завершена успешно!
) else (
    echo ❌ Ошибка пересборки!
)
goto end

:invalid
echo ❌ Неверный выбор!
goto end

:exit
exit /b 0

:end
echo.
echo Нажмите любую клавишу для продолжения...
pause >nul