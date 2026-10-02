use cxx_qt_lib::{QImage, QImageFormat};
use libdj::types::analysis::{PreviewWaveformColumn, WaveformColumn};

const WAVEFORM_COLUMNS_PER_SECOND: f32 = 150.0;

pub fn waveform_placeholder_texture() -> QImage {
    // Safety: Image statically guaranteed to be safe
    unsafe { QImage::from_raw_bytes(vec![0, 0, 0, 0], 1, 1, QImageFormat::Format_ARGB32) }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn waveform_to_shader_texture(waveform: &[WaveformColumn]) -> (QImage, f32, f32) {
    if waveform.is_empty() {
        return (waveform_placeholder_texture(), 1.0, 1.0);
    }

    #[allow(clippy::cast_precision_loss)]
    let waveform_length_seconds = waveform.len() as f32 / WAVEFORM_COLUMNS_PER_SECOND;

    let width = waveform.len().clamp(0, 16384);
    let mut height = (waveform.len() / 16384) + 1;

    let mut image_data: Vec<u8> = waveform
        .iter()
        .flat_map(|column| match column {
            WaveformColumn::Grayscale { height, saturation } => [
                95 - saturation / 4,
                95 - saturation / 2,
                95 - saturation,
                *height,
            ],
            WaveformColumn::RGB { height, color_rgb } => {
                [color_rgb.2, color_rgb.1, color_rgb.0, *height]
            }
            WaveformColumn::ThreeBand { height, bands } => [bands.2, bands.1, bands.0, *height],
        })
        .collect();

    #[allow(clippy::cast_precision_loss)]
    let stride = (image_data.len() / 4) as f32 / width as f32;

    if image_data.len() < width * height * 4 {
        image_data.resize(width * height * 4, 0);
    } else if image_data.len() > width * height * 4 {
        while image_data.len() > width * height * 4 {
            height += 1;
        }

        image_data.resize(width * height * 4, 0);
    }

    (
        // Safety: Image generation algorithm guarantees dimension match
        unsafe {
            QImage::from_raw_bytes(
                image_data,
                width as i32,
                height as i32,
                QImageFormat::Format_ARGB32_Premultiplied,
            )
        },
        stride,
        waveform_length_seconds,
    )
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn preview_waveform_to_shader_texture(waveform: &[PreviewWaveformColumn]) -> QImage {
    if waveform.is_empty() {
        return waveform_placeholder_texture();
    }

    let image_data: Vec<u8> = waveform
        .iter()
        .flat_map(|column| match column {
            PreviewWaveformColumn::Grayscale { height, saturation } => [
                95 - saturation / 4,
                95 - saturation / 2,
                95 - saturation,
                *height,
            ],
            PreviewWaveformColumn::RGB { height, color_rgb } => {
                [color_rgb.2, color_rgb.1, color_rgb.1, *height]
            }
            PreviewWaveformColumn::ThreeBand { height, bands } => {
                [bands.2, bands.1, bands.0, *height]
            }
        })
        .collect();

    // Safety: Image generation algorithm guarantees dimension match
    unsafe {
        QImage::from_raw_bytes(
            image_data,
            waveform.len() as i32,
            1,
            QImageFormat::Format_ARGB32_Premultiplied,
        )
    }
}
