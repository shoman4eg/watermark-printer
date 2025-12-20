use ab_glyph::FontRef;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::{draw_text_mut, text_size};
use imageproc::geometric_transformations::{Interpolation, Projection, warp};

#[derive(Clone)]
pub struct WatermarkConfig {
    pub size: f32,
    pub color: [u8; 4],
    pub rotation_degrees: f32,
    pub position_x: f32,
    pub position_y: f32,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            size: 256.0,
            color: [128, 128, 128, 192],
            rotation_degrees: -25.8,
            position_x: 0.0,
            position_y: 100.0,
        }
    }
}

pub struct WatermarkFactory {
    font: FontRef<'static>,
    config: WatermarkConfig,
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Default for WatermarkFactory {
    fn default() -> Self {
        Self::new(WatermarkConfig::default())
    }
}

impl WatermarkFactory {
    pub fn new(config: WatermarkConfig) -> Self {
        Self {
            font: FontRef::try_from_slice(include_bytes!("../resources/Inter-Black.ttf") as &[u8])
                .unwrap(),
            config,
        }
    }

    pub fn draw(&self, text: String, width: u32, height: u32, mirror: bool) -> Image {
        let mut image = ImageBuffer::new(width, height);

        let size = text_size(self.config.size, &self.font, &text);

        draw_text_mut(
            &mut image,
            Rgba(self.config.color),
            (width - size.0) as i32 / 2,
            (height - size.1) as i32 / 2,
            self.config.size,
            &self.font,
            &text,
        );

        let mut projection = Projection::translate(0.5 * width as f32, 0.5 * height as f32);

        if mirror {
            projection = projection * Projection::scale(1.0, -1.0);
        }

        // Конвертируем градусы в радианы
        let rotation_radians = self.config.rotation_degrees.to_radians();
        
        projection = projection
            * Projection::rotate(rotation_radians)
            * Projection::translate(-0.5 * width as f32, -0.5 * height as f32);

        Image {
            width,
            height,
            data: warp(
                &image,
                &projection,
                Interpolation::Nearest,
                Rgba([0u8, 0u8, 0u8, 0u8]),
            )
            .as_raw()
            .to_vec(),
        }
    }
}
