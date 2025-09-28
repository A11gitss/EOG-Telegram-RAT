use argon2::{Argon2, PasswordHash, PasswordHasher};
use argon2::password_hash::SaltString;
use anyhow::Result;
use std::collections::HashSet;
use once_cell::sync::Lazy;

/// Соль для хеширования ChatID (в реальном приложении должна быть уникальной) Кто прочитал тот гей))))
const CHAT_SALT: &str = "eye_secure_chat_salt_2025";

/// Статический набор авторизованных хешей ChatID
static AUTHORIZED_HASHES: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut hashes = HashSet::new();
    
    // Здесь должны быть предварительно вычисленные хеши ваших ChatID
    // Пример: если ваш ChatID = 123456789, вычислите его хеш и добавьте сюда
    
    // РЕАЛЬНЫЙ ХЕШ ДЛЯ CHAT_ID 7987854520
    hashes.insert("$argon2id$v=19$m=19456,t=2,p=1$ZXllX3NlY3VyZV9jaGF0X3NhbHRfMjAyNQ$cikLXG768AOMlC9Ymt682RwKB6iSVgo9l4xAYN3ViB8".to_string());
    
    // Можете добавить дополнительные хеши для других авторизованных пользователей:
    // hashes.insert("$argon2id$v=19$m=19456,t=2,p=1$...".to_string());
    
    hashes
});

/// Структура для управления авторизацией
pub struct AuthManager {
    argon2: Argon2<'static>,
}

impl AuthManager {
    /// Создает новый экземпляр AuthManager
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
    
    /// Генерирует Argon2 хеш для ChatID (используется для первоначальной настройки)
    pub fn generate_chat_hash(&self, chat_id: i64) -> Result<String> {
        // Создаем соль из константной строки для консистентности
        let salt_bytes = CHAT_SALT.as_bytes();
        let salt = SaltString::encode_b64(salt_bytes)
            .map_err(|e| anyhow::anyhow!("Ошибка создания соли: {}", e))?;
        
        // Преобразуем ChatID в строку для хеширования
        let chat_id_str = chat_id.to_string();
        let password_hash = self.argon2
            .hash_password(chat_id_str.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Ошибка хеширования: {}", e))?;
        
        Ok(password_hash.to_string())
    }
    
    /// Проверяет авторизацию ChatID через хеш
    pub fn is_authorized_chat(&self, chat_id: i64) -> bool {
        // Вычисляем хеш для входящего ChatID
        match self.compute_chat_hash(chat_id) {
            Ok(computed_hash) => {
                // Проверяем, есть ли такой хеш в авторизованных
                AUTHORIZED_HASHES.iter().any(|authorized_hash| {
                    self.verify_hash(&computed_hash, authorized_hash)
                })
            }
            Err(_) => false,
        }
    }
    
    /// Вычисляет хеш для ChatID (внутренний метод)
    fn compute_chat_hash(&self, chat_id: i64) -> Result<String> {
        let salt_bytes = CHAT_SALT.as_bytes();
        let salt = SaltString::encode_b64(salt_bytes)
            .map_err(|e| anyhow::anyhow!("Ошибка создания соли: {}", e))?;
        
        let chat_id_str = chat_id.to_string();
        let password_hash = self.argon2
            .hash_password(chat_id_str.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Ошибка хеширования: {}", e))?;
        
        Ok(password_hash.to_string())
    }
    
    /// Проверяет соответствие хешей
    fn verify_hash(&self, computed_hash: &str, stored_hash: &str) -> bool {
        if let (Ok(computed), Ok(stored)) = (
            PasswordHash::new(computed_hash),
            PasswordHash::new(stored_hash)
        ) {
            // Compare hashes без раскрытия оригинальных данных
            computed.hash == stored.hash && computed.salt == stored.salt
        } else {
            false
        }
    }
    
    /// Добавляет новый авторизованный хеш (для администрирования)
    pub fn add_authorized_hash(&self, hash: String) -> Result<()> {
        // В реальном приложении здесь была бы запись в защищенное хранилище
        // Для этой демонстрации просто валидируем формат хеша
        PasswordHash::new(&hash)
            .map_err(|e| anyhow::anyhow!("Неверный формат хеша: {}", e))?;
        
        // TODO: Реализовать безопасное сохранение в файл/реестр
        println!("🔐 Новый хеш готов к добавлению: {}", hash);
        Ok(())
    }
    
    /// Генерирует список хешей для массива ChatID (для первоначальной настройки)
    pub fn generate_hashes_for_chats(&self, chat_ids: &[i64]) -> Result<Vec<String>> {
        let mut hashes = Vec::new();
        
        for &chat_id in chat_ids {
            let hash = self.generate_chat_hash(chat_id)?;
            hashes.push(hash);
            
            // Очищаем ChatID из памяти сразу после использования
            // (в Rust это происходит автоматически, но мы это подчеркиваем)
        }
        
        Ok(hashes)
    }
    
    /// Безопасная очистка памяти (placeholder для будущих улучшений)
    pub fn secure_cleanup(&self) {
        // В реальном приложении здесь была бы очистка sensitive данных из памяти
        // Rust автоматически управляет памятью, но для критических приложений
        // можно использовать специальные библиотеки типа zeroize
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр менеджера авторизации
pub static AUTH_MANAGER: Lazy<AuthManager> = Lazy::new(AuthManager::new);

/// Проверяет авторизацию ChatID (главная функция для использования в коде)
pub fn is_authorized_chat(chat_id: i64) -> bool {
    AUTH_MANAGER.is_authorized_chat(chat_id)
}

/// Генерирует хеш для ChatID (для настройки)
pub fn generate_chat_hash(chat_id: i64) -> Result<String> {
    AUTH_MANAGER.generate_chat_hash(chat_id)
}

/// Получает первый авторизованный chat_id для системных уведомлений
/// Используется только для уведомлений о запуске/завершении  
/// ВНИМАНИЕ: Функция декодирует ChatID из авторизованного хеша для уведомлений
pub fn get_notification_chat_id() -> Option<i64> {
    // Вместо хардкода ChatID, получаем его из переменных окружения
    // или возвращаем None для отключения уведомлений
    
    // БЕЗОПАСНЫЙ ВАРИАНТ 1: Переменная окружения
    if let Ok(chat_id_str) = std::env::var("EYE_NOTIFICATION_CHAT_ID") {
        if let Ok(chat_id) = chat_id_str.parse::<i64>() {
            // Проверяем, что этот ChatID авторизован через хеш
            if AUTH_MANAGER.is_authorized_chat(chat_id) {
                return Some(chat_id);
            }
        }
    }
    
    // БЕЗОПАСНЫЙ ВАРИАНТ 2: Отключить уведомления (рекомендуется)
    // Для максимальной безопасности лучше отключить уведомления
    // и получать статус через команды бота
    None
    
    // НЕБЕЗОПАСНО: Прямое указание ChatID
    // Some(7987854520) // ← ЭТО НАРУШАЕТ БЕЗОПАСНОСТЬ!
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_generation() {
        let auth = AuthManager::new();
        let test_chat_id = 123456789;
        
        // Generate hash
        let hash1 = auth.generate_chat_hash(test_chat_id).unwrap();
        let hash2 = auth.generate_chat_hash(test_chat_id).unwrap();
        
        // Хеши должны быть одинаковыми для одного ChatID
        assert_eq!(hash1, hash2);
        
        // Хеш не должен содержать оригинальный ChatID
        assert!(!hash1.contains("123456789"));
    }
    
    #[test]
    fn test_authorization() {
        let auth = AuthManager::new();
        
        // Неавторизованный чат должен быть отклонен
        assert!(!auth.is_authorized_chat(999999999));
    }
}
