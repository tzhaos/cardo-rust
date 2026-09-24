# 应用与服务接口

[首页](../README.zh.md) · [English](application.md)

## 组合与依赖

`cardo-runtime` 不依赖 GPUI。platform 依赖 runtime；ui 依赖 runtime，启用 `system-icons` 时依赖 platform；update 依赖 runtime/platform；app 负责组合。框架 crate 不依赖应用 crate。

| Feature | 启用位置 | 能力 |
| --- | --- | --- |
| `config` | runtime / app | TOML 快照与 `ConfigService<T>` |
| `database` | runtime / app | 内置 SQLite |
| `localization` | runtime / ui / app | Project Fluent 资源与 Cardo 文案 |
| `diagnostics` | runtime / app | 文件诊断 |
| `system-icons` | ui | 系统图像适配 |
| `update` | app | 可选更新服务依赖 |

默认 feature 为空。`cardo-update` 启用自身文案所需的 runtime 本地化。GPUI Kit 固定为 0.6.1。框架不包含微软 Fluent UI 图标集；Project Fluent 是本地化引擎。

## 启动顺序

`AppDescriptor` 提供稳定标识、名称、应用自身版本、数据目录、窗口规格、单实例选项及更新配置。更新身份和版本必须与应用一致。

`AppServices` 按以下顺序运行：

1. 可选诊断在指定数据目录初始化。
2. `initialize` 显式加载配置和语言资源。
3. `maintenance` 路由辅助模式，直接返回，不创建主窗口、不参与普通单实例分发。
4. `prepare` 完成普通启动准备。
5. 单实例服务取得所有权，或转交参数后退出。
6. `initialize_ui` 应用字体与主题，再由 `create` 构造主页面。
7. `launch` 在所属窗口接收首次与后续请求。
8. 窗口关闭后退出；UI 事件循环结束后调用 `shutdown`。

参数解析、注册和业务路由留在这些钩子中。可选更新描述不会自动安装更新器或添加界面；应用仍需通过适配器构造 `UpdateService`。[正式模板](../templates/desktop/README.zh.md) 展示不启用更新的完整启动过程。

## 窗口与交互

共享控件渲染前调用 `theme::set_presentation` 并应用 `ThemeStyle`。`app_frame`、`panel`、`settings_page`、`chrome` 与 `controls` 组合布局，不限定导航形式；品牌、图标和业务尺寸由应用提供。

`DialogHost::open` 接收宿主、窗口与内容范围、尺寸、焦点、目标窗口内容工厂、渲染器和关闭策略。输入实体在工厂中创建，返回的内容在开窗流程结束前安装到宿主。窗口构造期间有渲染保护，按实测内容居中并限制在显示器可见范围。创建失败返回错误，应用取消关联工作并报告失败。

关闭原因区分提交、取消、Escape、窗口关闭及宿主释放。框架持有句柄、观察与释放订阅、焦点恢复和代次保护。替换弹窗先处理旧业务状态并使旧代次失效，再安装新状态，避免延迟回调影响新弹窗。应用决定任务期间能否关闭并负责业务清理。

`ToastHost` 管理替换、代次、三秒计时和拖动偏移。应用转交移动与释放事件并提供内容范围。新消息重置位置和计时。路径、操作和详细错误使用弹窗。原生命令菜单保留窗口与代次归属，设置选择器使用主题弹出层。

## 任务、设置与命令

`CancellationToken` 发出合作式取消请求；`TaskState<P>` 区分运行、取消中和结束，携带实际进度。UI `task::spawn` 向弱引用宿主交付类型化结果。句柄释放时请求取消并分离交付任务，让后台工作真正结束并释放资源；临时资源应由工作线程持有。`refresh_while` 只在真实进度活跃时刷新。

`SettingsDraft<T>` 捕获提交快照、记录在途保存及字段错误，失败保留草稿，成功提交对应快照。应用负责保存期间阻止切页或关闭，以及验证和副作用。`ConfigService<T>` 串行访问 TOML 快照，不提供跨文件、SQLite、注册表事务。

`CommandDescriptor<A>` 与覆盖表示默认、禁用和显式绑定。菜单与键盘使用相同绑定，应用按命令身份决定可用性和执行。共享冲突检测、录制和输入保护接受应用限制。禁止按翻译标签分支。

`CatalogSet` 管理资源和当前语言。先显式加载应用资源，再加载配置、选择语言，不在本地化初始化中隐式加载设置。Cardo 使用独立的 `cardo-*` 文案空间，提供英文、简体、繁体资源。

## 存储边界

| 媒介 | 用途 | 保证与限制 |
| --- | --- | --- |
| TOML | 可编辑偏好 | 仅缺失使用默认值；读取和解析错误带路径。原子写入保留注释、上一版备份，拒绝覆盖外部修改。 |
| SQLite | 历史和运行状态 | 应用标识及结构版本严格匹配，使用短事务；结果写入与待处理记录删除使用同一事务。不迁移、不导入。 |
| JSON | 协议、发布元数据、SQLite 结构化值 | 应用负责序列化，不作为额外偏好存储。 |

`database::Database` 提供完整性检查、事务和备份，使用 WAL、FULL 同步及五秒忙等待。存储不进入 render。应用负责结构、路径、保留策略和错误展示。注册表辅助不决定产品路径或关联策略。

## 平台与更新

`instance::Instance` 持有接收器和 Windows 管道线程直到释放，无需 UI 轮询。系统图标返回独立图像数据。文件选择器返回 `Result<Option<T>>`，区分选择、取消、失败。进程辅助按提供的程序身份工作。

`UpdateConfig` 指定版本与仓库、发行文件名、包内根目录、允许相对路径、程序及辅助文件、暂存位置和下载上限。`UpdateJournal` 读写待处理记录与结果；`finish` 在同一事务写结果并删除待处理记录。`InstallAdapter` 负责安装身份、安装器调用、额外备份、注册恢复和版本验证。

Cardo 负责稳定版检查、下载限制、SHA-256、清单及路径检查、进程身份验证、文件备份和替换失败恢复。安装器与策略由应用提供。配置真实发布信息后再启用更新；在隔离安装目录验收应用和恢复，构建通过不代表运行验收。
