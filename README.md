# 软件更新工具

一个使用Rust编写的命令行软件更新工具，支持ZIP文件解压、实时进度显示和GUI界面。

## 功能特点

- ✅ 支持ZIP文件解压
- ✅ 支持指定压缩包内的路径
- ✅ 支持指定目标路径
- ✅ 支持中文和英文界面
- ✅ 支持GUI进度条
- ✅ 支持实时进度更新
- ✅ 支持错误处理和显示
- ✅ 支持自定义窗口图标

## 安装和编译

### 前置条件

- Rust 1.60+（推荐使用最新稳定版本）
- Cargo包管理工具

### 编译方法

```bash
# 克隆仓库
# git clone <repository-url>
# cd software_updater

# 编译
cargo build --release

# 运行
cargo run --release -- <zip_path> [zip_inner_path] <target_path> [zh|en]
```

## 使用方法

### 命令行参数

```
software_updater <zip_path> [zip_inner_path] <target_path> [zh|en]
```

- `zip_path`：更新包的路径（必填）
- `zip_inner_path`：压缩包内要复制的路径（可选，默认为根目录）
- `target_path`：目标路径（必填）
- `zh|en`：语言选项（可选，默认为中文）

### 示例

1. **使用默认设置更新软件**
   ```bash
   software_updater update.zip C:\target\directory
   ```

2. **指定压缩包内的路径**
   ```bash
   software_updater update.zip app_folder C:\target\directory
   ```

3. **使用英文界面**
   ```bash
   software_updater update.zip C:\target\directory en
   ```

4. **指定所有参数**
   ```bash
   software_updater update.zip app_folder C:\target\directory en
   ```

## 项目结构

```
software_updater/
├── src/
│   ├── main.rs         # 主程序入口和GUI界面
│   └── language.rs     # 语言支持
├── assets/
│   └── update.png      # 窗口图标
├── Cargo.toml          # 依赖配置
└── README.md           # 项目说明文档
```

## 依赖项

- `zip`：用于ZIP文件解压
- `walkdir`：用于目录遍历
- `tempfile`：用于临时目录管理
- `log` 和 `env_logger`：用于日志记录
- `eframe` 和 `egui`：用于GUI界面
- `image`：用于图像处理和图标加载

## 许可证

本项目采用Apache-2.0许可证。详见[LICENSE](LICENSE)文件。

## 开发说明

### 代码风格

- 遵循Rust官方代码风格指南
- 使用`cargo fmt`进行代码格式化
- 使用`cargo clippy`进行代码检查

### 调试

```bash
# 开启日志调试
RUST_LOG=debug cargo run -- <zip_path> <target_path>
```

## 常见问题

### Q: 为什么运行时提示"必须提供目标路径"？
A: 请确保在命令行中提供了正确的目标路径参数。

### Q: 为什么运行时提示"系统找不到指定的文件"？
A: 请检查更新包路径是否正确，确保文件存在。

### Q: 为什么运行时提示"更新包中未找到指定目录"？
A: 请检查压缩包内路径是否正确，确保该路径在压缩包中存在。

### Q: 为什么界面显示乱码？
A: 请确保系统中安装了中文字体（如微软雅黑），或者使用英文界面。

## 贡献

欢迎提交Issue和Pull Request！

## 联系方式

如有问题或建议，请通过以下方式联系：

- 项目地址：<repository-url>
- Issue：<repository-url>/issues

## 更新日志

### v0.1.0
- 初始版本
- 支持ZIP文件解压
- 支持GUI进度条
- 支持中文和英文界面
- 支持命令行参数配置
- 支持自定义窗口图标
- 支持实时进度更新
