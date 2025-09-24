@echo off
REM Stealth запуск Eye Remote Bot без окон
set EYE_STEALTH=1

REM Запускаем в скрытом режиме
powershell -WindowStyle Hidden -Command "& '%~dp0eye-enhanced.exe'"

REM Альтернативный способ через Windows API
REM start /B "" "%~dp0eye-enhanced.exe"