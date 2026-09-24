use cardo_ui::theme::{Palette, ThemeStyle};
use gpui_kit::{component::ThemeMode, *};
fn palette_for(id: ThemeMode) -> Palette {
    match id {
        ThemeMode::Light => Palette {
            surface: 0xffffff,
            panel: 0xf0f1f3,
            title: 0xf5f6f8,
            text: 0x202123,
            muted: 0x777777,
            border: 0xe9e9e9,
            hover: 0xf3f3f3,
            selected: 0xe4edf9,
            selected_hover: 0xd7e5f7,
            accent: 0x7955ca,
            accent_hover: 0x6842b5,
            on_accent: 0xffffff,
            success: 0x267052,
            danger: 0xb42318,
        },
        ThemeMode::Dark => Palette {
            surface: 0x1f1f1f,
            panel: 0x141414,
            title: 0x141414,
            text: 0xe8eaeb,
            muted: 0xa6abad,
            border: 0x393c3e,
            hover: 0x2a2a2a,
            selected: 0x2c2e30,
            selected_hover: 0x393c3e,
            accent: 0xb29af0,
            accent_hover: 0xc3aff6,
            on_accent: 0x141414,
            success: 0x7dc9a1,
            danger: 0xf38d91,
        },
    }
}
pub fn apply(dark: bool, cx: &mut App) -> anyhow::Result<()> {
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    let palette = palette_for(mode);
    let fonts = cardo_ui::fonts::FontStack::resolve("Segoe UI", &cx.text_system().all_font_names())
        .map_err(|_| anyhow::anyhow!("Segoe UI is not installed"))?;
    cardo_ui::theme::set_presentation(palette, fonts.font(), px(13.), cx);
    ThemeStyle {
        mode,
        palette,
        font_family: fonts.family(),
        font_size: px(16.),
        radius: px(16.),
        radius_lg: px(8.),
    }
    .apply(None, cx);
    Ok(())
}
