fn main() {
    println!("🔧 ГЕНЕРАТОР ПРАВИЛЬНЫХ КОНСТАНТ");
    println!("═══════════════════════════════════");

    // Части токена
    let part1 = "8392892206";           // 10 символов
    let part2 = ":AAFspuWe6_";         // 11 символов 
    let part3 = "OK0_wB3uqk3";         // 11 символов
    let part4 = "E7YvtHHkvtvvZE";      // 14 символов

    println!("🎯 Части токена:");
    println!("Часть 1: {}", part1);
    println!("Часть 2: {}", part2);
    println!("Часть 3: {}", part3);
    println!("Часть 4: {}", part4);

    // Генерируем константы для части 1
    println!("\n📊 DEBUG_MEMORY_CONSTANTS (XOR с 0x17):");
    print!("const DEBUG_MEMORY_CONSTANTS: [u8; 10] = [");
    for (i, byte) in part1.bytes().enumerate() {
        let encoded = byte ^ 0x17;
        print!("0x{:02X}", encoded);
        if i < part1.len() - 1 {
            print!(", ");
        }
    }
    println!("];");

    // Генерируем константы для части 2
    println!("\n⚙️ COMPILER_OPTIMIZATION_TABLE (XOR с 0x3B):");
    let part2_bytes: Vec<u8> = part2.bytes().collect();
    // Разбиваем на группы по 4 байта
    println!("const COMPILER_OPTIMIZATION_TABLE: [[u8; 4]; 3] = [");
    
    // Первая группа: 4 байта
    print!("    [");
    for i in 0..4 {
        let encoded = part2_bytes[i] ^ 0x3B;
        print!("0x{:02X}", encoded);
        if i < 3 { print!(", "); }
    }
    println!("],");
    
    // Вторая группа: 4 байта
    print!("    [");
    for i in 4..8 {
        let encoded = part2_bytes[i] ^ 0x3B;
        print!("0x{:02X}", encoded);
        if i < 7 { print!(", "); }
    }
    println!("],");
    
    // Третья группа: 3 байта + 1 дополнительный
    print!("    [");
    for i in 8..11 {
        let encoded = part2_bytes[i] ^ 0x3B;
        print!("0x{:02X}", encoded);
        if i < 10 { print!(", "); }
    }
    println!(", 0x00]"); // Дополнительный нулевой байт
    println!("];");

    // Base64 для части 3
    println!("\n🎨 UI_LOCALIZATION_STRINGS (Base64):");
    // Простое base64 кодирование части 3
    let part3_bytes = part3.as_bytes();
    let base64_encoded = base64_encode(part3_bytes);
    println!("Base64 строка для '{}': {}", part3, base64_encoded);

    // Генерируем константы для части 4
    println!("\n🔢 CRYPTO_MATH_SEEDS (u64 массив):");
    println!("Часть 4: {}", part4);
    
    // Разбиваем на два u64
    let part4_bytes: Vec<u8> = part4.bytes().collect();
    
    // Первые 8 символов -> u64
    let mut bytes1 = [0u8; 8];
    for i in 0..8.min(part4_bytes.len()) {
        bytes1[i] = part4_bytes[i];
    }
    let seed1 = u64::from_be_bytes(bytes1);
    
    // Оставшиеся символы -> u64
    let mut bytes2 = [0u8; 8];
    for i in 8..part4_bytes.len() {
        bytes2[i-8] = part4_bytes[i];
    }
    let seed2 = u64::from_be_bytes(bytes2);
    
    println!("const CRYPTO_MATH_SEEDS: [u64; 6] = [");
    println!("    0x{:016X}, // первая часть: {}", seed1, &part4[0..8.min(part4.len())]);
    println!("    0x{:016X}, // вторая часть: {}", seed2, &part4[8..]);
    println!("    0x1234567890ABCDF1, // мусорные данные");
    println!("    0x1234567890ABCDF2, // мусорные данные");
    println!("    0x1234567890ABCDF3, // мусорные данные");
    println!("    0x1234567890ABCDF4  // мусорные данные");
    println!("];");
}

// Простая реализация base64 encoding
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in input.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(CHARS[((b >> 18) & 63) as usize] as char);
        result.push(CHARS[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(b & 63) as usize] as char } else { '=' });
    }
    
    result
}