# 软件更新脚本

一个用Rust编写的命令行更新脚本，用于解压更新包并替换目标文件，同时提供可视化进度窗口。

## 功能特性

### 核心功能
1. **解压更新包**：支持解压指定的zip完整更新包
2. **智能文件替换**：将当前运行路径下的目标文件进行替换
3. **跳过运行文件**：除了当前运行的可执行文件外，其他文件都会被替换
4. **可视化进度**：提供直观的GUI窗口显示更新进度
5. **完成弹窗**：更新完成后自动弹出提示框

### 可视化功能
- 动态进度条显示更新进度
- 实时显示当前处理的文件名
- 清晰的标题和状态文字
- 完成后自动弹窗提示

## 系统要求

- **操作系统**：Windows、macOS、Linux
- **Rust版本**：1.70.0+（推荐使用最新稳定版）
- **依赖**：需要网络连接下载GUI库（首次构建时）

## 安装与构建

### 1. 克隆或下载项目

```bash
git clone <repository-url>
cd software_updater
```

### 2. 构建项目

```bash
# 构建开发版本
cargo build

# 构建发布版本（推荐）
cargo build --release
```

构建完成后，可执行文件将位于：
- 开发版本：`target/debug/software_updater.exe`
- 发布版本：`target/release/software_updater.exe`

## 使用方法

### 正常模式

```bash
# 使用发布版本
software_updater.exe LHandProxxxxxx.zip

# 使用开发版本
cargo run -- LHandProxxxxxx.zip
```

### 测试模式

测试模式允许您在没有实际zip文件的情况下测试GUI界面和进度显示功能：

1. **构建测试版本**：
   ```bash
   rustc --extern fltk=d:\Project\RustProject\software_updater\target\debug\deps\libfltk-*.rlib --extern fltk_theme=d:\Project\RustProject\software_updater\target\debug\deps\libfltk_theme-*.rlib --extern env_logger=d:\Project\RustProject\software_updater\target\debug\deps\libenv_logger-*.rlib --extern log=d:\Project\RustProject\software_updater\target\debug\deps\liblog-*.rlib --extern tempfile=d:\Project\RustProject\software_updater\target\debug\deps\libtempfile-*.rlib --extern walkdir=d:\Project\RustProject\software_updater\target\debug\deps\libwalkdir-*.rlib --extern zip=d:\Project\RustProject\software_updater\target\debug\deps\libzip-*.rlib src\main_test.rs -o target\debug\software_updater_test.exe
   ```

2. **运行测试版本**：
   ```bash
   # 无需提供zip文件
   software_updater_test.exe
   
   # 也可以提供任意文件名
   software_updater_test.exe test.zip
   ```

## 工作原理

1. **命令行参数解析**：读取指定的zip文件路径
2. **GUI初始化**：创建可视化进度窗口
3. **多线程处理**：
   - 主线程：处理GUI渲染和用户交互
   - 子线程：执行文件解压和替换操作
4. **进度通信**：使用通道（channel）实现线程间通信
5. **文件替换**：
   - 解压更新包到临时目录
   - 计算总文件数
   - 逐个替换文件，实时更新进度
6. **完成通知**：更新完成后显示弹窗提示

## 目录结构

### 压缩包结构
```
LHandPro-xxxxxxxxx.zip
└── LHandPro
    ├── tools
    │   ├── software_update.exe
    │   └── 其他文件
    └── 其他文件和文件夹
```

### 软件运行目录
```
LHandPro
├── tools
│   ├── software_update.exe
│   └── 其他文件
└── 其他文件和文件夹
```

## 注意事项

1. **权限要求**：确保脚本有足够的权限读写文件
2. **备份建议**：在执行更新前，建议备份重要文件
3. **网络连接**：首次构建时需要网络连接下载依赖库
4. **窗口操作**：更新过程中请勿关闭进度窗口
5. **错误处理**：如果更新失败，会显示具体错误信息

## 测试与调试

### 查看日志

脚本使用`env_logger`库记录日志，您可以通过设置`RUST_LOG`环境变量来控制日志级别：

```bash
# 显示所有日志
set RUST_LOG=debug
software_updater.exe LHandProxxxxxx.zip

# 只显示错误日志
set RUST_LOG=error
software_updater.exe LHandProxxxxxx.zip
```

### 开发调试

```bash
# 运行开发版本并调试
cargo run -- LHandProxxxxxx.zip

# 运行测试版本并调试
cargo run --bin software_updater_test
```

## 许可证

本项目采用Apache License 2.0许可证。详见[LICENSE](LICENSE)文件。

```
Copyright 2026 <Your Name>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## 贡献

欢迎提交Issue和Pull Request来帮助改进这个项目。

## 联系方式

如有问题或建议，请通过以下方式联系：
- Email: <your-email@example.com>
- GitHub: <your-github-username>

## 更新日志

### v0.1.0 (2026-01-21)
- 初始版本
- 实现基本的更新功能
- 添加可视化进度窗口
- 支持测试模式
- 添加详细的README文档
