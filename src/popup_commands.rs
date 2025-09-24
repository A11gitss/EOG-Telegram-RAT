use anyhow::{Result, Context};
use tokio::process::Command as AsyncCommand;

use crate::device_manager::DeviceManager;

/// Выполняет команду /popup - показывает всплывающее окно с сообщением
pub async fn handle_popup_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    message: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        // Используем PowerShell для показа MessageBox
        let ps_script = format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms
            [System.Windows.Forms.MessageBox]::Show('{}', 'Системное уведомление', [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
            "#,
            message.replace("'", "''") // Экранируем одинарные кавычки
        );
        
        let output = AsyncCommand::new("powershell")
            .args(&["-WindowStyle", "Hidden", "-Command", &ps_script])
            .output()
            .await
            .context("Не удалось показать popup")?;
            
        if output.status.success() {
            Ok(format!("💬 Popup показан: '{}'", message))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка показа popup: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        // Для Linux попробуем разные способы
        if let Ok(output) = AsyncCommand::new("zenity")
            .args(&["--info", "--text", message])
            .output()
            .await 
        {
            if output.status.success() {
                Ok(format!("💬 Popup показан: '{}'", message))
            } else {
                Err(anyhow::anyhow!("Zenity недоступен"))
            }
        } else if let Ok(output) = AsyncCommand::new("notify-send")
            .args(&["Системное уведомление", message])
            .output()
            .await 
        {
            if output.status.success() {
                Ok(format!("💬 Уведомление отправлено: '{}'", message))
            } else {
                Err(anyhow::anyhow!("notify-send недоступен"))
            }
        } else {
            Err(anyhow::anyhow!("Popup не поддерживается на этой системе"))
        }
    }
}

/// Выполняет команду /popup с дополнительными параметрами (тип, заголовок)
pub async fn handle_popup_advanced_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    title: &str,
    message: &str,
    popup_type: PopupType
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        let (icon, buttons) = match popup_type {
            PopupType::Info => ("Information", "OK"),
            PopupType::Warning => ("Warning", "OK"),
            PopupType::Error => ("Error", "OK"),
            PopupType::Question => ("Question", "YesNo"),
        };
        
        let ps_script = format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms
            $result = [System.Windows.Forms.MessageBox]::Show('{}', '{}', [System.Windows.Forms.MessageBoxButtons]::{}, [System.Windows.Forms.MessageBoxIcon]::{})
            Write-Output "Result: $result"
            "#,
            message.replace("'", "''"),
            title.replace("'", "''"),
            buttons,
            icon
        );
        
        let output = AsyncCommand::new("powershell")
            .args(&["-WindowStyle", "Hidden", "-Command", &ps_script])
            .output()
            .await
            .context("Не удалось показать popup")?;
            
        if output.status.success() {
            let result_output = String::from_utf8_lossy(&output.stdout);
            
            if let Some(result_line) = result_output.lines().find(|line| line.starts_with("Result:")) {
                let user_choice = result_line.replace("Result:", "").trim().to_string();
                Ok(format!("💬 Popup '{}' показан. Пользователь выбрал: {}", title, user_choice))
            } else {
                Ok(format!("💬 Popup '{}' показан", title))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка показа popup: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        // Для Linux используем zenity с разными типами
        let zenity_type = match popup_type {
            PopupType::Info => "--info",
            PopupType::Warning => "--warning", 
            PopupType::Error => "--error",
            PopupType::Question => "--question",
        };
        
        if let Ok(output) = AsyncCommand::new("zenity")
            .args(&[zenity_type, "--title", title, "--text", message])
            .output()
            .await 
        {
            if output.status.success() {
                Ok(format!("💬 Popup '{}' показан", title))
            } else {
                Err(anyhow::anyhow!("Zenity недоступен"))
            }
        } else {
            Err(anyhow::anyhow!("Popup не поддерживается на этой системе"))
        }
    }
}

/// Выполняет команду для показа критического системного уведомления
pub async fn handle_critical_popup_command(
    device_manager: &DeviceManager, 
    device_id: &str, 
    message: &str
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    device_manager.update_device_activity(device_id);
    
    #[cfg(windows)]
    {
        // Показываем критическое системное уведомление
        let ps_script = format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms
            
            # Создаем форму поверх всех окон
            $form = New-Object System.Windows.Forms.Form
            $form.Text = 'КРИТИЧЕСКОЕ СИСТЕМНОЕ УВЕДОМЛЕНИЕ'
            $form.Size = New-Object System.Drawing.Size(500,200)
            $form.StartPosition = 'CenterScreen'
            $form.TopMost = $true
            $form.FormBorderStyle = 'FixedDialog'
            $form.MaximizeBox = $false
            $form.MinimizeBox = $false
            $form.BackColor = 'Red'
            $form.ForeColor = 'White'
            
            # Добавляем текст
            $label = New-Object System.Windows.Forms.Label
            $label.Text = '{}'
            $label.Size = New-Object System.Drawing.Size(460,100)
            $label.Location = New-Object System.Drawing.Point(20,20)
            $label.Font = New-Object System.Drawing.Font('Arial', 12, [System.Drawing.FontStyle]::Bold)
            $label.TextAlign = 'MiddleCenter'
            $form.Controls.Add($label)
            
            # Добавляем кнопку
            $button = New-Object System.Windows.Forms.Button
            $button.Text = 'ПОНЯТНО'
            $button.Size = New-Object System.Drawing.Size(100,30)
            $button.Location = New-Object System.Drawing.Point(200,130)
            $button.Add_Click({{ $form.Close() }})
            $form.Controls.Add($button)
            
            # Показываем форму
            $form.ShowDialog()
            "#,
            message.replace("'", "''")
        );
        
        let output = AsyncCommand::new("powershell")
            .args(&["-WindowStyle", "Hidden", "-Command", &ps_script])
            .output()
            .await
            .context("Не удалось показать критическое уведомление")?;
            
        if output.status.success() {
            Ok(format!("🚨 КРИТИЧЕСКОЕ уведомление показано: '{}'", message))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Ошибка показа критического уведомления: {}", stderr))
        }
    }
    
    #[cfg(not(windows))]
    {
        // Для Linux используем zenity с критическим типом
        if let Ok(output) = AsyncCommand::new("zenity")
            .args(&["--error", "--title", "КРИТИЧЕСКОЕ СИСТЕМНОЕ УВЕДОМЛЕНИЕ", "--text", message])
            .output()
            .await 
        {
            if output.status.success() {
                Ok(format!("🚨 КРИТИЧЕСКОЕ уведомление показано: '{}'", message))
            } else {
                Err(anyhow::anyhow!("Zenity недоступен"))
            }
        } else {
            Err(anyhow::anyhow!("Критические уведомления не поддерживаются на этой системе"))
        }
    }
}

/// Типы popup окон
#[derive(Debug, Clone)]
pub enum PopupType {
    Info,
    Warning,
    Error,
    Question,
}