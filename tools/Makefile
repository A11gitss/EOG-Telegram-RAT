# Makefile для Eye Remote Admin Bot
# Поддерживает различные архитектуры и платформы

# Переменные
PROJECT_NAME = eye
VERSION = 1.0.0
BUILD_DIR = builds
PORTABLE_DIR = portable
CROSS_DIR = cross-builds

# Цели Windows
WINDOWS_TARGETS = x86_64-pc-windows-msvc i686-pc-windows-msvc x86_64-pc-windows-gnu i686-pc-windows-gnu

# Цели Linux  
LINUX_TARGETS = x86_64-unknown-linux-gnu i686-unknown-linux-gnu aarch64-unknown-linux-gnu

# Цели macOS
MACOS_TARGETS = x86_64-apple-darwin aarch64-apple-darwin

.PHONY: all clean help install-targets windows linux macos portable cross

# Помощь
help:
	@echo "=== Eye Remote Admin Bot Build System ==="
	@echo ""
	@echo "Доступные команды:"
	@echo "  make build          - Быстрая сборка (текущая система)"
	@echo "  make windows        - Сборка для всех Windows архитектур"
	@echo "  make linux          - Сборка для всех Linux архитектур (требует cross)"
	@echo "  make macos          - Сборка для всех macOS архитектур"
	@echo "  make portable       - Создание портативной версии"
	@echo "  make cross          - Кросс-платформенная сборка"
	@echo "  make all            - Сборка для всех поддерживаемых платформ"
	@echo "  make clean          - Очистка файлов сборки"
	@echo "  make install-targets- Установка всех целевых архитектур"
	@echo "  make help           - Показать эту справку"
	@echo ""

# Быстрая сборка
build:
	@echo "🔨 Быстрая сборка..."
	cargo build --release
	@echo "✅ Сборка завершена: target/release/$(PROJECT_NAME).exe"

# Установка всех целевых архитектур
install-targets:
	@echo "📦 Установка целевых архитектур..."
	@for target in $(WINDOWS_TARGETS) $(LINUX_TARGETS) $(MACOS_TARGETS); do \
		echo "Устанавливаем $$target..."; \
		rustup target add $$target; \
	done
	@echo "✅ Все цели установлены!"

# Создание директорий сборки
$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

$(PORTABLE_DIR):
	@mkdir -p $(PORTABLE_DIR)

$(CROSS_DIR):
	@mkdir -p $(CROSS_DIR)

# Сборка для Windows
windows: $(BUILD_DIR)
	@echo "🏢 Сборка для Windows архитектур..."
	@for target in $(WINDOWS_TARGETS); do \
		echo "Компилируем для $$target..."; \
		rustup target add $$target 2>/dev/null || true; \
		cargo build --release --target $$target; \
		if [ $$? -eq 0 ]; then \
			arch_dir="$(BUILD_DIR)/$$(echo $$target | sed 's/pc-windows//' | sed 's/-msvc//' | sed 's/-gnu/-gnu/' | sed 's/x86_64/x64/' | sed 's/i686/x86/')"; \
			mkdir -p "$$arch_dir"; \
			cp "target/$$target/release/$(PROJECT_NAME).exe" "$$arch_dir/"; \
			echo "✅ $$target - готово!"; \
		else \
			echo "❌ Ошибка сборки для $$target"; \
		fi; \
	done

# Сборка для Linux (требует cross)
linux: $(CROSS_DIR)
	@echo "🐧 Сборка для Linux архитектур..."
	@which cross >/dev/null 2>&1 || (echo "❌ cross не установлен. Запустите: cargo install cross" && exit 1)
	@for target in $(LINUX_TARGETS); do \
		echo "Компилируем для $$target..."; \
		cross build --release --target $$target; \
		if [ $$? -eq 0 ]; then \
			arch_dir="$(CROSS_DIR)/$$(echo $$target | sed 's/unknown-linux-gnu//' | sed 's/-unknown//' | sed 's/x86_64/x64/' | sed 's/i686/x86/' | sed 's/aarch64/arm64/')"; \
			mkdir -p "$$arch_dir"; \
			cp "target/$$target/release/$(PROJECT_NAME)" "$$arch_dir/"; \
			echo "✅ $$target - готово!"; \
		else \
			echo "❌ Ошибка сборки для $$target"; \
		fi; \
	done

# Сборка для macOS
macos: $(CROSS_DIR)
	@echo "🍎 Сборка для macOS архитектур..."
	@for target in $(MACOS_TARGETS); do \
		echo "Компилируем для $$target..."; \
		rustup target add $$target 2>/dev/null || true; \
		cargo build --release --target $$target; \
		if [ $$? -eq 0 ]; then \
			arch_dir="$(CROSS_DIR)/$$(echo $$target | sed 's/apple-darwin//' | sed 's/-apple//' | sed 's/x86_64/macos-x64/' | sed 's/aarch64/macos-arm64/')"; \
			mkdir -p "$$arch_dir"; \
			cp "target/$$target/release/$(PROJECT_NAME)" "$$arch_dir/"; \
			echo "✅ $$target - готово!"; \
		else \
			echo "❌ Ошибка сборки для $$target"; \
		fi; \
	done

# Портативная версия
portable: $(PORTABLE_DIR)
	@echo "📱 Создание портативной версии..."
	@rustup target add x86_64-pc-windows-msvc 2>/dev/null || true
	@export RUSTFLAGS="-C target-cpu=native -C link-arg=-s" && \
	cargo build --release --target x86_64-pc-windows-msvc
	@if [ $$? -eq 0 ]; then \
		cp "target/x86_64-pc-windows-msvc/release/$(PROJECT_NAME).exe" "$(PORTABLE_DIR)/EyeRemoteAdmin.exe"; \
		echo "✅ Портативная версия создана!"; \
		echo "📁 Папка: $(PORTABLE_DIR)"; \
	else \
		echo "❌ Ошибка создания портативной версии"; \
	fi

# Кросс-платформенная сборка
cross: windows linux macos
	@echo "🌐 Кросс-платформенная сборка завершена!"

# Сборка для всех платформ
all: install-targets cross portable
	@echo "🎉 Сборка для всех платформ завершена!"
	@echo ""
	@echo "📊 Результаты сборки:"
	@find $(BUILD_DIR) $(CROSS_DIR) $(PORTABLE_DIR) -name "$(PROJECT_NAME)*" -type f 2>/dev/null | while read file; do \
		size=$$(du -h "$$file" | cut -f1); \
		echo "   $$file - $$size"; \
	done

# Очистка
clean:
	@echo "🧹 Очистка файлов сборки..."
	cargo clean
	rm -rf $(BUILD_DIR) $(PORTABLE_DIR) $(CROSS_DIR)
	@echo "✅ Очистка завершена!"

# Проверка кода
check:
	@echo "🔍 Проверка кода..."
	cargo check --all-targets
	@echo "✅ Проверка завершена!"

# Тестирование
test:
	@echo "🧪 Запуск тестов..."
	cargo test
	@echo "✅ Тестирование завершено!"

# Форматирование кода
fmt:
	@echo "✨ Форматирование кода..."
	cargo fmt
	@echo "✅ Форматирование завершено!"

# Линтинг
lint:
	@echo "🔍 Проверка стиля кода..."
	cargo clippy -- -D warnings
	@echo "✅ Проверка стиля завершена!"

# Информация о проекте
info:
	@echo "=== Информация о проекте ==="
	@echo "Название: $(PROJECT_NAME)"
	@echo "Версия: $(VERSION)"
	@echo "Поддерживаемые архитектуры:"
	@echo "  Windows: $(WINDOWS_TARGETS)"
	@echo "  Linux: $(LINUX_TARGETS)"
	@echo "  macOS: $(MACOS_TARGETS)"
	@echo ""
	@echo "Установленные цели Rust:"
	@rustup target list --installed
	@echo ""
	@echo "Версия Rust:"
	@rustc --version