// 语言类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

// 语言字典结构体
pub struct LangDict {
    pub title: &'static str,
    pub status_preparing: &'static str,
    pub status_complete: &'static str,
    pub status_failed: &'static str,
    pub button_ok: &'static str,
    pub usage: &'static str,
    pub lang: Language,
}

// 中文字典
pub const CHINESE: LangDict = LangDict {
    title: "软件更新",
    status_preparing: "正在准备更新...",
    status_complete: "软件更新已完成！",
    status_failed: "软件更新失败！",
    button_ok: "确定",
    usage: "用法: {} <更新包路径> [zh|en]",
    lang: Language::Chinese,
};

// 英文字典
pub const ENGLISH: LangDict = LangDict {
    title: "Software Update",
    status_preparing: "Preparing update...",
    status_complete: "Software update completed!",
    status_failed: "Software update failed!",
    button_ok: "OK",
    usage: "Usage: {} <update_package_path> [zh|en]",
    lang: Language::English,
};

impl LangDict {
    // 获取替换文件状态字符串
    pub fn status_replacing_files(&self, current: usize, total: usize) -> String {
        match self.lang {
            Language::Chinese => format!("正在替换文件 ({}/{})...", current, total),
            Language::English => format!("Replacing files ({}/{})...", current, total),
        }
    }
    
    // 获取处理文件状态字符串
    pub fn status_processing(&self, file_name: &str) -> String {
        match self.lang {
            Language::Chinese => format!("正在处理: {}", file_name),
            Language::English => format!("Processing: {}", file_name),
        }
    }
}

// 根据语言类型获取字典
pub fn get_dict(lang: Language) -> &'static LangDict {
    match lang {
        Language::Chinese => &CHINESE,
        Language::English => &ENGLISH,
    }
}

// 根据字符串解析语言类型
pub fn parse_language(lang_str: &str) -> Option<Language> {
    match lang_str.to_lowercase().as_str() {
        "zh" | "chinese" => Some(Language::Chinese),
        "en" | "english" => Some(Language::English),
        _ => None,
    }
}
