pub mod browser;
pub mod ipc;
pub mod menu;
pub mod player;
pub mod player_waveform;
pub mod types;

use godot::{
    classes::{Image, ImageTexture, image::Format},
    prelude::*,
};
use libdj::types::analysis::{PreviewWaveformColumn, WaveformColumn};

const WAVEFORM_COLUMNS_PER_SECOND: f32 = 150.0;

struct MyExtension;

/// safety: godot forces this to be unsafe, but this should be safe in all
/// reasonable contexts
#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn waveform_placeholder_texture() -> Option<Gd<Image>> {
    Image::create_from_data(1, 1, false, Format::RGBA8, &PackedArray::from([0, 0, 0, 0]))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn waveform_to_shader_texture(
    waveform: &[WaveformColumn],
) -> Option<(Gd<ImageTexture>, f32, f32)> {
    if waveform.is_empty() {
        let image =
            Image::create_from_data(1, 1, false, Format::RGBA8, &PackedArray::from([0, 0, 0, 0]))?;

        return ImageTexture::create_from_image(&image).map(|image| (image, 1.0, 1.0));
    }

    #[allow(clippy::cast_precision_loss)]
    let waveform_length_seconds = waveform.len() as f32 / WAVEFORM_COLUMNS_PER_SECOND;

    let width = waveform.len().clamp(0, 16384);
    let mut height = (waveform.len() / 16384) + 1;

    let mut image_data: Vec<u8> = waveform
        .iter()
        .flat_map(|column| match column {
            WaveformColumn::Grayscale { height, saturation } => [
                95 - saturation,
                95 - saturation / 2,
                95 - saturation / 4,
                *height,
            ],
            WaveformColumn::RGB { height, color_rgb } => {
                [color_rgb.0, color_rgb.1, color_rgb.2, *height]
            }
            WaveformColumn::ThreeBand { height, bands } => [bands.0, bands.1, bands.2, *height],
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

    let image = Image::create_from_data(
        width as i32,
        height as i32,
        false,
        Format::RGBA8,
        &PackedArray::from(image_data),
    )?;

    ImageTexture::create_from_image(&image).map(|image| (image, stride, waveform_length_seconds))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn preview_waveform_to_shader_texture(
    waveform: &[PreviewWaveformColumn],
) -> Option<Gd<ImageTexture>> {
    if waveform.is_empty() {
        let image =
            Image::create_from_data(1, 1, false, Format::RGBA8, &PackedArray::from([0, 0, 0, 0]))?;

        return ImageTexture::create_from_image(&image);
    }

    let image_data: Vec<u8> = waveform
        .iter()
        .flat_map(|column| match column {
            PreviewWaveformColumn::Grayscale { height, saturation } => [
                95 - saturation,
                95 - saturation / 2,
                95 - saturation / 4,
                *height,
            ],
            PreviewWaveformColumn::RGB { height, color_rgb } => {
                [color_rgb.0, color_rgb.1, color_rgb.2, *height]
            }
            PreviewWaveformColumn::ThreeBand { height, bands } => {
                [bands.0, bands.1, bands.2, *height]
            }
        })
        .collect();

    let image = Image::create_from_data(
        waveform.len() as i32,
        1,
        false,
        Format::RGBA8,
        &PackedArray::from(image_data),
    )?;

    ImageTexture::create_from_image(&image)
}
