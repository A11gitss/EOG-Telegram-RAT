# 🏗️ Руководство по сборке Eye Remote Admin Bot

## 📋 Поддерживаемые архитектуры и платформы

### Windows:
- ✅ **x86_64-pc-windows-msvc** - Windows 64-bit (рекомендуется)
- ✅ **i686-pc-windows-msvc** - Windows 32-bit
- ✅ **x86_64-pc-windows-gnu** - Windows 64-bit (MinGW)
- ✅ **i686-pc-windows-gnu** - Windows 32-bit (MinGW)

### Linux:
- ✅ **x86_64-unknown-linux-gnu** - Linux 64-bit
- ✅ **i686-unknown-linux-gnu** - Linux 32-bit
- ✅ **aarch64-unknown-linux-gnu** - Linux ARM64 (Raspberry Pi 4+)

### macOS:
- ✅ **x86_64-apple-darwin** - macOS Intel
- ✅ **aarch64-apple-darwin** - macOS Apple Silicon (M1/M2)

## 🚀 Быстрый старт

### 1. Простая сборка (текущая система):
```bash
cargo build --release
```

### 2. Использование готовых скриптов:
```bash
# Windows - запустить build.bat
build.bat

# Или PowerShell скрипты:
.\build-all.ps1      # Все Windows архитектуры
.\build-portable.ps1 # Портативная версия
.\build-cross.ps1    # Кросс-платформенная сборка
```

## 📦 Варианты сборки

### 🎯 1. Быстрая сборка (build.bat)
Интерактивное меню для выбора типа сборки:
- Быстрая сборка для текущей системы
- Сборка для всех Windows архитектур
- Портативная версия
- Кросс-платформенная сборка
- Очистка кэша

### 🏢 2. Все Windows архитектуры (build-all.ps1)
Создает сборки для всех поддерживаемых Windows архитектур:
```
builds/
├── windows-x64/eye.exe       # 64-bit MSVC
├── windows-x86/eye.exe       # 32-bit MSVC  
├── windows-x64-gnu/eye.exe   # 64-bit GNU
└── windows-x86-gnu/eye.exe   # 32-bit GNU
```

### 📱 3. Портативная версия (build-portable.ps1)
Создает готовую к использованию портативную версию:
```
portable/
├── EyeRemoteAdmin.exe  # Основной исполняемый файл
├── config.txt          # Файл конфигурации
├── start.bat           # Скрипт запуска
└── README.txt          # Инструкция
```

### 🌐 4. Кросс-платформенная сборка (build-cross.ps1)
Создает сборки для Linux и macOS (требует дополнительные инструменты).

## ⚙️ Ручная сборка для конкретной архитектуры

### Установка целевой архитектуры:
```bash
# Для Windows 32-bit
rustup target add i686-pc-windows-msvc

# Для Linux 64-bit
rustup target add x86_64-unknown-linux-gnu

# Для macOS ARM64
rustup target add aarch64-apple-darwin
```

### Компиляция:
```bash
# Windows 32-bit
cargo build --release --target i686-pc-windows-msvc

# Linux 64-bit (требует cross)
cross build --release --target x86_64-unknown-linux-gnu

# macOS ARM64
cargo build --release --target aarch64-apple-darwin
```

## 🛠️ Требования для кросс-компиляции

### Для Linux целей:
```bash
# Установка cross
cargo install cross

# Требует Docker
docker --version
```

### Для macOS целей (на Windows):
- Требуется специальные инструменты или GitHub Actions
- Рекомендуется использовать CI/CD

## 🤖 Автоматическая сборка через GitHub Actions

1. Загрузите проект в GitHub репозиторий
2. Создайте тег версии:
```bash
git tag v1.0.0
git push --tags
```
3. GitHub Actions автоматически создаст сборки для всех платформ

## 📊 Оптимизация размера

### Минимальный размер (release):
```bash
export RUSTFLAGS="-C strip=symbols -C target-cpu=native"
cargo build --release
```

### UPX сжатие (дополнительно):
```bash
# Установка UPX
# Windows: scoop install upx
# Linux: apt install upx

upx --best target/release/eye.exe
```

## 🔍 Проверка совместимости

### Проверка зависимостей Windows:
```bash
# Просмотр DLL зависимостей
dumpbin /dependents target/release/eye.exe

# Или с помощью Dependencies Walker
depends.exe target/release/eye.exe
```

### Тестирование на целевой системе:
1. Скопируйте exe файл на целевую машину
2. Убедитесь в наличии Visual C++ Redistributable (для MSVC версий)
3. Проверьте работу основных функций

## 📋 Размеры сборок (примерно)

| Архитектура | Размер (Release) | Размер (UPX) |
|-------------|------------------|---------------|
| x64 MSVC    | ~8-12 MB        | ~3-5 MB       |
| x86 MSVC    | ~7-10 MB        | ~2-4 MB       |
| x64 GNU     | ~10-15 MB       | ~4-6 MB       |
| Linux x64   | ~12-18 MB       | ~5-8 MB       |

## 🐛 Решение проблем

### Ошибка "linker not found":
```bash
# Windows
rustup toolchain install stable-x86_64-pc-windows-msvc

# Установите Visual Studio Build Tools
```

### Ошибка кросс-компиляции:
```bash
# Обновите cross
cargo install cross --force

# Проверьте Docker
docker pull ghcr.io/cross-rs/x86_64-unknown-linux-gnu:latest
```

### Ошибки Windows API:
- Убедитесь что используете последнюю версию windows crate
- Проверьте совместимость версий winapi

## 💡 Рекомендации

1. **Для большинства пользователей**: используйте `build.bat` → вариант 1 или 3
2. **Для распространения**: используйте портативную версию
3. **Для максимальной совместимости**: собирайте 32-bit версии
4. **Для серверов**: используйте Linux сборки
5. **Для автоматизации**: настройте GitHub Actions

## 📞 Поддержка

При возникновении проблем со сборкой:
1. Проверьте версию Rust: `rustc --version`
2. Обновите toolchain: `rustup update`
3. Очистите кэш: `cargo clean`
4. Проверьте зависимости в Cargo.toml