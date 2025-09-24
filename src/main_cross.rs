// Кросс-платформенная тестовая версия
use std::env;

const BOT_TOKEN: &str = "8392892206:AAFspuWe6_OK0_wB3uqk3E7YvtHHkvtvvZE";
const AUTHORIZED_CHAT_ID: i64 = 7987854520;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Eye Remote Admin Tool - Cross-Platform Build Test");
    println!("========================================");
    
    // Информация о платформе
    println!("🖥️  Operating System: {}", env::consts::OS);
    println!("🏗️  Architecture: {}", env::consts::ARCH);
    println!("🔧 Target Family: {}", env::consts::FAMILY);
    
    // Конфигурация
    println!("🤖 Bot Token: {}...", &BOT_TOKEN[..15]);
    println!("💬 Chat ID: {}", AUTHORIZED_CHAT_ID);
    
    // Базовая проверка системы
    match env::consts::OS {
        "windows" => {
            println!("🪟 Windows platform detected");
            println!("   - Windows API support available");
        },
        "linux" => {
            println!("🐧 Linux platform detected"); 
            println!("   - Unix system calls available");
        },
        "macos" => {
            println!("🍎 macOS platform detected");
            println!("   - Darwin system calls available");
        },
        _ => {
            println!("❓ Unknown platform: {}", env::consts::OS);
        }
    }
    
    println!("✅ Cross-platform build test completed successfully!");
    Ok(())
}