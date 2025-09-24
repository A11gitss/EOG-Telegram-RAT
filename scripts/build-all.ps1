# Скрипт для компиляции под разные архитектуры Windows
# Запускать из корневой папки проекта

Write-Host "🏗️ Начинаем компиляцию Eye Remote Admin Bot под разные архитектуры..." -ForegroundColor Green
Write-Host ""

# Создаем папку для готовых сборок
$BuildDir = "builds"
if (!(Test-Path $BuildDir)) {
    New-Item -ItemType Directory -Path $BuildDir
    Write-Host "📁 Создана папка: $BuildDir" -ForegroundColor Yellow
}

# Список целевых архитектур Windows
$Targets = @(
    @{Name="x86_64-pc-windows-msvc"; Description="Windows 64-bit (MSVC)"; Folder="windows-x64"},
    @{Name="i686-pc-windows-msvc"; Description="Windows 32-bit (MSVC)"; Folder="windows-x86"},
    @{Name="x86_64-pc-windows-gnu"; Description="Windows 64-bit (GNU)"; Folder="windows-x64-gnu"},
    @{Name="i686-pc-windows-gnu"; Description="Windows 32-bit (GNU)"; Folder="windows-x86-gnu"}
)

# Функция для проверки установки цели
function Test-Target {
    param($TargetName)
    $installed = rustup target list --installed | Select-String $TargetName
    return $null -ne $installed
}

# Функция для установки цели
function Install-Target {
    param($TargetName)
    Write-Host "📦 Устанавливаем цель: $TargetName" -ForegroundColor Yellow
    rustup target add $TargetName
}

# Функция для компиляции
function Build-Target {
    param($Target)
    
    $TargetName = $Target.Name
    $Description = $Target.Description
    $Folder = $Target.Folder
    
    Write-Host "🔨 Компилируем: $Description" -ForegroundColor Cyan
    
    # Проверяем установку цели
    if (!(Test-Target $TargetName)) {
        Install-Target $TargetName
    }
    
    # Компилируем в release режиме
    $BuildCommand = "cargo build --release --target $TargetName"
    Write-Host "   Команда: $BuildCommand" -ForegroundColor Gray
    
    Invoke-Expression $BuildCommand
    
    if ($LASTEXITCODE -eq 0) {
        # Создаем папку для архитектуры
        $ArchDir = Join-Path $BuildDir $Folder
        if (!(Test-Path $ArchDir)) {
            New-Item -ItemType Directory -Path $ArchDir
        }
        
        # Копируем exe файл
        $SourceExe = "target\$TargetName\release\eye.exe"
        $DestExe = Join-Path $ArchDir "eye.exe"
        
        if (Test-Path $SourceExe) {
            Copy-Item $SourceExe $DestExe -Force
            
            # Получаем размер файла
            $FileSize = (Get-Item $DestExe).Length
            $FileSizeMB = [math]::Round($FileSize / 1MB, 2)
            
            Write-Host "   ✅ Успешно! Размер: $FileSizeMB MB" -ForegroundColor Green
            Write-Host "   📁 Сохранено: $DestExe" -ForegroundColor Green
        } else {
            Write-Host "   ❌ Файл не найден: $SourceExe" -ForegroundColor Red
        }
    } else {
        Write-Host "   ❌ Ошибка компиляции для $Description" -ForegroundColor Red
    }
    
    Write-Host ""
}

# Компилируем для всех целей
foreach ($Target in $Targets) {
    Build-Target $Target
}

Write-Host "🎉 Компиляция завершена!" -ForegroundColor Green
Write-Host "📁 Все сборки находятся в папке: $BuildDir" -ForegroundColor Yellow

# Показываем итоговую информацию
Write-Host ""
Write-Host "📊 Итоговая информация:" -ForegroundColor Cyan
Get-ChildItem $BuildDir -Recurse -Filter "*.exe" | ForEach-Object {
    $Size = [math]::Round($_.Length / 1MB, 2)
    $RelativePath = $_.FullName.Replace((Get-Location).Path + "\", "")
    Write-Host "   $RelativePath - $Size MB" -ForegroundColor White
}

Write-Host ""
Write-Host "💡 Для запуска на целевой машине скопируйте соответствующий exe файл" -ForegroundColor Yellow
Write-Host "⚠️  Убедитесь, что на целевой машине установлен Visual C++ Redistributable (для MSVC версий)" -ForegroundColor Yellow