use anyhow::Result;
use gpui_kit::{component::Root, *};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub struct WindowSpec {
    pub preferred: Size<Pixels>,
    pub minimum: Size<Pixels>,
}

pub struct AppDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub data_directory: PathBuf,
    pub window: WindowSpec,
    pub single_instance: bool,
    #[cfg(feature = "update")]
    pub update: Option<cardo_update::UpdateConfig>,
}

/// Hooks run in order: initialize, maintenance, prepare, instance, UI, create.
/// Product policy and business requests stay in the implementation.
pub trait AppServices: 'static {
    type View: Render;
    fn initialize(&mut self, descriptor: &AppDescriptor, args: &mut Vec<String>) -> Result<()>;
    fn maintenance(&mut self, _args: &[String]) -> Option<Result<()>> {
        None
    }
    fn prepare(&mut self, _args: &mut Vec<String>) -> Result<()> {
        Ok(())
    }
    fn initialize_ui(&mut self, cx: &mut App) -> Result<()>;
    fn create(&mut self, window: &mut Window, cx: &mut App) -> Entity<Self::View>;
    fn launch(
        view: &mut Self::View,
        args: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self::View>,
    );
    fn shutdown(&mut self) {}
}

struct Lifecycle {
    _instance: Option<cardo_platform::instance::Instance>,
    _receiver: Option<Task<()>>,
    _closed: Subscription,
}
impl Global for Lifecycle {}

/// Runs maintenance modes before opening windows or joining the regular instance.
#[cfg(windows)]
pub fn run<S: AppServices>(
    descriptor: AppDescriptor,
    assets: impl AssetSource,
    mut services: S,
    mut args: Vec<String>,
    report: fn(anyhow::Error),
) -> Result<()> {
    #[cfg(feature = "update")]
    if let Some(update) = &descriptor.update {
        anyhow::ensure!(
            update.application_id == descriptor.id && update.version == descriptor.version,
            "Update identity and version must match the application descriptor"
        );
    }
    #[cfg(feature = "diagnostics")]
    let _logs = cardo_runtime::diagnostics::init(
        &descriptor.data_directory.join("logs"),
        &descriptor.name,
    )?;
    services.initialize(&descriptor, &mut args)?;
    if let Some(result) = services.maintenance(&args) {
        return result;
    }
    services.prepare(&mut args)?;
    let instance = if descriptor.single_instance {
        match cardo_platform::instance::instance(&descriptor.id, &args)? {
            Some(instance) => Some(instance),
            None => return Ok(()),
        }
    } else {
        None
    };
    let services = Rc::new(RefCell::new(services));
    let running = services.clone();
    gpui_kit::application().with_assets(assets).run(move |cx| {
        gpui_kit::init(cx);
        if let Err(error) = running.borrow_mut().initialize_ui(cx) {
            report(error);
            cx.quit();
            return;
        }
        let closed = cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        });
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(descriptor.window.preferred, cx)),
            window_min_size: Some(descriptor.window.minimum),
            inactive_frame_interval: None,
            titlebar: Some(TitlebarOptions {
                title: Some(descriptor.name.into()),
                appears_transparent: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        cx.spawn(async move |cx| {
            let mut root = None;
            let opened = cx.open_window(options, |window, cx| {
                let view = running.borrow_mut().create(window, cx);
                view.update(cx, |view, cx| S::launch(view, args, window, cx));
                root = Some(view.downgrade());
                cx.new(|cx| Root::new(view, window, cx))
            });
            match opened {
                Ok(handle) => {
                    let receiver = instance.as_ref().map(|i| i.receiver.clone());
                    let view = root.expect("created root");
                    cx.update(|cx| {
                        let task = receiver.map(|receiver| {
                            cx.spawn(async move |cx| {
                                while let Ok(event) = receiver.recv().await {
                                    if handle
                                        .update(cx, |_, window, cx| match event {
                                            cardo_platform::instance::LaunchEvent::Launch(args) => {
                                                window.activate_window();
                                                let _ = view.update(cx, |view, cx| {
                                                    S::launch(view, args, window, cx)
                                                });
                                            }
                                            cardo_platform::instance::LaunchEvent::Error(error) => {
                                                report(anyhow::anyhow!(error))
                                            }
                                        })
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            })
                        });
                        cx.set_global(Lifecycle {
                            _instance: instance,
                            _receiver: task,
                            _closed: closed,
                        });
                    });
                }
                Err(error) => {
                    report(error);
                    cx.update(|cx| cx.quit());
                }
            }
        })
        .detach();
    });
    services.borrow_mut().shutdown();
    Ok(())
}
