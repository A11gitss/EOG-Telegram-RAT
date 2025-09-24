use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs as async_fs;

use crate::device_manager::DeviceManager;
use crate::config::{is_path_safe, MAX_FILE_SIZE, TEMP_DIR};

/// Выполняет команду /listdirs - получает список папок в указанной директории
pub async fn handle_listdirs_command(
    device_manager: &DeviceManager,
    device_id: &str,
    path: &str,
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    if !is_path_safe(path) {
        return Ok("❌ Указанный путь небезопасен".to_string());
    }

    device_manager.update_device_activity(device_id);

    match list_directories(path).await {
        Ok(dirs) => Ok(format_directories_list(path, &dirs)),
        Err(e) => Ok(format!("❌ Ошибка получения списка папок: {}", e))
    }
}

/// Выполняет команду /listfiles - получает список файлов в указанной директории
pub async fn handle_listfiles_command(
    device_manager: &DeviceManager,
    device_id: &str,
    path: &str,
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    if !is_path_safe(path) {
        return Ok("❌ Указанный путь небезопасен".to_string());
    }

    device_manager.update_device_activity(device_id);

    match list_files(path).await {
        Ok(files) => Ok(format_files_list(path, &files)),
        Err(e) => Ok(format!("❌ Ошибка получения списка файлов: {}", e))
    }
}

/// Выполняет команду /download - подготавливает файл для скачивания
pub async fn handle_download_command(
    device_manager: &DeviceManager,
    device_id: &str,
    file_path: &str,
) -> Result<(String, Vec<u8>)> {
    if !device_manager.is_valid_device_id(device_id) {
        return Err(anyhow::anyhow!("Устройство с ID {} не найдено", device_id));
    }

    if !is_path_safe(file_path) {
        return Err(anyhow::anyhow!("Указанный путь небезопасен"));
    }

    device_manager.update_device_activity(device_id);

    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Файл не существует: {}", file_path));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!("Указанный путь не является файлом"));
    }

    let metadata = fs::metadata(path)
        .context("Не удалось получить информацию о файле")?;

    if metadata.len() > MAX_FILE_SIZE as u64 {
        return Err(anyhow::anyhow!(
            "Файл слишком большой ({} байт). Максимальный размер: {} байт",
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }

    let file_data = async_fs::read(path).await
        .context("Не удалось прочитать файл")?;

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok((file_name, file_data))
}

/// Выполняет команду /upload - сохраняет загруженный файл
pub async fn handle_upload_command(
    device_manager: &DeviceManager,
    device_id: &str,
    target_path: &str,
    file_data: Vec<u8>,
    file_name: &str,
) -> Result<String> {
    if !device_manager.is_valid_device_id(device_id) {
        return Ok(format!("❌ Устройство с ID `{}` не найдено", device_id));
    }

    if !is_path_safe(target_path) {
        return Ok("❌ Указанный путь небезопасен".to_string());
    }

    if file_data.len() > MAX_FILE_SIZE {
        return Ok(format!(
            "❌ Файл слишком большой ({} байт). Максимальный размер: {} байт",
            file_data.len(),
            MAX_FILE_SIZE
        ));
    }

    device_manager.update_device_activity(device_id);

    // Создаем полный путь к файлу
    let target_dir = Path::new(target_path);
    let full_path = target_dir.join(file_name);

    // Создаем директорию если она не существует
    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Ok(format!("❌ Не удалось создать директорию: {}", e));
        }
    }

    // Сохраняем файл
    match async_fs::write(&full_path, &file_data).await {
        Ok(_) => {
            let size_str = format_file_size(file_data.len() as u64);
            Ok(format!(
                "✅ Файл успешно загружен:\n\
                📁 **Путь:** `{}`\n\
                📄 **Размер:** {}\n\
                🕐 **Время:** {}",
                full_path.display(),
                size_str,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ))
        }
        Err(e) => Ok(format!("❌ Ошибка сохранения файла: {}", e))
    }
}

/// Информация о директории
#[derive(Debug)]
struct DirectoryInfo {
    name: String,
    path: PathBuf,
    is_accessible: bool,
    item_count: Option<usize>,
    modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// Информация о файле
#[derive(Debug)]
struct FileInfo {
    name: String,
    path: PathBuf,
    size: u64,
    extension: Option<String>,
    modified: Option<chrono::DateTime<chrono::Utc>>,
    is_hidden: bool,
    is_readonly: bool,
}

/// Получает список директорий
async fn list_directories(path: &str) -> Result<Vec<DirectoryInfo>> {
    let dir_path = Path::new(path);
    
    if !dir_path.exists() {
        return Err(anyhow::anyhow!("Директория не существует"));
    }

    if !dir_path.is_dir() {
        return Err(anyhow::anyhow!("Указанный путь не является директорией"));
    }

    let mut directories = Vec::new();
    let mut entries = async_fs::read_dir(dir_path).await
        .context("Не удалось прочитать директорию")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Пытаемся получить количество элементов в директории
            let item_count = match async_fs::read_dir(&path).await {
                Ok(mut sub_entries) => {
                    let mut count = 0;
                    while sub_entries.next_entry().await?.is_some() {
                        count += 1;
                    }
                    Some(count)
                }
                Err(_) => None,
            };

            // Получаем время модификации
            let modified = entry.metadata().await.ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| chrono::DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0
                ));

            directories.push(DirectoryInfo {
                name,
                path: path.clone(),
                is_accessible: item_count.is_some(),
                item_count,
                modified,
            });
        }
    }

    // Сортируем по имени
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(directories)
}

/// Получает список файлов
async fn list_files(path: &str) -> Result<Vec<FileInfo>> {
    let dir_path = Path::new(path);
    
    if !dir_path.exists() {
        return Err(anyhow::anyhow!("Директория не существует"));
    }

    if !dir_path.is_dir() {
        return Err(anyhow::anyhow!("Указанный путь не является директорией"));
    }

    let mut files = Vec::new();
    let mut entries = async_fs::read_dir(dir_path).await
        .context("Не удалось прочитать директорию")?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if path.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            
            let metadata = entry.metadata().await.ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase());

            let modified = metadata.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| chrono::DateTime::from_timestamp(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64, 0
                ));

            // Проверяем атрибуты файла (Windows specific)
            let (is_hidden, is_readonly) = get_file_attributes(&path);

            files.push(FileInfo {
                name,
                path: path.clone(),
                size,
                extension,
                modified,
                is_hidden,
                is_readonly,
            });
        }
    }

    // Сортируем по имени
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(files)
}

/// Форматирует список директорий для отправки
fn format_directories_list(base_path: &str, directories: &[DirectoryInfo]) -> String {
    let mut result = format!("📁 **Папки в:** `{}`\n\n", base_path);

    if directories.is_empty() {
        result.push_str("📭 В этой директории нет папок");
        return result;
    }

    for dir in directories {
        let accessibility_icon = if dir.is_accessible { "🔓" } else { "🔒" };
        let item_count_str = match dir.item_count {
            Some(count) => format!("{} элементов", count),
            None => "Недоступно".to_string(),
        };

        let modified_str = dir.modified
            .map(|m| m.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Неизвестно".to_string());

        result.push_str(&format!(
            "{} **{}**\n\
            └ 📊 {} | 🕐 {}\n\n",
            accessibility_icon,
            dir.name,
            item_count_str,
            modified_str
        ));
    }

    result.push_str(&format!("\n📈 **Всего папок:** {}", directories.len()));
    result
}

/// Форматирует список файлов для отправки
fn format_files_list(base_path: &str, files: &[FileInfo]) -> String {
    let mut result = format!("📄 **Файлы в:** `{}`\n\n", base_path);

    if files.is_empty() {
        result.push_str("📭 В этой директории нет файлов");
        return result;
    }

    // Группируем файлы по расширениям для статистики
    let mut extension_stats: std::collections::HashMap<String, (usize, u64)> = std::collections::HashMap::new();
    let mut total_size = 0u64;

    for file in files {
        let ext = file.extension.as_deref().unwrap_or("без расширения");
        let entry = extension_stats.entry(ext.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += file.size;
        total_size += file.size;

        let size_str = format_file_size(file.size);
        let modified_str = file.modified
            .map(|m| m.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Неизвестно".to_string());

        let mut attributes = Vec::new();
        if file.is_hidden {
            attributes.push("🙈");
        }
        if file.is_readonly {
            attributes.push("🔒");
        }
        let attributes_str = if attributes.is_empty() {
            String::new()
        } else {
            format!(" {}", attributes.join(""))
        };

        let extension_icon = get_file_icon(&file.extension);

        result.push_str(&format!(
            "{} **{}**{}\n\
            └ 📏 {} | 🕐 {}\n\n",
            extension_icon,
            file.name,
            attributes_str,
            size_str,
            modified_str
        ));
    }

    // Добавляем статистику
    result.push_str(&format!(
        "\n📊 **Статистика:**\n\
        📈 Всего файлов: {}\n\
        💾 Общий размер: {}\n\n",
        files.len(),
        format_file_size(total_size)
    ));

    // Добавляем статистику по типам файлов
    if !extension_stats.is_empty() {
        result.push_str("**По типам файлов:**\n");
        let mut ext_vec: Vec<_> = extension_stats.into_iter().collect();
        ext_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0)); // Сортируем по количеству

        for (ext, (count, size)) in ext_vec.into_iter().take(5) {
            result.push_str(&format!(
                "• {}: {} шт. ({})\n",
                ext.to_uppercase(),
                count,
                format_file_size(size)
            ));
        }
    }

    result
}

/// Получает атрибуты файла (Windows specific)
fn get_file_attributes(path: &Path) -> (bool, bool) {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = fs::metadata(path) {
            let attributes = metadata.file_attributes();
            let is_hidden = (attributes & 0x2) != 0; // FILE_ATTRIBUTE_HIDDEN
            let is_readonly = (attributes & 0x1) != 0; // FILE_ATTRIBUTE_READONLY
            return (is_hidden, is_readonly);
        }
    }
    
    (false, false)
}

/// Возвращает иконку для типа файла
fn get_file_icon(extension: &Option<String>) -> &'static str {
    match extension.as_deref() {
        Some("txt") | Some("log") | Some("md") => "📝",
        Some("pdf") => "📄",
        Some("doc") | Some("docx") => "📘",
        Some("xls") | Some("xlsx") => "📊",
        Some("ppt") | Some("pptx") => "📈",
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "🖼️",
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => "🎬",
        Some("mp3") | Some("wav") | Some("flac") | Some("ogg") => "🎵",
        Some("zip") | Some("rar") | Some("7z") | Some("tar") => "📦",
        Some("exe") | Some("msi") => "⚙️",
        Some("dll") | Some("sys") => "🔧",
        Some("bat") | Some("cmd") | Some("ps1") => "💻",
        Some("py") => "🐍",
        Some("js") => "📜",
        Some("html") | Some("htm") => "🌐",
        Some("css") => "🎨",
        _ => "📄",
    }
}

/// Форматирует размер файла
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if size == 0 {
        return "0 B".to_string();
    }

    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// Создает временную директорию если она не существует
pub fn ensure_temp_directory() -> Result<PathBuf> {
    let temp_path = Path::new(TEMP_DIR);
    if !temp_path.exists() {
        fs::create_dir_all(temp_path)
            .context("Не удалось создать временную директорию")?;
    }
    Ok(temp_path.to_path_buf())
}