use crate::level::WALL_TYPE_COUNT;
use image::ImageFormat;

const WALL_TEXTURES: [&[u8]; 3] = [
    include_bytes!("../assets/texturas/mamposteria_terracota.png"),
    include_bytes!("../assets/texturas/mamposteria_lunar.png"),
    include_bytes!("../assets/texturas/mamposteria_musgo.png"),
];
const _: () = assert!(WALL_TEXTURES.len() == WALL_TYPE_COUNT as usize);

const LIGHT_LEVEL_COUNT: usize = 16;
const MIN_LIGHT: f32 = 0.3;

#[derive(Debug, PartialEq)]
struct Texture {
    width: usize,
    height: usize,
    light_levels: Vec<Vec<u32>>,
}

#[derive(Debug)]
pub struct WallTextures {
    textures: Vec<Texture>,
}

pub struct TextureColumn<'a> {
    pixels: &'a [u32],
    width: usize,
    height: usize,
    x: usize,
}

impl WallTextures {
    pub fn load_embedded() -> Result<Self, String> {
        let textures = WALL_TEXTURES
            .iter()
            .enumerate()
            .map(|(index, bytes)| decode_png(bytes, index + 1))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { textures })
    }

    pub fn column(&self, wall_type: u8, u: f32, brightness: f32) -> TextureColumn<'_> {
        let index = wall_type.saturating_sub(1) as usize;
        let texture = self
            .textures
            .get(index)
            .unwrap_or_else(|| &self.textures[0]);
        let u = if u.is_finite() {
            u.rem_euclid(1.0)
        } else {
            0.0
        };
        let x = (u * texture.width as f32) as usize;
        let brightness = if brightness.is_finite() {
            brightness.clamp(MIN_LIGHT, 1.0)
        } else {
            1.0
        };
        let light_level = (((brightness - MIN_LIGHT) / (1.0 - MIN_LIGHT)
            * (LIGHT_LEVEL_COUNT - 1) as f32)
            .round() as usize)
            .min(LIGHT_LEVEL_COUNT - 1);

        TextureColumn {
            pixels: &texture.light_levels[light_level],
            width: texture.width,
            height: texture.height,
            x,
        }
    }
}

impl TextureColumn<'_> {
    pub fn sample(&self, v: f32) -> u32 {
        let v = if v.is_finite() {
            v.clamp(0.0, 1.0 - f32::EPSILON)
        } else {
            0.0
        };
        let y = (v * self.height as f32) as usize;
        self.pixels[y * self.width + self.x]
    }
}

fn decode_png(bytes: &[u8], wall_type: usize) -> Result<Texture, String> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|error| format!("No se pudo cargar la textura {wall_type}: {error}"))?
        .to_rgb8();
    let width = image.width() as usize;
    let height = image.height() as usize;
    if width == 0 || height == 0 {
        return Err(format!("La textura {wall_type} no puede estar vacía"));
    }
    let base_pixels: Vec<_> = image
        .pixels()
        .map(|pixel| {
            let [red, green, blue] = pixel.0;
            ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
        })
        .collect();
    let light_levels = (0..LIGHT_LEVEL_COUNT)
        .map(|level| {
            let factor =
                MIN_LIGHT + (1.0 - MIN_LIGHT) * level as f32 / (LIGHT_LEVEL_COUNT - 1) as f32;
            base_pixels
                .iter()
                .map(|color| shade(*color, factor))
                .collect()
        })
        .collect();

    Ok(Texture {
        width,
        height,
        light_levels,
    })
}

fn shade(color: u32, factor: f32) -> u32 {
    let factor = factor.clamp(0.0, 1.0);
    let red = (((color >> 16) & 0xFF) as f32 * factor) as u32;
    let green = (((color >> 8) & 0xFF) as f32 * factor) as u32;
    let blue = ((color & 0xFF) as f32 * factor) as u32;
    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carga_tres_texturas_distintas() {
        let textures = WallTextures::load_embedded().expect("las texturas deben cargar");

        assert_eq!(textures.textures.len(), 3);
        assert!(textures
            .textures
            .iter()
            .all(|texture| texture.width == 128 && texture.height == 128));
        assert!(textures
            .textures
            .iter()
            .all(|texture| texture.light_levels.len() == LIGHT_LEVEL_COUNT));
        assert_ne!(textures.textures[0], textures.textures[1]);
        assert_ne!(textures.textures[1], textures.textures[2]);
    }
}
