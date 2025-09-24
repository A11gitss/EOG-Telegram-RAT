# Скрипт для создания портативной версии
# Создает самодостаточный исполняемый файл с минимальными зависимостями

Write-Host "📦 Создание портативной версии Eye Remote Admin Bot..." -ForegroundColor Green
Write-Host ""

# Создаем папку для портативной версии
$PortableDir = "portable"
if (!(Test-Path $PortableDir)) {
    New-Item -ItemType Directory -Path $PortableDir
}

# Целевая архитектура (по умолчанию x64)
$Target = "x86_64-pc-windows-msvc"
$Description = "Windows 64-bit Portable"

Write-Host "🔨 Компилируем портативную версию..." -ForegroundColor Cyan

# Проверяем установку цели
$installed = rustup target list --installed | Select-String $Target
if (!$installed) {
    Write-Host "📦 Устанавливаем цель: $Target" -ForegroundColor Yellow
    rustup target add $Target
}

# Компилируем с оптимизациями
$env:RUSTFLAGS = "-C target-cpu=native -C link-arg=-s"
cargo build --release --target $Target

if ($LASTEXITCODE -eq 0) {
    # Копируем exe
    $SourceExe = "target\$Target\release\eye.exe"
    $DestExe = Join-Path $PortableDir "EyeRemoteAdmin.exe"
    
    if (Test-Path $SourceExe) {
        Copy-Item $SourceExe $DestExe -Force
        
        # Создаем README файл
        $ReadmeContent = @"
===========================================
    Eye Remote Admin Bot - Portable
===========================================

🚀 ИНСТРУКЦИЯ ПО ЗАПУСКУ:

1. НАСТРОЙКА БОТА:
   - Создайте бота у @BotFather в Telegram
   - Получите токен бота
   - Узнайте ваш chat_id у @userinfobot

2. КОНФИГУРАЦИЯ:
   - Отредактируйте файл config.txt
   - Укажите ваш BOT_TOKEN и CHAT_ID

3. ЗАПУСК:
   - Запустите EyeRemoteAdmin.exe
   - Бот автоматически подключится к Telegram

===========================================
    КОМАНДЫ БОТА:
===========================================

📋 СИСТЕМНАЯ ИНФОРМАЦИЯ:
/devices - список всех устройств
/info <id> - информация об устройстве
/ipinfo <id> - информация об IP

📁 ФАЙЛОВАЯ СИСТЕМА:
/listdrives <id> - список дисков
/listdirs <id> <path> - список папок
/listfiles <id> <path> - список файлов
/download <id> <file> - скачать файл

⚙️ УПРАВЛЕНИЕ:
/reroll <id> - изменить ID устройства
/help - справка по командам

===========================================
    БЕЗОПАСНОСТЬ:
===========================================

⚠️  ВАЖНО:
- Используйте только на ваших устройствах
- Не передавайте токен бота третьим лицам
- Регулярно меняйте токен бота
- Проверяйте chat_id перед использованием

===========================================

Версия: 1.0
Дата сборки: $(Get-Date -Format "yyyy-MM-dd HH:mm")
Архитектура: x86_64 Windows
"@
        
        $ReadmeFile = Join-Path $PortableDir "README.txt"
        $ReadmeContent | Out-File -FilePath $ReadmeFile -Encoding UTF8
        
        # Создаем файл конфигурации
        $ConfigContent = @"
# Конфигурация Eye Remote Admin Bot
# Отредактируйте значения ниже:

BOT_TOKEN=YOUR_BOT_TOKEN_HERE
CHAT_ID=0

# Получение токена:
# 1. Напишите @BotFather в Telegram
# 2. Создайте бота командой /newbot
# 3. Скопируйте полученный токен

# Получение Chat ID:
# 1. Напишите @userinfobot в Telegram
# 2. Скопируйте ваш User ID
"@
        
        $ConfigFile = Join-Path $PortableDir "config.txt"
        $ConfigContent | Out-File -FilePath $ConfigFile -Encoding UTF8
        
        # Создаем bat файл для запуска
        $BatchContent = @"
@echo off
title Eye Remote Admin Bot
echo 🚀 Запуск Eye Remote Admin Bot...
echo.
EyeRemoteAdmin.exe
echo.
echo ❌ Программа завершена. Нажмите любую клавишу для закрытия...
pause >nul
"@
        
        $BatchFile = Join-Path $PortableDir "start.bat"
        $BatchContent | Out-File -FilePath $BatchFile -Encoding ASCII
        
        # Получаем размер файла
        $FileSize = (Get-Item $DestExe).Length
        $FileSizeMB = [math]::Round($FileSize / 1MB, 2)
        
        Write-Host "✅ Портативная версия создана!" -ForegroundColor Green
        Write-Host "📁 Папка: $PortableDir" -ForegroundColor Yellow
        Write-Host "📄 Размер EXE: $FileSizeMB MB" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "📦 Содержимое портативной версии:" -ForegroundColor Cyan
        Get-ChildItem $PortableDir | ForEach-Object {
            Write-Host "   $($_.Name)" -ForegroundColor White
        }
        
    } else {
        Write-Host "❌ Файл не найден: $SourceExe" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Ошибка компиляции" -ForegroundColor Red
}

# Очищаем переменную окружения
$env:RUSTFLAGS = ""

Write-Host ""
Write-Host "💡 Для использования:" -ForegroundColor Yellow
Write-Host "   1. Скопируйте папку '$PortableDir' на целевую машину" -ForegroundColor Yellow
Write-Host "   2. Отредактируйте config.txt" -ForegroundColor Yellow
Write-Host "   3. Запустите start.bat или EyeRemoteAdmin.exe" -ForegroundColor Yellow