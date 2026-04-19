use panos::Config;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_load_valid_config() -> anyhow::Result<()> {
    // 1. เตรียม Temp Directory และไฟล์ TOML จำลอง
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("panos.toml");

    let toml_content = r#"
        source_dir = "."
        watch_mode = true
        debounce_seconds = 5
        
        [[rules]]
        name = "Images"
        extensions = [".jpg", ".png"]
        destination = "MyPhotos"
    "#;
    fs::write(&config_path, toml_content)?;

    // 2. รัน Loader จริงจากโปรเจกต์
    let config = Config::load(&config_path)?;

    // 3. Assert ตรวจสอบค่าต่างๆ (ตัวแปรเหล่านี้จะซิงค์กับ src/ เสมอ)
    assert_eq!(
        config.source_dir.to_str().unwrap(),
        ".",
        "Source directory ต้องตรงกับในไฟล์"
    );
    assert!(config.watch_mode, "Watch mode ต้องเป็น true");
    assert_eq!(config.debounce_seconds, 5, "Debounce seconds ต้องเป็น 5");

    // 4. ตรวจสอบ Rules และการ Sanitize อัตโนมัติ
    assert_eq!(config.rules.len(), 1, "ควรมี 1 rule");
    assert_eq!(config.rules[0].name, "Images");

    // ตรวจสอบว่า sanitize ทำงานอัตโนมัติ (ลบจุดออก)
    assert!(
        config.rules[0].extensions.contains(&"jpg".to_string()),
        "Extension ควรถูก sanitize ลบจุดออกอัตโนมัติ"
    );

    Ok(())
}

#[test]
fn test_load_config_not_found() {
    // ทดสอบกรณีไม่พบไฟล์: ระบบควรจะ Error
    let result = Config::load(Path::new("non_existent_file.toml"));
    assert!(result.is_err(), "การโหลดไฟล์ที่ไม่มีอยู่จริงควรจะคืนค่าเป็น Error");
}

#[test]
fn test_config_default_values() {
    // ทดสอบว่าค่าเริ่มต้น (Default) ซิงค์กับโปรเจกต์หลัก
    let config = Config::default();
    assert_eq!(
        config.debounce_seconds, 2,
        "ค่า Default ของ debounce_seconds ควรเป็น 2"
    );
    assert!(
        config.exclude_hidden,
        "โดยปกติควรจะ exclude_hidden เป็นค่าเริ่มต้น"
    );
}
