use gpui_kit::*;
use std::sync::Arc;
const SUPERSAMPLE: u32 = 2;
pub fn artwork(asset: &'static str, logical: f32, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let device = device_pixels(logical, window.scale_factor());
    let cached = window.use_keyed_state(
        SharedString::from(format!("{}:{device}", asset)),
        cx,
        |_, cx| rasterize_artwork(asset, device, cx),
    );
    match cached.read(cx).clone() {
        Some(image) => img(image),
        None => img(asset),
    }
    .size(px(logical))
    .flex_shrink_0()
}

fn device_pixels(logical: f32, scale: f32) -> u32 {
    (logical * scale).ceil().max(1.0) as u32
}

fn rasterize_artwork(asset: &str, device: u32, cx: &mut App) -> Option<Arc<RenderImage>> {
    let bytes = cx.asset_source().load(asset).ok().flatten()?;
    let renderer = cx.svg_renderer();
    let svg = renderer.parse_svg(&bytes).ok()?;
    let factor = SUPERSAMPLE.min(2048 / device).max(1);
    let raster = device * factor;
    let image = renderer
        .render_parsed(
            &svg,
            SvgSize::ExactSize(size(
                DevicePixels(raster as i32),
                DevicePixels(raster as i32),
            )),
        )
        .ok()?;
    let source = image.as_bytes(0)?;
    let pixels = if factor == 1 {
        source.to_vec()
    } else {
        downsample_premultiplied(source, raster, factor)
    };
    let buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(device, device, pixels)?;
    Some(Arc::new(RenderImage::new([image::Frame::new(buffer)])))
}

fn downsample_premultiplied(source: &[u8], width: u32, factor: u32) -> Vec<u8> {
    let destination = width / factor;
    let samples = factor * factor;
    let mut output = vec![0u8; (destination * destination * 4) as usize];
    for y in 0..destination {
        for x in 0..destination {
            let mut channels = [0u32; 4];
            for offset_y in 0..factor {
                for offset_x in 0..factor {
                    let index =
                        (((y * factor + offset_y) * width + x * factor + offset_x) * 4) as usize;
                    let alpha = source[index + 3] as u32;
                    channels[0] += source[index] as u32 * alpha;
                    channels[1] += source[index + 1] as u32 * alpha;
                    channels[2] += source[index + 2] as u32 * alpha;
                    channels[3] += alpha;
                }
            }
            let index = ((y * destination + x) * 4) as usize;
            if channels[3] == 0 {
                continue;
            }
            output[index] = (channels[0] / channels[3]) as u8;
            output[index + 1] = (channels[1] / channels[3]) as u8;
            output[index + 2] = (channels[2] / channels[3]) as u8;
            output[index + 3] = (channels[3] / samples) as u8;
        }
    }
    output
}
