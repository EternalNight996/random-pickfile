#[macro_use]
#[path = "./ex_utils.rs"]
mod ex_utils;
#[path = "./setting.rs"]
mod p_settings;
#[path = "./p_zip.rs"]
mod p_zip;

use chrono::Local;
use p_utils::random;

use std::{
    fs,
    io::Error,
    path::{self, PathBuf},
};

const PATH_KEY: char = if cfg!(windows) { '\\' } else { '/' };
const DIR_SUFFIX_FLAG: &'static str = "_#_";
#[windows_subsystem = "windows"]
fn main() -> Result<(), Error> {
    output!("reading setting file.....");
    let settings = p_settings::get_setting().unwrap();
    // 获取当前日期
    let date: String = Local::now().format(&settings.dir.date).to_string();
    // 获取文件格式列表
    let file_type: Vec<String> = serde_json::from_str(&settings.file.filetypes).unwrap();
    output!("finding filetype -> {:?}", file_type);
    // 目标目录的前缀
    let dir_prefix: String = format!(
        "{}{}{}{}{}",
        settings.dir.workspace, PATH_KEY, date, settings.dir.name, DIR_SUFFIX_FLAG
    );
    let (dir_suffix_num, mut file_list, other_list) =
        get_dir_file(&settings.dir.workspace, file_type.clone()).unwrap();
    // 获取文件数据
    let mut list_num: usize = file_list.len();
    if list_num > 0 {
        // 判断是否需要建立大于1的文件夹
        let f_num: f32 = list_num as f32 / settings.dir.pick_count;
        let i_num: i32 = f_num as i32;
        let num: i32 = if f_num > i_num as f32 {
            i_num + 1
        } else {
            i_num
        };
        for i in dir_suffix_num..num + dir_suffix_num {
            let src_dir: PathBuf = path::PathBuf::from(&format!("{}{}", dir_prefix, i));
            let dst_dir = PathBuf::from(&format!("{}{}", src_dir.display(), ".zip"));
            // 创建文件夹
            create_dir_fn(&src_dir)?;
            for _j in 1..settings.dir.pick_count as i32 + 1 {
                // 获取随机数范围
                let random_num = if list_num != 0 {
                    random!(0usize..list_num)
                } else {
                    0
                };
                let from_path = file_list.swap_remove(random_num);
                let random_name = format!("{}{}", random!(nanoid 12), file_type[from_path.0]);
                let from_name = if settings.file.is_rename {
                    random_name
                } else {
                    from_path
                        .1
                        .file_name()
                        .and_then(|x| x.to_str())
                        .unwrap_or(&random_name)
                        .to_string()
                };
                // 获取新文件位置与名称
                let to_path =
                    path::PathBuf::from(format!("{}{}{}", src_dir.display(), PATH_KEY, from_name));
                // 移动文件
                move_file_fn(
                    path::PathBuf::from(from_path.1.display().to_string()),
                    &to_path,
                )?;
                list_num = file_list.len();
                if list_num == 0 {
                    break;
                }
            }
            // 判断是否需要压缩
            if settings.dir.is_compress && !dst_dir.exists() {
                output!(zip_compress_dir(
                    &src_dir,
                    &dst_dir,
                    settings.dir.compress_level,
                    settings.dir.is_save_compress_dir,
                    settings.dir.compress_method.clone()
                ));
            }
        }
    } else {
        let _e = format!("无法找到相对应格式的文件: \n {:?}", file_type);
        // return Err(Error::new(ErrorKind::Other, e));
        // 判断是否需要压缩
        if settings.dir.is_compress {
            println!(
                "trying check -> {} whether exists dir...",
                settings.dir.workspace
            );
            // 防止重复名称标识
            let dir_list = get_dir_file(&settings.dir.workspace, vec![])
                .and_then(|(_, _, x)| {
                    Ok(x.into_iter()
                        .filter_map(|xx| if xx.is_dir() { Some(xx) } else { None })
                        .collect::<Vec<PathBuf>>())
                })
                .unwrap();
            println!("check list {:?}", other_list);
            if dir_list.len() > 0 {
                output!("finded [`{}`] dir, trying compress of zip.", dir_list.len());
                for src_dir in dir_list {
                    let dst_dir = PathBuf::from(&format!("{}{}", src_dir.display(), ".zip"));
                    output!(zip_compress_dir(
                        &src_dir,
                        &dst_dir,
                        settings.dir.compress_level,
                        settings.dir.is_save_compress_dir,
                        settings.dir.compress_method.clone()
                    ));
                }
            } else {
                output!("Couldn't find dir of {}", settings.dir.workspace);
            }
        }
    }

    Ok(())
}

/// 获取目标目录数据
pub fn get_dir_file(
    workspace: &str,
    file_type: Vec<String>,
) -> Result<(i32, Vec<(usize, PathBuf)>, Vec<PathBuf>), Error> {
    let mut res: (i32, Vec<(usize, PathBuf)>, Vec<PathBuf>) = (1, vec![], vec![]);

    let paths = fs::read_dir(workspace)?;
    for p in paths {
        if let Ok(data) = p {
            let d = data.path();
            if let Some(tmp) = d.to_str() {
                if tmp.contains(DIR_SUFFIX_FLAG) {
                    res.0 += 1;
                };
                // 获取地址并匹配对应格式
                if let Some(index) = file_type
                    .iter()
                    .position(|x| tmp.contains(&format!(".{}", *x)))
                {
                    // 添加文件路径
                    res.1.push((index, d));
                } else {
                    res.2.push(d);
                }
            }
        }
    }
    Ok(res)
}
/// 移动文件
pub fn move_file_fn(from: PathBuf, to: &PathBuf) -> Result<(), Error> {
    output!("from: {} -> to: {}", from.display(), to.display());
    fs::rename(from.clone(), to)?;
    Ok(())
}
/// 创建文件夹
pub fn create_dir_fn(dst_path: &PathBuf) -> Result<(), Error> {
    if !dst_path.exists() {
        fs::create_dir(dst_path)?;
        output!("成功创建文件夹: {}", dst_path.display());
    }
    Ok(())
}
/// zip压缩
fn zip_compress_dir(
    src_dir: &PathBuf,
    dst_dir: &PathBuf,
    level: i32,
    src_keep: bool,
    method: String,
) -> String {
    match p_zip::compress_dir(
        src_dir.as_path(),
        dst_dir.as_path(),
        Some(level),
        0o755,
        method,
    ) {
        Err(_e) => {
            format!("Error compress zip -> {}", dst_dir.display())
        }
        Ok(_) => {
            // 判断是否移除源文件
            if !src_keep {
                let _ = fs::remove_dir_all(src_dir);
            }
            format!("Successful compress zip -> {}", dst_dir.display())
        }
    }
}
