#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::io;
use std::path::Path;
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
    Status(String),
    TotalFiles(usize),
    Progress(usize, usize, String),
    Complete,
    Error(String),
}

// 应用状态结构体
struct UpdateApp {
    package_path: String,
    target_path: Option<String>,
    zip_inner_path: String,
    total_files: usize,
    current_file: usize,
    status: String,
    status_text: String,
    current_file_name: String,
    is_complete: bool,
    error: Option<String>,
    receiver: Option<mpsc::Receiver<UpdateMsg>>,
    dict: &'static LangDict,
    delay_seconds: u64,
    start_time: Option<std::time::Instant>,
}

impl UpdateApp {
    fn new(package_path: String, lang: Language, target_path: Option<String>, zip_inner_path: String, delay_seconds: u64) -> Self {
        let dict = get_dict(lang);
        Self {
            package_path,
            target_path,
            zip_inner_path,
            total_files: 0,
            current_file: 0,
            status: dict.status_preparing.to_string(),
            status_text: dict.status_preparing.to_string(),
            current_file_name: "".to_string(),
            is_complete: false,
            error: None,
            receiver: None,
            dict,
            delay_seconds,
            start_time: None,
        }
    }
}

impl App for UpdateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // 初始化更新线程
        if self.receiver.is_none() {
            // 如果还没有开始计时，记录开始时间
            if self.start_time.is_none() {
                self.start_time = Some(std::time::Instant::now());
            }
            
            // 检查是否已经过了指定的延时时间
            if let Some(start_time) = self.start_time {
                let elapsed = start_time.elapsed();
                let delay_duration = std::time::Duration::from_secs(self.delay_seconds);
                
                if elapsed < delay_duration {
                    // 如果延时未完成，显示延时状态
                    let remaining_seconds = self.delay_seconds - elapsed.as_secs();
                    let delay_msg = format!("{}", self.dict.status_starting_in(remaining_seconds));
                    self.status = delay_msg.clone();
                    self.status_text = delay_msg;
                } else {
                    // 延时完成，开始执行更新操作
                    let (sender, receiver) = mpsc::channel();
                    self.receiver = Some(receiver);
                    
                    let package_path = self.package_path.clone();
                    let target_path = self.target_path.clone();
                    let zip_inner_path = self.zip_inner_path.clone();
                    thread::spawn(move || {
                        // 直接调用perform_update，它内部会处理所有错误并发送到GUI
                        perform_update(&package_path, &target_path, &zip_inner_path, sender);
                    });
                }
            }
        }
        
        // 处理消息
        if let Some(receiver) = &self.receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    UpdateMsg::Status(text) => {
                        self.status_text = text.clone();
                        self.status = text;
                    },
                    UpdateMsg::TotalFiles(total) => {
                        self.total_files = total;
                    },
                    UpdateMsg::Progress(current, total, file) => {
                        self.current_file = current;
                        self.total_files = total;
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
        
        // 无论是否有更新，都请求重绘UI，确保界面实时更新
        ctx.request_repaint();
        
        // 设置现代化主题
        ctx.set_visuals(Visuals::light());
        
        // 创建主窗口
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 标题
                ui.label(egui::RichText::new(self.dict.title).font(egui::FontId::proportional(24.0)).color(egui::Color32::from_rgb(0, 120, 212)));
                
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
    // 初始化日志系统
    env_logger::init();
    
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    // 解析语言选项，默认为中文
    let mut lang = Language::Chinese;
    let mut lang_index = 0;
    
    // 解析参数
    let package_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "".to_string()
    };
    
    // 解析压缩包内路径，默认为根目录
    let zip_inner_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "".to_string()
    };
    
    let mut target_path = None;
    let mut delay_seconds = 0;
    
    // 解析目标路径、延时参数和语言选项
    for i in 3..args.len() {
        if parse_language(&args[i]).is_some() {
            lang_index = i;
            break;
        } else if target_path.is_none() {
            target_path = Some(args[i].clone());
        } else if delay_seconds == 0 {
            // 尝试解析为延时参数
            if let Ok(seconds) = args[i].parse::<u64>() {
                delay_seconds = seconds;
            }
        }
    }
    
    // 解析语言
    if lang_index > 0 {
        lang = match parse_language(&args[lang_index]) {
            Some(l) => l,
            None => {
                eprintln!("无效的语言选项: {}. 使用默认语言: 中文", args[lang_index]);
                Language::Chinese
            }
        };
    }
    
    // 设置窗口选项
    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_inner_size([450.0, 250.0])
        .with_resizable(false)
        .with_always_on_top();
    
    // 嵌入图标文件到可执行文件中
    let icon_bytes = include_bytes!(r"../assets/update.png");
    
    // 使用image crate解码嵌入的PNG数据
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        log::info!("成功解码嵌入的PNG图标");
        
        // 转换为RGBA格式
        let img = img.into_rgba8();
        let width = img.width();
        let height = img.height();
        let rgba = img.into_raw();
        
        // 创建IconData
        let icon_data = egui::IconData {
            rgba,
            width,
            height,
        };
        
        // 设置图标
        viewport_builder = viewport_builder.with_icon(icon_data);
        log::info!("已设置窗口图标");
    } else {
        log::error!("无法解码嵌入的PNG图标");
        // 不再绘制备用图标，直接使用默认图标
    }
    
    let options = eframe::NativeOptions {
        viewport: viewport_builder,
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
            
            // 添加微软雅黑字体
            fonts.font_data.insert(
                "system_font".to_owned(),
                egui::FontData::from_static(include_bytes!(r"C:\Windows\Fonts\msyh.ttc")),
            );
            
            // 将中文字体添加到默认字体列表
            for family in &mut fonts.families.values_mut() {
                family.insert(0, "system_font".to_owned());
            }
            
            // 应用字体配置
            cc.egui_ctx.set_fonts(fonts);
            
            Ok(Box::new(UpdateApp::new(package_path, lang, target_path, zip_inner_path, delay_seconds)))
        }),
    ).unwrap();
    
    Ok(())
}

// 执行更新操作
fn perform_update(package_path: &str, target_path: &Option<String>, zip_inner_path: &str, sender: mpsc::Sender<UpdateMsg>) {
    match actual_perform_update(package_path, target_path, zip_inner_path, sender.clone()) {
        Ok(_) => {
            log::info!("更新完成！");
            
            // 更新完成后，删除源zip文件
            if let Err(e) = std::fs::remove_file(package_path) {
                log::error!("删除源zip文件失败: {}", e);
            } else {
                log::info!("已成功删除源zip文件: {}", package_path);
            }
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
fn actual_perform_update(package_path: &str, target_path: &Option<String>, zip_inner_path: &str, sender: mpsc::Sender<UpdateMsg>) -> io::Result<()> {
    // 检查必要参数
    if package_path.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "未提供更新包路径"));
    }
    
    // 必须提供目标路径
    let target_dir = match target_path {
        Some(path) => {
            log::info!("使用命令行指定的目标目录: {:?}", path);
            Path::new(path).to_path_buf()
        },
        None => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "必须提供目标路径"));
        }
    };
    
    // 获取当前可执行文件路径
    let exe_path = env::current_exe()?;
    let exe_name = exe_path.file_name().unwrap().to_str().unwrap();
    
    // 确定目标更新目录
    let current_dir = target_dir;
    
    // 创建临时目录用于解压
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();
    
    // 打开zip文件
    log::info!("正在解压更新包: {}", package_path);
    let file = fs::File::open(package_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    // 发送解压状态
    sender.send(UpdateMsg::Status("正在解压更新包...".to_string())).unwrap();
    
    // 计算总文件数
    let total_files = archive.len();
    sender.send(UpdateMsg::TotalFiles(total_files)).unwrap();
    
    // 逐文件解压，实时更新进度
    for i in 0..total_files {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => temp_path.join(path),
            None => continue,
        };
        
        // 发送当前解压的文件名称和进度
        let file_name = file.name().to_string();
        sender.send(UpdateMsg::Progress(i + 1, total_files, file_name.clone())).unwrap();
        
        // 创建目录
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                fs::create_dir_all(p)?;
            }
        }
        
        // 跳过目录
        if (*file.name()).ends_with('/') {
            continue;
        }
        
        // 写入文件
        let mut outfile = fs::File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    
    // 找到解压后的指定目录
    let inner_path = if zip_inner_path.is_empty() {
        temp_path.to_path_buf()
    } else {
        temp_path.join(zip_inner_path)
    };
    log::info!("压缩包内指定目录路径: {:?}", inner_path);
    
    // 验证指定目录是否存在
    if !inner_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("更新包中未找到指定目录: {}", zip_inner_path)));
    }
    
    // 计算指定目录下的总文件数
    let total_files: usize = WalkDir::new(&inner_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();
    
    // 发送替换文件状态和总文件数
    sender.send(UpdateMsg::Status("正在复制文件...".to_string())).unwrap();
    sender.send(UpdateMsg::TotalFiles(total_files)).unwrap();
    
    // 遍历指定目录下的文件，复制到目标目录
    log::info!("开始复制文件...");
    let mut current_file = 0;
    
    for entry in WalkDir::new(&inner_path).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            continue;
        }
        
        // 计算相对路径（相对于指定目录）
        let relative_path = entry_path.strip_prefix(&inner_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let dest_path = current_dir.join(relative_path);
        
        // 确保目标目录存在
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 处理当前运行的可执行文件
        let final_dest_path = if let Some(file_name) = dest_path.file_name() {
            if file_name.to_str().unwrap() == exe_name {
                // 如果是当前运行的可执行文件，将其重命名为.exe.new后缀
                let new_path_str = format!("{}.new", dest_path.to_str().unwrap());
                let new_path = Path::new(&new_path_str).to_path_buf();
                log::info!("重命名当前运行文件: {:?} -> {:?}", dest_path, new_path);
                new_path
            } else {
                dest_path.clone()
            }
        } else {
            dest_path.clone()
        };
        
        // 复制文件
        current_file += 1;
        let file_name = relative_path.to_str().unwrap().to_string();
        sender.send(UpdateMsg::Progress(current_file, total_files, file_name.clone())).unwrap();
        log::info!("复制文件: {:?} -> {:?}", entry_path, final_dest_path);
        fs::copy(entry_path, final_dest_path)?;
    }
    
    // 发送完成消息
    sender.send(UpdateMsg::Complete).unwrap();
    
    Ok(())
}


