#![cfg_attr(windows, windows_subsystem = "windows")]
mod theme;
use anyhow::{Context as _, Result};
use cardo_app::{AppDescriptor, AppServices, WindowSpec};
use cardo_runtime::{
    config::{ConfigService, Snapshot},
    localization::{Catalog, CatalogSet},
    settings::SettingsDraft,
    task::{CancellationToken, TaskState},
};
use cardo_ui::{controls::command, panel::*, settings_page::*, text::body_text, toast::ToastHost};
use gpui_kit::{
    component::{Disableable, Icon, h_flex, v_flex},
    *,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::PathBuf,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

static TEXT: OnceLock<CatalogSet> = OnceLock::new();
fn tr(key: &str) -> &'static str {
    TEXT.get()
        .expect("initialized catalog")
        .text(key)
        .expect("bundled message")
}
fn tf(key: &str, args: &[(&str, cardo_runtime::localization::MessageValue<'_>)]) -> String {
    TEXT.get()
        .unwrap()
        .format(key, args)
        .expect("bundled arguments")
}
#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    dark: bool,
    locale: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: false,
            locale: "en-US".into(),
        }
    }
}
struct Services {
    config: Option<Arc<ConfigService<Settings>>>,
}
impl AppServices for Services {
    type View = Desk;
    fn initialize(&mut self, descriptor: &AppDescriptor, _: &mut Vec<String>) -> Result<()> {
        let config = Arc::new(ConfigService::new(Snapshot::load(
            descriptor.data_directory.join("settings.toml"),
            || Ok(Settings::default()),
        )?));
        let settings = config.current()?;
        let catalogs = CatalogSet::new(
            vec![
                Catalog::new("en-US", include_str!("../locales/en-US.ftl"))?,
                Catalog::new("zh-CN", include_str!("../locales/zh-CN.ftl"))?,
                Catalog::new("zh-TW", include_str!("../locales/zh-TW.ftl"))?,
            ],
            &settings.locale,
        )?;
        cardo_runtime::messages::select(&settings.locale);
        let _ = TEXT.set(catalogs);
        self.config = Some(config);
        Ok(())
    }
    fn initialize_ui(&mut self, cx: &mut App) -> Result<()> {
        theme::apply(self.config.as_ref().unwrap().current()?.dark, cx)
    }
    fn create(&mut self, window: &mut Window, cx: &mut App) -> Entity<Desk> {
        let config = self.config.take().unwrap();
        cx.new(|cx| Desk::new(config, window, cx))
    }
    fn launch(view: &mut Desk, args: Vec<String>, _: &mut Window, cx: &mut Context<Desk>) {
        if let Some(path) = args.first() {
            view.inspect(Some(PathBuf::from(path)), cx);
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Settings,
    About,
}
struct Desk {
    config: Arc<ConfigService<Settings>>,
    draft: SettingsDraft<Settings>,
    page: Page,
    task: Option<cardo_ui::task::TaskHandle>,
    state: TaskState<Arc<AtomicU64>>,
    refresh: Option<Task<()>>,
    toast: Entity<ToastHost>,
    result: String,
    error: Option<String>,
    dialog: cardo_ui::dialog::DialogHost,
    main: AnyWindowHandle,
    focus: FocusHandle,
}
impl Desk {
    fn new(
        config: Arc<ConfigService<Settings>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let draft = SettingsDraft::new(config.current().expect("loaded settings"));
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        let weak = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            weak.read_with(cx, |this, _| {
                !this.state.is_busy() && !this.draft.is_saving()
            })
            .unwrap_or(true)
        });
        Self {
            config,
            draft,
            page: Page::Home,
            task: None,
            state: Default::default(),
            refresh: None,
            toast: cx.new(|_| {
                ToastHost::localized(
                    Icon::new(gpui_kit::assets::IconName::Info),
                    Icon::new(gpui_kit::assets::IconName::X),
                )
            }),
            result: String::new(),
            error: None,
            dialog: Default::default(),
            main: window.window_handle(),
            focus,
        }
    }
    fn inspect(&mut self, path: Option<PathBuf>, cx: &mut Context<Self>) {
        if self.state.is_busy() || self.draft.is_saving() {
            return;
        }
        let cancel = self.state.begin(tr("working"));
        let progress = Arc::new(AtomicU64::new(0));
        self.state.set_progress(progress.clone());
        self.result.clear();
        self.error = None;
        self.task = Some(cardo_ui::task::spawn(
            cx,
            cancel,
            move |cancel| -> Result<Option<(PathBuf, u64, String)>> {
                let path = match path {
                    Some(path) => path,
                    None => match cardo_platform::picker::FileDialog::new()
                        .set_title(tr("choose"))
                        .pick_file()?
                    {
                        Some(path) => path,
                        None => return Ok(None),
                    },
                };
                let mut file = std::fs::File::open(&path)
                    .with_context(|| format!("Cannot read {}", path.display()))?;
                let mut hash = Sha256::new();
                let mut bytes = 0;
                let mut buffer = [0; 65536];
                loop {
                    if cancel.is_cancelled() {
                        return Ok(None);
                    }
                    let count = file
                        .read(&mut buffer)
                        .with_context(|| format!("Cannot read {}", path.display()))?;
                    if count == 0 {
                        break;
                    }
                    hash.update(&buffer[..count]);
                    bytes += count as u64;
                    progress.store(bytes, Ordering::Relaxed);
                }
                Ok(Some((path, bytes, format!("{:x}", hash.finalize()))))
            },
            |this, result, cx| {
                this.state.finish();
                this.task = None;
                this.refresh = None;
                match result {
                    Ok(Some((path, bytes, hash))) => {
                        this.result = tf(
                            "result",
                            &[
                                ("path", path.display().to_string().into()),
                                ("bytes", bytes.to_string().into()),
                                ("hash", hash.into()),
                            ],
                        );
                        this.toast
                            .update(cx, |toast, cx| toast.show(tr("finished").into(), cx));
                    }
                    Ok(None) => this
                        .toast
                        .update(cx, |toast, cx| toast.show(tr("cancelled").into(), cx)),
                    Err(error) => this.show_error(format!("{error:#}"), cx),
                }
                cx.notify();
            },
        ));
        self.refresh = Some(cardo_ui::task::refresh_while(cx, |this| {
            this.state.is_busy()
        }));
        cx.notify();
    }
    fn show_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.error = Some(error);
        let owner = cx.entity();
        let main = self.main;
        let focus = self.focus.clone();
        cx.defer(move |cx| {
            owner.update(cx, |this, cx| {
                let result = this.dialog.open(
                    owner.clone(),
                    main,
                    Bounds::default(),
                    tr("error").into(),
                    cardo_ui::dialog::DialogSpec {
                        preferred: size(px(560.), px(320.)),
                        minimum: size(px(360.), px(200.)),
                    },
                    focus,
                    |_, _| (),
                    |this, _, cx| {
                        Some(
                            panel_layout(cx)
                                .id("error-dialog")
                                .track_focus(&this.focus)
                                .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                    if event.keystroke.key == "escape" {
                                        this.dialog
                                            .close(cardo_ui::dialog::CloseReason::Escape, cx);
                                        this.error = None;
                                        cx.stop_propagation();
                                    }
                                }))
                                .child(
                                    panel_header(tr("error"), true, cx).child(
                                        cardo_ui::controls::icon_button(
                                            "dismiss-error",
                                            Icon::new(gpui_kit::assets::IconName::X),
                                            cardo_runtime::messages::tr("close"),
                                            false,
                                            cx,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.dialog.close(
                                                    cardo_ui::dialog::CloseReason::Cancel,
                                                    cx,
                                                );
                                                this.error = None;
                                            }),
                                        ),
                                    ),
                                )
                                .child(
                                    panel_body("error-body")
                                        .child(body_text(this.error.clone().unwrap_or_default())),
                                )
                                .child(panel_actions(cx))
                                .into_any_element(),
                        )
                    },
                    |_, _| true,
                    |this, _, _| {
                        this.dialog.detach();
                        this.error = None;
                    },
                    || cardo_runtime::messages::tr("menu-more").into(),
                    cx,
                );
                if let Err(error) = result {
                    report(error);
                    this.error = None;
                }
            });
        });
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.draft.begin_save() else {
            return;
        };
        let config = self.config.clone();
        self.task = Some(cardo_ui::task::spawn(
            cx,
            CancellationToken::default(),
            move |_| {
                config.save(snapshot.clone(), |_| Ok(()))?;
                Ok::<_, anyhow::Error>(snapshot)
            },
            |this, result, cx| {
                this.task = None;
                match result {
                    Ok(saved) => {
                        this.draft.complete();
                        let _ = TEXT.get().unwrap().select(&saved.locale);
                        cardo_runtime::messages::select(&saved.locale);
                        if let Err(error) = theme::apply(saved.dark, cx) {
                            this.draft.fail("appearance", error.to_string());
                        }
                        this.toast
                            .update(cx, |toast, cx| toast.show(tr("saved").into(), cx));
                    }
                    Err(error) => this.draft.fail("save", format!("{error:#}")),
                }
                cx.notify();
            },
        ));
        cx.notify();
    }
}
impl Render for Desk {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let busy = self.state.is_busy() || self.draft.is_saving();
        let navigation = h_flex()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .gap_2()
            .children(
                [
                    (Page::Home, "home"),
                    (Page::Settings, "settings"),
                    (Page::About, "about"),
                ]
                .into_iter()
                .map(|(page, key)| {
                    command(key, tr(key)).disabled(busy).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.page = page;
                            cx.notify();
                        },
                    ))
                }),
            );
        let chrome = cardo_ui::chrome::titlebar(
            "Cardo Desk",
            navigation,
            busy,
            ["minimize", "maximize", "close"].map(|k| cardo_runtime::messages::tr(k).into()),
            |name, size| {
                Icon::new(match name {
                    "Subtract" => gpui_kit::assets::IconName::Minus,
                    "Dismiss" => gpui_kit::assets::IconName::X,
                    _ => gpui_kit::assets::IconName::Square,
                })
                .size(px(size))
            },
            window,
            cx,
        );
        let body = match self.page {
            Page::Home => v_flex()
                .gap_4()
                .child(body_text(tr("intro")))
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            command("choose", tr("choose"))
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| this.inspect(None, cx))),
                        )
                        .child(
                            command("stop", tr("stop"))
                                .disabled(!self.state.is_busy())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.state.cancel(tr("cancelling"));
                                    cx.notify();
                                })),
                        ),
                )
                .child(body_text(if self.state.is_busy() {
                    tf("progress", &[("bytes", this_bytes(&self.state).into())])
                } else {
                    self.result.clone()
                })),
            Page::Settings => settings_frame()
                .child(settings_section(
                    tr("settings"),
                    cardo_ui::settings::group(
                        [
                            cardo_ui::settings::row(
                                tr("theme"),
                                None,
                                command(
                                    "theme",
                                    tr(if self.draft.dark { "dark" } else { "light" }),
                                )
                                .disabled(busy)
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.draft.dark = !this.draft.dark;
                                        cx.notify();
                                    },
                                )),
                                cx,
                            )
                            .into_any_element(),
                            cardo_ui::settings::row(
                                tr("language"),
                                None,
                                h_flex().gap_2().children(
                                    [
                                        ("en-US", "English"),
                                        ("zh-CN", "简体中文"),
                                        ("zh-TW", "繁體中文"),
                                    ]
                                    .into_iter()
                                    .map(|(locale, label)| {
                                        command(locale, label).disabled(busy).on_click(cx.listener(
                                            move |this, _, _, cx| {
                                                this.draft.locale = locale.into();
                                                cx.notify();
                                            },
                                        ))
                                    }),
                                ),
                                cx,
                            )
                            .into_any_element(),
                        ],
                        cx,
                    ),
                    cx,
                ))
                .child(
                    command("save", tr("save"))
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                )
                .children(self.draft.errors.values().cloned().map(body_text)),
            Page::About => v_flex().gap_4().child(body_text(tf(
                "about-text",
                &[("version", env!("CARGO_PKG_VERSION").into())],
            ))),
        };
        cardo_ui::shell::app_frame(cx)
            .id("desk")
            .track_focus(&self.focus)
            .child(chrome)
            .on_mouse_move(cx.listener(|this, event, window, cx| {
                this.toast
                    .update(cx, |toast, cx| toast.pointer_move(event, window, cx))
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.toast.update(cx, |toast, _| toast.end_drag())),
            )
            .child(
                panel_surface(cx)
                    .shadow_none()
                    .m(px(12.))
                    .child(settings_content("page", body))
                    .child(self.toast.clone()),
            )
    }
}
fn this_bytes(state: &TaskState<Arc<AtomicU64>>) -> String {
    state
        .progress()
        .map(|p| p.load(Ordering::Relaxed))
        .unwrap_or(0)
        .to_string()
}
fn report(error: anyhow::Error) {
    rfd::MessageDialog::new()
        .set_title("Cardo Desk")
        .set_description(format!("{error:#}"))
        .set_level(rfd::MessageLevel::Error)
        .show();
}
fn main() {
    let result = (|| {
        let data = dirs::data_local_dir()
            .context("Local data directory is unavailable")?
            .join("cardo-desk");
        let descriptor = AppDescriptor {
            id: "cardo-desk".into(),
            name: "Cardo Desk".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            data_directory: data,
            window: WindowSpec {
                preferred: size(px(900.), px(680.)),
                minimum: size(px(700.), px(460.)),
            },
            single_instance: true,
            #[cfg(feature = "update")]
            update: None,
        };
        cardo_app::run(
            descriptor,
            gpui_kit::assets::AllAssets,
            Services { config: None },
            std::env::args().skip(1).collect(),
            report,
        )
    })();
    if let Err(error) = result {
        report(error);
        std::process::exit(1);
    }
}
