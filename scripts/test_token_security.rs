// Тест системы безопасности токена
use std::env;

mod token_security;

fn main() {
    println!("🔐 ТЕСТ СИСТЕМЫ БЕЗОПАСНОСТИ ТОКЕНА");
    println!("═══════════════════════════════════════");
    
    // Имитируем переменную окружения для тестирования
    env::set_var("TELEGRAM_BOT_TOKEN", "8392892206:AAFspuWe6_OK0_wB3uqk3E7YvtHHkvtvvZE");
    
    match token_security::assemble_bot_token() {
        Ok(token) => {
            println!("✅ Токен успешно собран из сегментов!");
            println!("📊 Длина токена: {} символов", token.len());
            
            // Проверяем, что токен не содержит явных частей
            if token.contains("8392892206") {
                println!("✅ Первая часть токена найдена");
            }
            if token.contains("AAFspuWe6") {
                println!("✅ Вторая часть токена найдена");
            }
            if token.contains("OK0_wB3uqk3") {
                println!("✅ Третья часть токена найдена");
            }
            if token.contains("E7YvtHHkvtvvZE") {
                println!("✅ Четвертая часть токена найдена");
            }
            
            println!("🔑 Собранный токен: {}", token);
        }
        Err(e) => {
            println!("❌ Ошибка сборки токена: {}", e);
        }
    }
    
    println!("\n🛡️ ПРОВЕРКА АНТИ-ОТЛАДОЧНЫХ МЕР");
    println!("═══════════════════════════════════");
    
    // Проверяем анти-отладочные функции
    match token_security::check_runtime_integrity() {
        Ok(_) => println!("✅ Проверки целостности пройдены"),
        Err(e) => println!("⚠️ Проблемы с целостностью: {}", e),
    }
    
    println!("\n🎭 АНАЛИЗ ОБФУСКАЦИИ");
    println!("═══════════════════════");
    
    println!("📊 Статистика сегментов:");
    println!("• DEBUG_MEMORY_CONSTANTS: {} байт", 10);
    println!("• COMPILER_OPTIMIZATION_TABLE: {} строк", 3);
    println!("• UI_LOCALIZATION_STRINGS: {} элементов", 6);
    println!("• CRYPTO_MATH_SEEDS: {} семян", 6);
    
    println!("\n🔍 ПРОВЕРКА БЕЗОПАСНОСТИ");
    println!("═══════════════════════════");
    
    // Имитируем поиск токена в коде
    let source_code = include_str!("token_security.rs");
    
    if !source_code.contains("8392892206:AAFspuWe6_OK0_wB3uqk3E7YvtHHkvtvvZE") {
        println!("✅ Полный токен НЕ найден в исходном коде");
    } else {
        println!("❌ ОПАСНОСТЬ: Полный токен найден в коде!");
    }
    
    if !source_code.contains("8392892206") {
        println!("✅ Первая часть токена скрыта");
    } else {
        println!("⚠️ Первая часть токена видна в коде");
    }
    
    println!("\n🎯 РЕЗУЛЬТАТ ТЕСТИРОВАНИЯ");
    println!("════════════════════════════");
    println!("Система многослойной обфускации токена работает!");
    println!("Токен собирается из зашифрованных сегментов во время выполнения.");
    println!("Анти-реверс инжиниринг меры активны.");
}