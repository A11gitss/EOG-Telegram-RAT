fn main() {
    println!("🔐 ДИАГНОСТИКА ОБНОВЛЕННЫХ КОНСТАНТ");
    println!("════════════════════════════════════════");
    
    // Новые константы
    let debug_constants: [u8; 10] = [0x2F, 0x24, 0x2E, 0x25, 0x2F, 0x2E, 0x25, 0x25, 0x27, 0x21];
    let compiler_optimization_table: [[u8; 4]; 3] = [
        [0x01, 0x7A, 0x7A, 0x7D],
        [0x48, 0x4B, 0x4E, 0x6C],
        [0x5E, 0x0D, 0x64, 0x00]
    ];
    let crypto_math_seeds: [u64; 6] = [
        0x453759767448486B, // первая часть: E7YvtHHk
        0x767476765A450000, // вторая часть: vtvvZE
        0x1234567890ABCDF1, // мусорные данные
        0x1234567890ABCDF2, // мусорные данные
        0x1234567890ABCDF3, // мусорные данные
        0x1234567890ABCDF4  // мусорные данные
    ];
    
    println!("📊 АНАЛИЗ СЕГМЕНТОВ:");
    
    // Сегмент 1: debug_constants -> "8392892206"
    print!("🔧 Сегмент 1 (DEBUG_CONSTANTS): ");
    for byte in &debug_constants {
        let decoded = byte ^ 0x17; // XOR с ключом 0x17
        print!("{}", decoded as char);
    }
    println!();
    
    // Сегмент 2: compiler_optimization_table -> ":AAFspuWe6_"
    print!("⚙️ Сегмент 2 (OPTIMIZATION_TABLE): ");
    for row in &compiler_optimization_table {
        for byte in row {
            if *byte != 0x00 { // Пропускаем нулевые байты
                let decoded = byte ^ 0x3B; // XOR с ключом 0x3B
                print!("{}", decoded as char);
            }
        }
    }
    println!();
    
    // Сегмент 3: Простое декодирование строки "OK0_wB3uqk3"
    print!("🎨 Сегмент 3 (UI_STRINGS): OK0_wB3uqk3");
    println!();
    
    // Сегмент 4: crypto seeds -> "E7YvtHHkvtvvZE"
    print!("🔢 Сегмент 4 (CRYPTO_SEEDS): ");
    for i in 0..2 {
        let seed = crypto_math_seeds[i];
        let seed_bytes = seed.to_be_bytes();
        
        for &byte in &seed_bytes {
            if byte != 0 && byte.is_ascii_graphic() {
                print!("{}", byte as char);
            }
        }
    }
    println!();
    
    println!("\n🔗 ИТОГОВАЯ СБОРКА:");
    print!("Полный токен: ");
    
    // Сегмент 1
    for byte in &debug_constants {
        let decoded = byte ^ 0x17;
        print!("{}", decoded as char);
    }
    
    // Сегмент 2
    for row in &compiler_optimization_table {
        for byte in row {
            if *byte != 0x00 {
                let decoded = byte ^ 0x3B;
                print!("{}", decoded as char);
            }
        }
    }
    
    // Сегмент 3
    print!("OK0_wB3uqk3");
    
    // Сегмент 4
    for i in 0..2 {
        let seed = crypto_math_seeds[i];
        let seed_bytes = seed.to_be_bytes();
        
        for &byte in &seed_bytes {
            if byte != 0 && byte.is_ascii_graphic() {
                print!("{}", byte as char);
            }
        }
    }
    
    println!();
    println!("\n🎯 Ожидаемый токен: 8392892206:AAFspuWe6_OK0_wB3uqk3E7YvtHHkvtvvZE");
}