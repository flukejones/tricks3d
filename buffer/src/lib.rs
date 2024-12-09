use sdl2::{
    pixels::PixelFormatEnum,
    render::{Canvas, TextureCreator},
    video::{FullscreenType, Window, WindowContext},
};

const SOFT_PIXEL_CHANNELS: usize = 2;
const PIXEL_FORMAT: PixelFormatEnum = PixelFormatEnum::RGB555;

/// For use with `PixelFormatEnum::BGR555` and channel size 2
#[inline(always)]
pub const fn px16(r: u16, g: u16, b: u16) -> [u8; 2] {
    ((b & 31) + ((g & 31) << 5) + ((r & 31) << 10)).to_le_bytes()
}

/// A structure holding display data
pub struct Framebuffer {
    _tc: TextureCreator<WindowContext>,
    texture: sdl2::render::Texture,
    size: (usize, usize),
    stride: usize,
    disp_buf: Vec<u8>,
    draw_buf: Vec<u8>,
    canvas: Canvas<Window>,
}

impl Framebuffer {
    pub fn new(mut canvas: Canvas<Window>, width: usize, height: usize) -> Self {
        if matches!(canvas.window().fullscreen_state(), FullscreenType::True) {
            let requested_ratio = width as f32 / height as f32;
            let height = canvas.window().drawable_size().1;
            let ratio = height as f32 / requested_ratio;
            canvas
                .set_logical_size(ratio as u32, height as u32)
                .unwrap();
        } else {
            canvas
                .set_logical_size(width as u32, height as u32)
                .unwrap();
        }

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_streaming(Some(PIXEL_FORMAT), width as u32, height as u32)
            .unwrap();
        Self {
            _tc: texture_creator,
            texture,
            size: (width, height),
            stride: width * SOFT_PIXEL_CHANNELS,
            disp_buf: vec![0; (width * height) * SOFT_PIXEL_CHANNELS],
            draw_buf: vec![0; (width * height) * SOFT_PIXEL_CHANNELS],
            canvas,
        }
    }

    pub fn clear(&mut self, colour: &[u8; SOFT_PIXEL_CHANNELS]) {
        self.draw_buf
            .chunks_mut(SOFT_PIXEL_CHANNELS)
            .for_each(|n| n.copy_from_slice(colour));
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, colour: &[u8; SOFT_PIXEL_CHANNELS]) {
        #[cfg(feature = "safety_check")]
        if x >= self.size.width || y >= self.size.height {
            dbg!(x, y);
            panic!();
        }

        let pos = y * self.stride + x * SOFT_PIXEL_CHANNELS;
        #[cfg(not(feature = "safety_check"))]
        unsafe {
            self.draw_buf
                .get_unchecked_mut(pos..pos + SOFT_PIXEL_CHANNELS)
                .copy_from_slice(colour);
        }
        #[cfg(feature = "safety_check")]
        self.buffer2[pos..pos + SOFT_PIXEL_CHANNELS].copy_from_slice(colour);
    }

    /// Read the colour of a single pixel at X|Y
    pub fn read_pixel(&self, x: usize, y: usize) -> [u8; SOFT_PIXEL_CHANNELS] {
        let pos = y * self.stride + x * SOFT_PIXEL_CHANNELS;
        let mut slice = [0u8; SOFT_PIXEL_CHANNELS];
        slice.copy_from_slice(&self.draw_buf[pos..pos + SOFT_PIXEL_CHANNELS]);
        slice
    }

    /// Read the full buffer
    pub fn buf_mut(&mut self) -> &mut [u8] {
        &mut self.draw_buf
    }

    pub fn pitch(&self) -> usize {
        self.size.0 * SOFT_PIXEL_CHANNELS
    }

    pub fn flip(&mut self) {
        std::mem::swap(&mut self.disp_buf, &mut self.draw_buf);
    }

    /// Throw buffer1 at the screen
    pub fn blit(&mut self) {
        self.texture
            .update(None, &self.disp_buf, self.stride)
            .unwrap();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}
