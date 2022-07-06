use std::{
    fs,
    io::{
        self,
        prelude::{Read, Write},
    },
    path::Path,
};
use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, CompressionMethod};
/// 压缩文件夹
pub fn compress_dir(
    src_dir: &Path,
    dst: &Path,
    level: Option<i32>,
    permissions: u32,
    method: String,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let zipfile = std::fs::File::create(dst)?;
    let walkdir = WalkDir::new(src_dir);
    zip_compress_dir(
        &mut walkdir.into_iter().filter_map(|e| e.ok()),
        src_dir.to_str().unwrap(),
        zipfile,
        level,
        permissions,
        match &*method {
            ".zip" => CompressionMethod::Deflated,
            ".bz2" => CompressionMethod::Bzip2,
            ".zst" => CompressionMethod::Zstd,
            _ => CompressionMethod::Deflated,
        },
    )?;
    Ok(())
}
/// 压缩文件
pub fn compress_file(
    src_file: &Path,
    dst: &Path,
    level: Option<i32>,
    permissions: u32,
    method: String,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let zipfile = fs::File::create(dst)?;
    let walkdir = WalkDir::new(src_file);
    let prefix = src_file
        .parent()
        .map_or_else(|| "/", |p| p.to_str().unwrap());
    zip_compress_dir(
        &mut walkdir.into_iter().filter_map(|e| e.ok()),
        prefix,
        zipfile,
        level,
        permissions,
        match &*method {
            ".zip" => CompressionMethod::Deflated,
            ".bz2" => CompressionMethod::Bzip2,
            ".zst" => CompressionMethod::Zstd,
            _ => CompressionMethod::Deflated,
        },
    )?;
    Ok(())
}

/// zip压缩
fn zip_compress_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    level: Option<i32>,
    permissions: u32,
    method: CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: io::Write + io::Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .compression_level(level)
        .unix_permissions(permissions);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();
        if path.is_file() {
            println!("adding file {:?} as {:?}. . .", path, name);
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = fs::File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
/// zip解压
pub fn zip_extract(src_zip: &Path, mut dst: &Path) {
    let zipfile = fs::File::open(&src_zip).unwrap();
    let mut zip = zip::ZipArchive::new(zipfile).unwrap();
    if !dst.exists() {
        let _ = fs::create_dir_all(dst).map_err(|e| {
            println!("{}", e);
        });
    }
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        println!("Filename: {} {:?}", file.name(), file.mangled_name());
        if file.is_dir() {
            println!("file utf8 path {:?}", file.name_raw());
            let dst = dst.join(Path::new(&file.name().replace("\\", "")));
            fs::create_dir_all(dst).unwrap();
        } else {
            let file_path = dst.join(Path::new(file.name()));
            let mut dst_file = if !file_path.exists() {
                println!("sfile path {}", file_path.to_str().unwrap());
                fs::File::create(file_path).unwrap()
            } else {
                fs::File::open(file_path).unwrap()
            };
            let _ = io::copy(&mut file, &mut dst_file);
        }
    }
}
