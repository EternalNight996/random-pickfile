# Rust多线程随机分拣并打包

***

## `使用说明`
* bin文件夹下，有可执行文件 -> ./bin/randomPickFile.exe 和 ./bin/settings.ini

## `编译环境`
```
# 先安装rust和cargo
https://www.rust-lang.org/learn/get-started
# github项目
git clone https://github.com/EternalNight996/random-pickfile.git
# 进入项目
cd random-pickfile
# 编译完后即在 .\\target\\release\\random-pickfile.exe
cargo build --release
```

---

### 提示！ p-uitils由于被墙无法下载，所以给放在本地了， settings.ini与random-pickfile.exe需要放在同一文件夹中。

---

### 多线程打包对磁盘写入速度有要求. 
### 随机分拣特定