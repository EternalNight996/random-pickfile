#[macro_use]
#[path = "./macros/ex_utils.rs"]
mod ex_utils;

use chrono::Local;
use config::{Config, File, FileFormat};
use p_utils::random;
use serde_derive::Deserialize;

use std::{
    fs,
    io::{Error, ErrorKind},
    path::{self, PathBuf},
};

#[derive(Debug, Deserialize, PartialEq)]
struct SettingsDir {
    pick_count: f32,
    name: String,
    date: String,
    workspace: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct SettingsFile {
    is_rename: bool,
    filetypes: String,
}
#[derive(Debug, Deserialize, PartialEq)]
struct SettingsMain {}
#[derive(Debug, Deserialize, PartialEq)]
struct Settings {
    dir: SettingsDir,
    file: SettingsFile,
    main: SettingsMain,
}

#[windows_subsystem = "windows"]
fn main() -> Result<(), Error> {
    output!("reading setting file.....");
    // 配置文件名
    let setting_file = "./settings";
    // 配置文件类型
    let setting_type = FileFormat::Ini;
    let settings: Settings = read_setting_fn(setting_file, setting_type)
        .try_deserialize()
        .unwrap();
    // 获取文件格式列表
    let file_type: Vec<String> = serde_json::from_str(&settings.file.filetypes).unwrap();
    let mut dir_list: Vec<(usize, PathBuf)> = vec![];
    let mut other_list: Vec<PathBuf> = vec![];
    output!("finding filetype -> {:?}", file_type);

    match fs::read_dir(settings.dir.workspace.clone()) {
        Err(e) => {
            output!("read path Error: {:#?}", e.kind());
        }
        Ok(paths) => {
            for p in paths {
                if let Ok(data) = p {
                    let d = data.path();
                    if let Some(tmp) = d.to_str() {
                        // 获取地址并匹配对应格式
                        if let Some(index) = file_type
                            .iter()
                            .position(|x| tmp.contains(&format!(".{}", *x)))
                        {
                            // 添加文件路径
                            dir_list.push((index, d));
                        } else {
                            other_list.push(d);
                        }
                    }
                }
            }
        }
    }
    // 处理已经找到的文件
    let mut list_num: usize = dir_list.len();
    if list_num > 0 {
        // 判断是否需要建立大于1的文件夹
        let f_num: f32 = list_num as f32 / settings.dir.pick_count;
        let i_num: i32 = f_num as i32;
        let num: i32 = if f_num > i_num as f32 {
            i_num + 1
        } else {
            i_num
        };
        for i in 1..num + 1 {
            let k: &str;
            #[cfg(target_os = "windows")]
            {
                k = "\\";
            }
            #[cfg(not(target_os = "windows"))]
            {
                k = "/";
            }
            let now = Local::now().format(&settings.dir.date);
            println!("{}", now);
            let p: PathBuf = path::PathBuf::from(&format!(
                "{}{}{}{}-{}",
                settings.dir.workspace, k, now, settings.dir.name, i
            ));
            // 创建文件夹
            create_dir_fn(&p, &other_list)?;
            for j in 1..settings.dir.pick_count as i32 + 1 {
                // 获取随机数范围
                let random_num = if list_num != 0 {
                    random!(0usize..list_num)
                } else {
                    0
                };
                let from_path = dir_list.swap_remove(random_num);
                let from_name = if settings.file.is_rename {
                    format!("{}.{}", j, file_type[from_path.0])
                } else {
                    from_path
                        .1
                        .display()
                        .to_string()
                        .split_once(&format!("{}{}", settings.dir.workspace, k))
                        .unwrap_or(("", ""))
                        .1
                        .to_string()
                };
                // 获取新文件位置与名称
                let to_path = path::PathBuf::from(format!("{}{}{}", p.display(), k, from_name));
                // 移动文件
                move_file_fn(
                    path::PathBuf::from(from_path.1.display().to_string()),
                    &to_path,
                )?;
                list_num = dir_list.len();
                if list_num == 0 {
                    break;
                }
            }
        }
    } else {
        let e = format!("无法找到相对应格式的文件: \n {:?}", file_type);
        return Err(Error::new(ErrorKind::Other, e));
    }
    Ok(())
}

/// 移动文件
pub fn move_file_fn(from: PathBuf, to: &PathBuf) -> Result<(), Error> {
    output!("from: {} -> to: {}", from.display(), to.display());
    fs::copy(from.clone(), to)?;
    fs::remove_file(from)?;
    Ok(())
}
/// 创建文件夹
pub fn create_dir_fn(dirpath: &PathBuf, dirlist: &Vec<PathBuf>) -> Result<(), Error> {
    if let None = dirlist.iter().find(|x| *x == dirpath) {
        fs::create_dir(dirpath)?;
        output!("成功创建文件夹: {}", dirpath.display());
    }
    Ok(())
}
/// 读取配置文件
fn read_setting_fn(setting_name: &str, fileformat: FileFormat) -> Config {
    Config::builder()
        .add_source(File::new(setting_name, fileformat))
        .build()
        .unwrap()
}
