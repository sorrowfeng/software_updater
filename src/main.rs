use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

use eframe::{App, Frame};
use egui::{CentralPanel, Context, ProgressBar, Visuals};

mod language;
use language::{Language, LangDict, get_dict, parse_language};

// 更新消息类型
enum UpdateMsg {
    TotalFiles(usize),
    Progress(usize, usize, String),
    Complete,
    Error(String),
}

// 应用状态结构体
struct UpdateApp {
    package_path: String,
    total_files: usize,
    current_file: usize,
    status: String,
    current_file_name: String,
    is_complete: bool,
    error: Option<String>,
    receiver: Option<mpsc::Receiver<UpdateMsg>>,
    dict: &'static LangDict,
}

impl UpdateApp {
    fn new(package_path: String, lang: Language) -> Self {
        let dict = get_dict(lang);
        Self {
            package_path,
            total_files: 0,
            current_file: 0,
            status: dict.status_preparing.to_string(),
            current_file_name: "".to_string(),
            is_complete: false,
            error: None,
            receiver: None,
            dict,
        }
    }
}

impl App for UpdateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // 初始化更新线程
        if self.receiver.is_none() {
            let (sender, receiver) = mpsc::channel();
            self.receiver = Some(receiver);
            
            let package_path = self.package_path.clone();
            thread::spawn(move || {
                // 直接调用perform_update，它内部会处理所有错误并发送到GUI
                perform_update(&package_path, sender);
            });
        }
        
        // 处理消息
        let mut has_updates = false;
        if let Some(receiver) = &self.receiver {
            while let Ok(msg) = receiver.try_recv() {
                has_updates = true;
                match msg {
                    UpdateMsg::TotalFiles(total) => {
                        self.total_files = total;
                        self.status = self.dict.status_replacing_files(0, total);
                    },
                    UpdateMsg::Progress(current, total, file) => {
                        self.current_file = current;
                        self.total_files = total;
                        self.status = self.dict.status_replacing_files(current, total);
                        self.current_file_name = file;
                    },
                    UpdateMsg::Complete => {
                        self.status = self.dict.status_complete.to_string();
                        self.current_file = self.total_files;
                        self.current_file_name = "".to_string();
                        self.is_complete = true;
                    },
                    UpdateMsg::Error(err) => {
                        self.status = self.dict.status_failed.to_string();
                        self.error = Some(err);
                    },
                }
            }
        }
        
        // 如果有更新，请求重绘UI
        if has_updates {
            ctx.request_repaint();
        }
        
        // 设置现代化主题
        ctx.set_visuals(Visuals::light());
        
        // 创建主窗口
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 标题
                ui.label(egui::RichText::new(self.dict.title).font(egui::FontId::proportional(20.0)).color(egui::Color32::from_rgb(0, 120, 212)));
                
                ui.add_space(20.0);
                
                // 状态信息
                ui.label(egui::RichText::new(&self.status).font(egui::FontId::proportional(14.0)));
                
                ui.add_space(8.0);
                
                // 进度条
                let progress = if self.total_files > 0 {
                    self.current_file as f32 / self.total_files as f32
                } else {
                    0.0
                };
                
                ui.add(ProgressBar::new(progress).show_percentage());
                
                ui.add_space(10.0);
                
                // 当前处理的文件
                if !self.current_file_name.is_empty() {
                    ui.label(egui::RichText::new(self.dict.status_processing(&self.current_file_name))
                        .font(egui::FontId::proportional(12.0))
                        .color(egui::Color32::GRAY));
                }
                
                // 显示完成或错误信息
                if self.is_complete {
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new(self.dict.status_complete).font(egui::FontId::proportional(16.0)).color(egui::Color32::GREEN));
                    ui.add_space(15.0);
                    if ui.add(egui::Button::new(self.dict.button_ok).min_size(egui::Vec2::new(80.0, 30.0))).clicked() {
                        std::process::exit(0);
                    }
                }
                
                if let Some(error) = &self.error {
                    ui.add_space(15.0);
                    ui.label(egui::RichText::new(self.dict.status_failed).font(egui::FontId::proportional(16.0)).color(egui::Color32::RED));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(error).font(egui::FontId::proportional(13.0)));
                    ui.add_space(15.0);
                    if ui.add(egui::Button::new(self.dict.button_ok).min_size(egui::Vec2::new(80.0, 30.0))).clicked() {
                        std::process::exit(1);
                    }
                }
            });
        });
    }
}

fn main() -> io::Result<()> {
    env_logger::init();
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: {} <更新包路径> [zh|en]", args[0]);
        std::process::exit(1);
    }
    
    let package_path = args[1].clone();
    
    // 解析语言选项，默认为中文
    let lang = if args.len() > 2 {
        match parse_language(&args[2]) {
            Some(l) => l,
            None => {
                eprintln!("无效的语言选项: {}. 使用默认语言: 中文", args[2]);
                Language::Chinese
            }
        }
    } else {
        Language::Chinese
    };
    
    // 设置窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 250.0])
            .with_resizable(false),
        centered: true,
        ..Default::default()
    };
    
    // 获取字典以设置窗口标题
    let dict = get_dict(lang);
    
    // 运行应用
    eframe::run_native(
        dict.title,
        options,
        Box::new(move |cc| {
            // 配置字体以支持中文显示
            let mut fonts = egui::FontDefinitions::default();
            
            // 添加系统默认中文字体
            fonts.font_data.insert(
                "system_font".to_owned(),
                egui::FontData::from_static(include_bytes!(r"C:\Windows\Fonts\simsun.ttc")),
            );
            
            // 将中文字体添加到默认字体列表
            for family in &mut fonts.families.values_mut() {
                family.insert(0, "system_font".to_owned());
            }
            
            // 应用字体配置
            cc.egui_ctx.set_fonts(fonts);
            
            Ok(Box::new(UpdateApp::new(package_path, lang)))
        }),
    ).unwrap();
    
    Ok(())
}

// 执行更新操作
fn perform_update(package_path: &str, sender: mpsc::Sender<UpdateMsg>) {
    match actual_perform_update(package_path, sender.clone()) {
        Ok(_) => {
            log::info!("更新完成！");
        },
        Err(e) => {
            let error_msg = e.to_string();
            log::error!("更新失败: {}", error_msg);
            if let Err(send_err) = sender.send(UpdateMsg::Error(error_msg)) {
                log::error!("无法发送错误消息: {:?}", send_err);
            }
        }
    }
}

// 实际执行更新操作的内部函数
fn actual_perform_update(package_path: &str, sender: mpsc::Sender<UpdateMsg>) -> io::Result<()> {
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
