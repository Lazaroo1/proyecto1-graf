use image::ImageFormat;

const WALL_TEXTURES: [&[u8]; 3] = [
    include_bytes!("../assets/texturas/ladrillo_rojo.png"),
    include_bytes!("../assets/texturas/piedra_azul.png"),
    include_bytes!("../assets/texturas/piedra_musgo.png"),
];

#[derive(Debug, PartialEq)]
struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    shaded_pixels: Vec<u32>,
}

#[derive(Debug)]
pub struct WallTextures {
    textures: Vec<Texture>,
}

pub struct TextureColumn<'a> {
    texture: &'a Texture,
    x: usize,
    shaded: bool,
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

    pub fn column(&self, wall_type: u8, u: f32, shaded: bool) -> TextureColumn<'_> {
        let index = wall_type.saturating_sub(1) as usize % self.textures.len();
        let texture = &self.textures[index];
        let u = if u.is_finite() {
            u.rem_euclid(1.0)
        } else {
            0.0
        };
        let x = (u * texture.width as f32) as usize;

        TextureColumn { texture, x, shaded }
    }
}

impl TextureColumn<'_> {
    pub fn sample(&self, v: f32) -> u32 {
        let v = if v.is_finite() {
            v.clamp(0.0, 1.0 - f32::EPSILON)
        } else {
            0.0
        };
        let y = (v * self.texture.height as f32) as usize;
        let pixels = if self.shaded {
            &self.texture.shaded_pixels
        } else {
            &self.texture.pixels
        };
        pixels[y * self.texture.width + self.x]
    }
}

fn decode_png(bytes: &[u8], wall_type: usize) -> Result<Texture, String> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|error| format!("No se pudo cargar la textura {wall_type}: {error}"))?
        .to_rgb8();
    let width = image.width() as usize;
    let height = image.height() as usize;
    let pixels: Vec<_> = image
        .pixels()
        .map(|pixel| {
            let [red, green, blue] = pixel.0;
            ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
        })
        .collect();
    let shaded_pixels = pixels.iter().map(|color| shade(*color, 0.72)).collect();

    Ok(Texture {
        width,
        height,
        pixels,
        shaded_pixels,
    })
}

fn shade(color: u32, factor: f32) -> u32 {
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
        assert_ne!(textures.textures[0], textures.textures[1]);
        assert_ne!(textures.textures[1], textures.textures[2]);
    }
}
