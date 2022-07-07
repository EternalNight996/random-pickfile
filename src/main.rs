// #![windows_subsystem = "windows"]
#[macro_use]
#[path = "./ex_utils.rs"]
mod ex_utils;
#[path = "./setting.rs"]
mod p_settings;
#[path = "./p_zip.rs"]
mod p_zip;

use chrono::Local;
use futures::{executor::block_on, future::join_all, TryStreamExt};
use once_cell::sync::Lazy;
use p_settings::SETTINGS;
use p_utils::random;
use std::{
    io::ErrorKind,
    path::{self, Path, PathBuf},
    time::Instant,
};

use tokio::{fs, io::Error, sync::mpsc};
// 判断系统路径类型
const PATH_KEY: char = if cfg!(windows) { '\\' } else { '/' };
// 特殊的标识，方便去重
const DIR_SUFFIX_FLAG: &'static str = "_#_";
// 主要的数据体
static DATA: Lazy<Box<Data>> = Lazy::new(|| Box::new(Data::new()));

#[derive(Default)]
struct Data {
    // 目标目录的前缀
    dir_prefix: String,
    // 获取文件格式列表
    file_type: Vec<String>,
    // 扫描标识的文件夹
    suffix_flag_dir_count: usize,
    // 文件列表
    file_list: Vec<(usize, PathBuf)>,
    // 文件数量
    file_count: usize,
    // 文件夹列表
    dir_list: Vec<PathBuf>,
    // 文件夹数量
    dir_count: usize,
}
impl Data {
    fn new() -> Self {
        let mut x = Data::default();
        x.dir_prefix = format!(
            "{}{}{}{}{}",
            SETTINGS.dir.workspace,
            PATH_KEY,
            Local::now().format(&SETTINGS.dir.date).to_string(),
            SETTINGS.dir.name,
            DIR_SUFFIX_FLAG
        );
        x.file_type = serde_json::from_str::<Vec<String>>(&SETTINGS.file.filetypes).unwrap();
        println!("1");
        let (a, b, c) = get_dir_file(&SETTINGS.dir.workspace, x.file_type.clone(), true).unwrap();
        println!("2");
        x.suffix_flag_dir_count = a;
        x.file_count = b.len();
        x.file_list = b;
        x.dir_count = c.len();
        x.dir_list = c;
        x
    }
}
// unsafe impl Send for Data {}
// unsafe impl Sync for Data {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let instant = Instant::now();
    println!("reading setting file.....");
    if check_safety_lock() {
        return Err(Error::new(
            ErrorKind::BrokenPipe,
            "进程已经在运行中, 请删除.lock文件后, 尝试执行",
        ));
    } else {
        let _r = create_safety_lock().await;
        // 初始化数据
        println!("finding filetype -> {:?}", (&*DATA).file_type);
        if (&*DATA).file_list.len() > 0 {
            // 判断是否需要建立大于1的文件夹
            let f_num: f32 = (&*DATA).file_count as f32 / SETTINGS.dir.pick_count as f32;
            let i_num: i32 = f_num as i32;
            let num: usize = if f_num > i_num as f32 {
                i_num + 1
            } else {
                i_num
            } as usize;
            let mut handler = vec![];
            let (tx, mut rx) = mpsc::channel(num);
            for i in 0..num {
                if i > SETTINGS.main.thread {
                    println!("recv {}/", rx.recv().await.unwrap());
                }
                let mut cp_tx = tx.clone();
                handler.push(tokio::spawn(async move {
                    let file_list = &(*DATA).file_list;
                    let suffix_flag_dir_count = (*&DATA).suffix_flag_dir_count;
                    let dir_prefix = &*DATA.dir_prefix;
                    let file_count = (&*DATA).file_count;
                    // 目标地址
                    let dst_dir = PathBuf::from(&format!(
                        "{}{}{}",
                        dir_prefix,
                        suffix_flag_dir_count + i + 1,
                        SETTINGS.dir.compress_method
                    ));
                    // 指向的列表源开始
                    let start = i * SETTINGS.dir.pick_count;
                    let _end = start + SETTINGS.dir.pick_count;
                    let end = if _end > file_count { file_count } else { _end };
                    println!(
                        "thread -> {} start {} end {} -> count {} pick {} flagcount {}",
                        i, start, end, file_count, SETTINGS.dir.pick_count, suffix_flag_dir_count
                    );
                    // 源地址
                    let src_dir: PathBuf = path::PathBuf::from(&format!("{}{}", dir_prefix, i + 1));
                    // 创建文件夹
                    let _ = create_dir_fn(&src_dir).await;
                    for j in start..end {
                        let random_name = format!(
                            "{}{}",
                            random!(nanoid 12),
                            (&*DATA).file_type[file_list[j].0]
                        );
                        let from_path = file_list[j].1.clone();
                        let from_name = if SETTINGS.file.is_rename {
                            random_name
                        } else {
                            from_path
                                .file_name()
                                .and_then(|x| x.to_str())
                                .unwrap_or(&random_name)
                                .to_string()
                        };
                        // // 获取新文件位置与名称
                        let to_path = path::PathBuf::from(format!(
                            "{}{}{}",
                            src_dir.display(),
                            PATH_KEY,
                            from_name
                        ));
                        // // 移动文件
                        let _ = move_file_fn(
                            path::PathBuf::from(from_path.display().to_string()),
                            &to_path,
                        )
                        .await;
                    }
                    // 判断是否需要压缩
                    if SETTINGS.dir.is_compress && !dst_dir.exists() {
                        println!(
                            "{}",
                            zip_compress_dir(
                                &src_dir,
                                &dst_dir,
                                SETTINGS.dir.compress_level,
                                SETTINGS.dir.compress_permissions,
                                SETTINGS.dir.is_save_compress_dir,
                                SETTINGS.dir.compress_method.clone()
                            )
                            .await
                        );
                    }
                    // 发送完成指令
                    if SETTINGS.main.thread < num {
                        cp_tx.send(i).await.unwrap();
                    }
                }));
            }
            let _res = join_all(handler).await;
        } else {
            let _e = format!("无法找到相对应格式的文件: \n {:?}", (&*DATA).file_type);
            // 判断是否需要压缩
            if SETTINGS.dir.is_compress {
                println!(
                    "trying check -> {} whether exists dir...",
                    SETTINGS.dir.workspace
                );
                println!("check list {:?}", (&*DATA).dir_list);
                let dir_count = (&*DATA).dir_count;
                if dir_count > 0 {
                    println!(
                        "finded [`{}`] dir, trying compress of zip.",
                        (&*DATA).dir_count
                    );
                    let mut handler = vec![];
                    println!(
                        "thread -> {} start {} end {} flagcount {}",
                        dir_count, 0, dir_count, DATA.suffix_flag_dir_count
                    );
                    let thread_num = SETTINGS.main.thread;
                    let (tx, mut rx) = mpsc::channel(dir_count);
                    for i in 0..dir_count {
                        if i > thread_num {
                            println!("recv {}/", rx.recv().await.unwrap());
                        }
                        let mut cp_tx = tx.clone();
                        handler.push(tokio::spawn(async move {
                            let src_dir = &(*&DATA).dir_list[i];
                            let dst_dir = PathBuf::from(&format!(
                                "{}{}",
                                (*&DATA).dir_list[i].display(),
                                SETTINGS.dir.compress_method
                            ));
                            println!(
                                "{}",
                                zip_compress_dir(
                                    &src_dir,
                                    &dst_dir,
                                    SETTINGS.dir.compress_level,
                                    SETTINGS.dir.compress_permissions,
                                    SETTINGS.dir.is_save_compress_dir,
                                    SETTINGS.dir.compress_method.clone()
                                )
                                .await
                            );
                            if dir_count > SETTINGS.main.thread {
                                cp_tx.send(i).await.unwrap();
                            }
                        }));
                    }
                    let _res = join_all(handler).await;
                } else {
                    println!("Couldn't find dir of {}", SETTINGS.dir.workspace);
                }
            }
        }
    }
    println!("expend -> {:?}ms", instant.elapsed().as_millis());
    println!("trying remove lock of file");
    remove_safety_lock().await?;
    Ok(())
}

/// 获取目标目录数据
pub fn get_dir_file(
    workspace: &str,
    file_type: Vec<String>,
    is_random: bool,
) -> Result<(usize, Vec<(usize, PathBuf)>, Vec<PathBuf>), Error> {
    let mut res: (usize, Vec<(usize, PathBuf)>, Vec<PathBuf>) = (0, vec![], vec![]);
    for entry in std::fs::read_dir(workspace)? {
        let entry = entry?;
        let pathbuf = entry.path();
        let fname = pathbuf
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_string();
        println!("path {:?} ", pathbuf);
        if fname.contains(DIR_SUFFIX_FLAG) {
            res.0 += 1;
        }
        // 获取地址并匹配对应格式
        if let Some(index) = file_type
            .iter()
            .position(|x| fname.contains(&format!(".{}", *x)))
        {
            // 添加文件路径
            res.1.push((index, entry.path()));
        } else if pathbuf.is_dir() {
            res.2.push(entry.path());
        }
    }
    if is_random {
        {
            let tries = (res.1.len() as f64 / 2.0) as usize;
            for i in 0..tries {
                let end = res.1.len();
                let index = random!(0..end);
                let cp = res.1[index].clone();
                res.1[index] = res.1[i].clone();
                res.1[i] = cp;
            }
        }
        {
            let tries = (res.2.len() as f64 / 2.0) as usize;
            for i in 0..tries {
                let end = res.2.len();
                let index = random!(0..end);
                let cp = res.2[index].clone();
                res.2[index] = res.2[i].clone();
                res.2[i] = cp;
            }
        }
    }
    Ok(res)
}

/// 移动文件
pub async fn move_file_fn(from: PathBuf, to: &PathBuf) -> Result<(), Error> {
    println!("from: {} -> to: {}", from.display(), to.display());
    fs::rename(from.clone(), to).await?;
    Ok(())
}

/// 创建文件夹
pub async fn create_dir_fn(dst_path: &PathBuf) -> Result<(), Error> {
    if !dst_path.exists() {
        fs::create_dir(dst_path).await?;
        println!("成功创建文件夹: {}", dst_path.display());
    }
    Ok(())
}

/// zip压缩
async fn zip_compress_dir(
    src_dir: &PathBuf,
    dst_dir: &PathBuf,
    level: i32,
    _permissions: u32,
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
                let _ = block_on(fs::remove_dir_all(src_dir));
            }
            format!("Successful compress zip -> {}", dst_dir.display())
        }
    }
}

/// 检查安全文件
fn check_safety_lock() -> bool {
    Path::new("./.lock").exists()
}

/// 创建安全文件
async fn create_safety_lock() -> Result<fs::File, Error> {
    let p = Path::new("./.lock");
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)
        .await
}

/// 移除安全锁
async fn remove_safety_lock() -> Result<(), Error> {
    fs::remove_file(Path::new("./.lock")).await
}
