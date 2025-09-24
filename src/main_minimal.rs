use std::sync::Arc;
use anyhow::Result;

mod config_minimal;
mod device_manager_minimal;
mod telegram_client_minimal;

use config_minimal::{BOT_TOKEN, AUTHORIZED_CHAT_ID, validate_config};
use device_manager_minimal::DeviceManager;
use telegram_client_minimal::TelegramClient;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🚀 Eye Remote Admin Bot - Minimal Version");
    
    // Проверка конфигурации
    if let Err(e) = validate_config() {
        eprintln!("❌ Ошибка конфигурации: {}", e);
        return Ok(());
    }
    
    // Инициализация менеджера устройств
    let device_manager = Arc::new(DeviceManager::new());
    
    // Инициализация устройства
    match device_manager.initialize_current_device().await {
        Ok(device_id) => {
            println!("✅ Устройство инициализировано с ID: {}", device_id);
        }
        Err(e) => {
            eprintln!("❌ Ошибка инициализации устройства: {}", e);
            return Ok(());
        }
    }
    
    // Инициализация Telegram клиента
    let telegram_client = TelegramClient::new();
    
    // Уведомление о запуске
    if let Some(device) = device_manager.get_current_device() {
        let startup_message = format!(
            "🟢 **Eye Remote Admin Bot запущен**\n📱 ID: `{}`\n💻 Имя: `{}`",
            device.device_id,
            device.device_name
        );
        
        if let Err(e) = telegram_client.send_message(AUTHORIZED_CHAT_ID, &startup_message).await {
            eprintln!("⚠️ Не удалось отправить уведомление: {}", e);
        }
    }
    
    println!("🤖 Telegram бот запущен (минимальная версия)");
    println!("📋 Доступные команды: /devices, /info, /help");
    println!("💡 Для остановки нажмите Ctrl+C");
    
    // Простой loop для поддержания работы
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        println!("✅ Бот активен...");
    }
}