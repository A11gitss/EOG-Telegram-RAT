# Скрипт для кросс-компиляции под Linux и другие платформы
# ВНИМАНИЕ: Требует установки дополнительных инструментов

Write-Host "🌐 Кросс-компиляция Eye Remote Admin Bot под различные платформы..." -ForegroundColor Green
Write-Host ""

# Создаем папку для кросс-сборок
$CrossDir = "cross-builds"
if (!(Test-Path $CrossDir)) {
    New-Item -ItemType Directory -Path $CrossDir
}

# Список целевых платформ
$CrossTargets = @(
    @{Name="x86_64-unknown-linux-gnu"; Description="Linux 64-bit"; Folder="linux-x64"; Extension=""},
    @{Name="i686-unknown-linux-gnu"; Description="Linux 32-bit"; Folder="linux-x86"; Extension=""},
    @{Name="aarch64-unknown-linux-gnu"; Description="Linux ARM64"; Folder="linux-arm64"; Extension=""},
    @{Name="x86_64-apple-darwin"; Description="macOS 64-bit"; Folder="macos-x64"; Extension=""},
    @{Name="aarch64-apple-darwin"; Description="macOS ARM64 (M1/M2)"; Folder="macos-arm64"; Extension=""}
)

Write-Host "⚠️  ТРЕБОВАНИЯ ДЛЯ КРОСС-КОМПИЛЯЦИИ:" -ForegroundColor Yellow
Write-Host "   - Docker Desktop (для Linux)" -ForegroundColor Yellow
Write-Host "   - Cross compilation tools" -ForegroundColor Yellow
Write-Host "   - Или используйте GitHub Actions" -ForegroundColor Yellow
Write-Host ""

# Проверяем наличие Docker
$DockerInstalled = $false
try {
    docker --version | Out-Null
    $DockerInstalled = $true
    Write-Host "✅ Docker найден" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker не установлен" -ForegroundColor Red
}

# Проверяем наличие cross
$CrossInstalled = $false
try {
    cross --version | Out-Null
    $CrossInstalled = $true
    Write-Host "✅ Cross найден" -ForegroundColor Green
} catch {
    Write-Host "❌ Cross не установлен" -ForegroundColor Red
    Write-Host "   Установите: cargo install cross" -ForegroundColor Yellow
}

Write-Host ""

if ($CrossInstalled) {
    Write-Host "🔨 Начинаем кросс-компиляцию..." -ForegroundColor Cyan
    
    foreach ($Target in $CrossTargets) {
        $TargetName = $Target.Name
        $Description = $Target.Description
        $Folder = $Target.Folder
        $Extension = $Target.Extension
        
        Write-Host "🔨 Компилируем: $Description" -ForegroundColor Cyan
        
        # Компилируем с помощью cross
        $BuildCommand = "cross build --release --target $TargetName"
        Write-Host "   Команда: $BuildCommand" -ForegroundColor Gray
        
        try {
            Invoke-Expression $BuildCommand
            
            if ($LASTEXITCODE -eq 0) {
                # Создаем папку для архитектуры
                $ArchDir = Join-Path $CrossDir $Folder
                if (!(Test-Path $ArchDir)) {
                    New-Item -ItemType Directory -Path $ArchDir
                }
                
                # Определяем имя исполняемого файла
                $ExeName = "eye$Extension"
                $SourceExe = "target\$TargetName\release\$ExeName"
                $DestExe = Join-Path $ArchDir $ExeName
                
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
        } catch {
            Write-Host "   ❌ Ошибка выполнения cross для $Description" -ForegroundColor Red
        }
        
        Write-Host ""
    }
} else {
    Write-Host "⏭️  Пропускаем кросс-компиляцию (нет необходимых инструментов)" -ForegroundColor Yellow
}

# Создаем GitHub Actions workflow
$WorkflowDir = ".github\workflows"
if (!(Test-Path $WorkflowDir)) {
    New-Item -ItemType Directory -Path $WorkflowDir -Force
}

$WorkflowContent = @"
name: Cross-platform Build

on:
  push:
    tags:
      - 'v*'
  pull_request:
    branches: [ main ]

jobs:
  build:
    name: Build for `${{ matrix.target }}`
    runs-on: `${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            name: windows-x64
          - target: i686-pc-windows-msvc
            os: windows-latest
            name: windows-x86
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: linux-x64
          - target: i686-unknown-linux-gnu
            os: ubuntu-latest
            name: linux-x86
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            name: linux-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: macos-x64
          - target: aarch64-apple-darwin
            os: macos-latest
            name: macos-arm64

    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: `${{ matrix.target }}
        override: true
    
    - name: Install cross
      if: matrix.os == 'ubuntu-latest'
      run: cargo install cross
    
    - name: Build
      run: |
        if [ "`${{ matrix.os }}" == "ubuntu-latest" ]; then
          cross build --release --target `${{ matrix.target }}
        else
          cargo build --release --target `${{ matrix.target }}
        fi
      shell: bash
    
    - name: Prepare artifacts
      run: |
        mkdir -p artifacts
        if [ "`${{ matrix.os }}" == "windows-latest" ]; then
          cp target/`${{ matrix.target }}/release/eye.exe artifacts/eye-`${{ matrix.name }}.exe
        else
          cp target/`${{ matrix.target }}/release/eye artifacts/eye-`${{ matrix.name }}
        fi
      shell: bash
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: eye-`${{ matrix.name }}
        path: artifacts/*

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    
    steps:
    - name: Download all artifacts
      uses: actions/download-artifact@v3
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: |
          eye-*/eye-*
        draft: false
        prerelease: false
      env:
        GITHUB_TOKEN: `${{ secrets.GITHUB_TOKEN }}
"@

$WorkflowFile = Join-Path $WorkflowDir "cross-build.yml"
$WorkflowContent | Out-File -FilePath $WorkflowFile -Encoding UTF8

Write-Host "🎉 Кросс-компиляция настроена!" -ForegroundColor Green
Write-Host "📁 GitHub Actions workflow создан: $WorkflowFile" -ForegroundColor Yellow
Write-Host ""
Write-Host "💡 Для автоматической сборки:" -ForegroundColor Yellow
Write-Host "   1. Загрузите код в GitHub репозиторий" -ForegroundColor Yellow
Write-Host "   2. Создайте тег: git tag v1.0.0 && git push --tags" -ForegroundColor Yellow
Write-Host "   3. GitHub Actions автоматически создаст сборки для всех платформ" -ForegroundColor Yellow