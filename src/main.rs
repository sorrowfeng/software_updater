use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

use fltk::app;
use fltk::button::Button;
use fltk::dialog;
use fltk::frame::Frame;
use fltk::group::Group;
use fltk::progress::Progress;
use fltk::window::Window;
use fltk::enums::{Align, Color, Font};
use fltk::prelude::*;

fn main() -> io::Result<()> {
    env_logger::init();
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("用法: {} <更新包路径>", args[0]);
        std::process::exit(1);
    }
    let package_path = args[1].clone();
    
    // 创建GUI应用
    let app = app::App::default();
    
    // 创建主窗口
    let mut window = Window::new(100, 100, 400, 200, "软件更新");
    window.set_color(Color::White);
    
    // 创建进度框架
    let mut frame = Frame::new(50, 50, 300, 30, "正在准备更新...");
    frame.set_align(Align::Center);
    frame.set_font(Font::HelveticaBold, 14);
    
    // 创建进度条
    let mut progress = Progress::new(50, 100, 300, 30, "");
    progress.set_minimum(0.0);
    progress.set_maximum(100.0);
    progress.set_value(0.0);
    
    // 创建状态框架
    let mut status_frame = Frame::new(50, 140, 300, 20, "");
    status_frame.set_align(Align::Center);
    
    window.end();
    window.show();
    
    // 创建通道用于线程通信
    let (sender, receiver) = mpsc::channel();
    
    // 在新线程中执行更新操作
    thread::spawn(move || {
        match perform_update(&package_path, sender) {
            Ok(_) => {
                log::info!("更新完成！");
            },
            Err(e) => {
                log::error!("更新失败: {:?}", e);
            }
        }
    });
    
    // 处理GUI事件和进度更新
    while app.wait() {
        if let Ok(msg) = receiver.try_recv() {
            match msg {
                UpdateMsg::TotalFiles(total) => {
                    frame.set_label(&format!("正在替换文件 (0/{})..", total));
                },
                UpdateMsg::Progress(current, total, file) => {
                    let percentage = (current as f64 / total as f64) * 100.0;
                    progress.set_value(percentage);
                    frame.set_label(&format!("正在替换文件 ({}/{})..", current, total));
                    status_frame.set_label(&format!("正在处理: {}", file));
                },
                UpdateMsg::Complete => {
                    frame.set_label("更新完成！");
                    status_frame.set_label("");
                    app::awake();
                    // 显示完成弹窗
                    dialog::message_default("更新完成", "软件更新已完成！");
                    app.quit();
                },
                UpdateMsg::Error(err) => {
                    frame.set_label("更新失败！");
                    status_frame.set_label(&format!("错误: {}", err));
                    app::awake();
                    // 显示错误弹窗
                    dialog::message_default("更新失败", &err);
                    app.quit();
                }
            }
        }
    }
    
    Ok(())
}

// 更新消息类型
enum UpdateMsg {
    TotalFiles(usize),
    Progress(usize, usize, String),
    Complete,
    Error(String),
}

// 执行更新操作
fn perform_update(package_path: &str, sender: mpsc::Sender<UpdateMsg>) -> io::Result<()> {
    // 获取当前可执行文件路径
    let exe_path = env::current_exe()?;
    let exe_name = exe_path.file_name().unwrap().to_str().unwrap();
    let current_dir = env::current_dir()?;
    
    // 创建临时目录用于解压
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // 解压更新包
    log::info!("正在解压更新包: {}", package_path);
    let file = fs::File::open(package_path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(temp_path)?;
    
    // 找到解压后的根目录
    let extract_root = find_extract_root(temp_path)?;
    log::info!("解压根目录: {:?}", extract_root);
    
    // 计算总文件数
    let total_files: usize = WalkDir::new(&extract_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();
    
    // 发送总文件数
    sender.send(UpdateMsg::TotalFiles(total_files)).unwrap();
    
    // 遍历解压后的文件，替换到目标目录
    log::info!("开始替换文件...");
    let mut current_file = 0;
    
    for entry in WalkDir::new(&extract_root).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            continue;
        }
        
        // 计算相对路径
        let relative_path = entry_path.strip_prefix(&extract_root)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let dest_path = current_dir.join(relative_path);
        
        // 跳过当前运行的可执行文件
        if let Some(file_name) = dest_path.file_name() {
            if file_name.to_str().unwrap() == exe_name {
                log::info!("跳过当前运行文件: {:?}", dest_path);
                continue;
            }
        }
        
        // 确保目标目录存在
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 复制文件
        current_file += 1;
        let file_name = relative_path.to_str().unwrap().to_string();
        sender.send(UpdateMsg::Progress(current_file, total_files, file_name.clone())).unwrap();
        log::info!("替换文件: {:?} -> {:?}", entry_path, dest_path);
        fs::copy(entry_path, dest_path)?;
    }
    
    // 发送完成消息
    sender.send(UpdateMsg::Complete).unwrap();
    Ok(())
}

// 查找解压后的根目录
fn find_extract_root(temp_path: &Path) -> io::Result<PathBuf> {
    let mut entries = fs::read_dir(temp_path)?;
    while let Some(entry) = entries.next() {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // 检查目录结构是否符合预期（LHandPro目录）
            if let Some(dir_name) = path.file_name() {
                if dir_name == "LHandPro" {
                    return Ok(path);
                }
            }
        }
    }
    
    // 如果没有找到LHandPro目录，返回临时目录本身
    Ok(temp_path.to_path_buf())
}
