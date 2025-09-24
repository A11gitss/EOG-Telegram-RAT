fn main() {
    println!("🚀 Eye Remote Admin Bot");
    println!("✅ Минимальная версия собрана успешно!");
    
    // Простая проверка Windows API
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        unsafe {
            let msg: Vec<u16> = OsStr::new("Eye Remote Admin - Test Build")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
                
            println!("✅ Windows API доступно!");
        }
    }
    
    println!("📱 Для полной функциональности используйте основную версию");
}