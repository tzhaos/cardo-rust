# Cardo Desk

[English](README.md) · [Cardo](../../README.zh.md)

可直接运行的 Windows 桌面应用起点，拥有独立 Cargo workspace、应用身份和数据目录。包含主页、设置、关于、主题与语言设置、原生文件选择、单实例启动转交、可取消的流式 SHA-256 文件计算、弹窗和短暂通知。

## 构建运行

需要 Windows x64、仓库固定的 Rust 工具链、Visual Studio C++ Build Tools 和 Windows SDK。在 Cardo 仓库执行：

```powershell
./templates/desktop/build.ps1
./target/x86_64-pc-windows-msvc/release/cardo-desk.exe
```

脚本构建模板自己的 workspace，并从已安装的 Visual C++ 工具集复制运行库。无需任何下游应用源码、二进制或资源。也可以向程序传入文件路径；重复启动会把路径交给现有窗口处理。

## 改成自己的应用

1. 将本目录复制到应用仓库，以 submodule 引用 Cardo，修改 `Cargo.toml` 中四个 Cardo 路径依赖，同时调整 `build.ps1` 中的 Cardo 路径。
2. 设置包名和版本、`AppDescriptor` 的身份、名称、数据目录、窗口标题及语言文案。应用版本独立于框架版本。
3. 实现 `AppServices`，用 `cardo-ui` 组织业务页面；共享控件渲染前先调用 `theme::apply`。
4. 业务验证和任务仍由应用实现。示例在后台读取文件，显示实际字节数，工作线程返回后才结束取消状态。

设置保存在 `%LOCALAPPDATA%\cardo-desk\settings.toml`，同目录下保存日志。显式保存会捕获提交时的草稿；失败保留草稿。外部修改导致保存冲突时，重启重新加载。模板不创建数据库。

更新默认关闭。`./templates/desktop/build.ps1 -WithUpdate` 只验证可选依赖组合，不启用发布源。配置真实的 `UpdateConfig`、`UpdateJournal`、`InstallAdapter`、维护模式路由和更新界面后，再将 `AppDescriptor.update` 设置为 `Some(...)`。

模板使用 GPUI Kit 0.6.1 附带的 Lucide 图标，分发时保留 `licenses/lucide.txt` 及适用的依赖许可。本模板不包含自动化测试代码。
