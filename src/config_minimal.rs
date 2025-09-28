// Минимальная конфигурация для тестовой сборки

pub const BOT_TOKEN: &str = "8392892206:AAFspuWe6_OK0_wB3uqk3E7YvtHHkvtvvZE"; //test
pub const AUTHORIZED_CHAT_ID: i64 = nothin;

pub fn validate_config() -> Result<(), String> {
    if BOT_TOKEN == "YOUR_BOT_TOKEN_HERE" || BOT_TOKEN.is_empty() {
        return Err("Bot token не настроен!".to_string());
    }
    
    if AUTHORIZED_CHAT_ID == 0 {
        return Err("Chat ID не настроен!".to_string());
    }
    
    Ok(())
}
