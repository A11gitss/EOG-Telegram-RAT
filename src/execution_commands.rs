use anyhow::{Result, Context};
use tokio::process::Command as AsyncCommand;
use serde_json;

use crate::device_manager::DeviceManager;

/// Выполняет команду /exec - выполняет команду в командной строке
pub async fn handle_exec_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    command: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("cmd")
            .args(&["/C", command])
            .output()
            .await
            .context("Не удалось выполнить команду")?;
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut result = format!("💻 Выполнена команда: `{}`\n\n", command);
        
        if !stdout.is_empty() {
            let preview = if stdout.len() > 1000 {
                format!("{}...\n[Вывод обрезан, всего {} символов]", &stdout[..1000], stdout.len())
            } else {
                stdout.to_string()
            };
            result.push_str(&format!("📤 Вывод:\n```\n{}\n```\n", preview));
        }
        
        if !stderr.is_empty() {
            let preview = if stderr.len() > 500 {
                format!("{}...", &stderr[..500])
            } else {
                stderr.to_string()
            };
            result.push_str(&format!("❌ Ошибки:\n```\n{}\n```", preview));
        }
        
        if stdout.is_empty() && stderr.is_empty() {
            result.push_str("✅ Команда выполнена без вывода");
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("sh")
            .args(&["-c", command])
            .output()
            .await
            .context("Не удалось выполнить команду")?;
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut result = format!("💻 Выполнена команда: `{}`\n\n", command);
        
        if !stdout.is_empty() {
            result.push_str(&format!("📤 Вывод:\n```\n{}\n```\n", stdout));
        }
        
        if !stderr.is_empty() {
            result.push_str(&format!("❌ Ошибки:\n```\n{}\n```", stderr));
        }
        
        Ok(result)
    }
}

/// Выполняет команду /start - запускает программу или файл
pub async fn handle_start_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    target: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("cmd")
            .args(&["/C", "start", "", target])
            .output()
            .await
            .context("Не удалось запустить программу")?;
            
        if output.status.success() {
            Ok(format!("🚀 Запущено: {}", target))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка запуска: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("xdg-open")
            .arg(target)
            .output()
            .await
            .context("Не удалось запустить программу")?;
            
        if output.status.success() {
            Ok(format!("🚀 Запущено: {}", target))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка запуска: {}", stderr))
        }
    }
}

/// Выполняет команду /apps - показывает список установленных приложений
pub async fn handle_apps_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        // Получаем список установленных программ через PowerShell
        let ps_script = r#"
            Get-ItemProperty HKLM:\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\* | 
            Select-Object DisplayName, DisplayVersion, Publisher, InstallDate |
            Where-Object {$_.DisplayName -ne $null} |
            Sort-Object DisplayName |
            ConvertTo-Json
        "#;
        
        let output = AsyncCommand::new("powershell")
            .args(&["-Command", ps_script])
            .output()
            .await
            .context("Не удалось получить список приложений")?;
            
        let apps_json = String::from_utf8_lossy(&output.stdout);
        
        if let Ok(apps) = serde_json::from_str::<serde_json::Value>(&apps_json) {
            let mut result = String::from("📱 Установленные приложения:\n\n");
            let mut count = 0;
            
            if let Some(apps_array) = apps.as_array() {
                for app in apps_array.iter().take(20) { // Показываем первые 20
                    if let (Some(name), Some(version)) = (
                        app.get("DisplayName").and_then(|v| v.as_str()),
                        app.get("DisplayVersion").and_then(|v| v.as_str())
                    ) {
                        result.push_str(&format!("• {} (v{})\n", name, version));
                        count += 1;
                    }
                }
            }
            
            if count == 20 {
                result.push_str("\n[Показаны первые 20 приложений]");
            }
            
            Ok(result)
        } else {
            // Fallback - используем обычную команду
            get_apps_fallback().await
        }
    }
    
    #[cfg(not(windows))]
    {
        // Для Linux/macOS - попробуем разные методы
        get_apps_linux().await
    }
}

/// Выполняет команду /kill - завершает процесс по имени или PID
pub async fn handle_kill_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    target: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        // Проверяем, является ли target числом (PID) или именем процесса
        if target.parse::<u32>().is_ok() {
            // Завершаем по PID
            let output = AsyncCommand::new("taskkill")
                .args(&["/F", "/PID", target])
                .output()
                .await
                .context("Не удалось завершить процесс")?;
                
            if output.status.success() {
                Ok(format!("☠️ Процесс с PID {} завершен", target))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Ошибка завершения процесса: {}", stderr))
            }
        } else {
            // Завершаем по имени
            let output = AsyncCommand::new("taskkill")
                .args(&["/F", "/IM", target])
                .output()
                .await
                .context("Не удалось завершить процесс")?;
                
            if output.status.success() {
                Ok(format!("☠️ Процесс {} завершен", target))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Ошибка завершения процесса: {}", stderr))
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("kill")
            .arg(target)
            .output()
            .await
            .context("Не удалось завершить процесс")?;
            
        if output.status.success() {
            Ok(format!("☠️ Процесс {} завершен", target))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка завершения процесса: {}", stderr))
        }
    }
}

/// Выполняет команду /processes - показывает список запущенных процессов
pub async fn handle_processes_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("tasklist")
            .args(&["/FO", "CSV", "/NH"])
            .output()
            .await
            .context("Не удалось получить список процессов")?;
            
        let processes_output = String::from_utf8_lossy(&output.stdout);
        let mut result = String::from("🔄 Запущенные процессы:\n\n");
        let mut count = 0;
        
        for line in processes_output.lines().take(25) { // Показываем первые 25
            if let Some(parts) = parse_csv_line(line) {
                if parts.len() >= 5 {
                    let name = parts[0].trim_matches('"');
                    let pid = parts[1].trim_matches('"');
                    let memory = parts[4].trim_matches('"');
                    
                    result.push_str(&format!("• {} (PID: {}, RAM: {})\n", name, pid, memory));
                    count += 1;
                }
            }
        }
        
        if count == 25 {
            result.push_str("\n[Показаны первые 25 процессов]");
        }
        
        Ok(result)
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("ps")
            .args(&["aux"])
            .output()
            .await
            .context("Не удалось получить список процессов")?;
            
        let processes_output = String::from_utf8_lossy(&output.stdout);
        let mut result = String::from("🔄 Запущенные процессы:\n\n");
        
        for line in processes_output.lines().skip(1).take(20) { // Пропускаем заголовок, показываем 20
            result.push_str(&format!("{}\n", line));
        }
        
        Ok(result)
    }
}

// Вспомогательные функции
async fn get_apps_fallback() -> Result<String> {
    let output = AsyncCommand::new("wmic")
        .args(&["product", "get", "name,version", "/format:csv"])
        .output()
        .await
        .context("Не удалось получить список приложений")?;
        
    let apps_output = String::from_utf8_lossy(&output.stdout);
    let mut result = String::from("📱 Установленные приложения:\n\n");
    
    for line in apps_output.lines().skip(1).take(15) { // Пропускаем заголовок
        if !line.trim().is_empty() && line.contains(',') {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 && !parts[1].trim().is_empty() {
                result.push_str(&format!("• {} ({})\n", parts[1].trim(), parts[2].trim()));
            }
        }
    }
    
    Ok(result)
}

async fn get_apps_linux() -> Result<String> {
    // Попробуем разные менеджеры пакетов
    if let Ok(output) = AsyncCommand::new("dpkg").args(&["--list"]).output().await {
        let apps_output = String::from_utf8_lossy(&output.stdout);
        let mut result = String::from("📱 Установленные пакеты:\n\n");
        
        for line in apps_output.lines().take(20) {
            if line.starts_with("ii") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    result.push_str(&format!("• {} ({})\n", parts[1], parts[2]));
                }
            }
        }
        
        Ok(result)
    } else {
        Ok("📱 Список приложений недоступен на этой системе".to_string())
    }
}

fn parse_csv_line(line: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                result.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    
    if !current.is_empty() {
        result.push(current);
    }
    
    Some(result)
}