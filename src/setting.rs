use config::{self, Config, ConfigError, FileFormat};
use once_cell::sync::Lazy;
use serde_derive::Deserialize;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| get_setting().unwrap());
#[derive(Debug, Deserialize, PartialEq)]
pub struct SettingsDir {
    pub pick_count: usize,
    pub name: String,
    pub date: String,
    pub workspace: String,
    pub is_compress: bool,
    pub is_save_compress_dir: bool,
    pub compress_level: i32,
    pub compress_permissions: u32,
    pub compress_method: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SettingsFile {
    pub is_rename: bool,
    pub filetypes: String,
}
#[derive(Debug, Deserialize, PartialEq)]
pub struct SettingsMain {
    pub thread: usize,
    pub cpu_core: usize,
}
#[derive(Debug, Deserialize, PartialEq)]
pub struct Settings {
    pub dir: SettingsDir,
    pub file: SettingsFile,
    pub main: SettingsMain,
}

/// 读取配置文件
fn read_setting_fn(setting_name: &str, fileformat: FileFormat) -> Config {
    Config::builder()
        .add_source(config::File::new(setting_name, fileformat))
        .build()
        .unwrap()
}
/// 获取配置文件
pub fn get_setting() -> Result<Settings, ConfigError> {
    // 配置文件名
    let setting_file = "./settings";
    // 配置文件类型
    let setting_type = FileFormat::Ini;
    read_setting_fn(setting_file, setting_type).try_deserialize()
}
