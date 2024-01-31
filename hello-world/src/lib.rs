use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
};
use extism_pdk::*;

const SCREEN_WIDTH: usize = 32;
const SCREEN_HEIGHT: usize = 16;

struct DisplayBuffer {
    data: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

impl DisplayBuffer {
    pub fn new() -> Self {
        Self {
            data: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        let mut output = vec![0u8; 32 * 16 / 8];
        for (row, row_data) in self.data.into_iter().enumerate() {
            for (col, elem) in row_data.into_iter().enumerate() {
                let idx = col + (row * 32);
                if elem {
                    output[idx / 8] |= 1 << (idx % 8);
                }
            }
        }

        output
    }
}

impl DrawTarget for DisplayBuffer {
    type Color = BinaryColor;
    type Error = ();

    fn clear(&mut self, _color: Self::Color) -> Result<(), Self::Error> {
        for row in &mut self.data {
            for col in row.into_iter() {
                *col = false;
            }
        }
        Ok(())
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x < 0 || coord.y < 0 {
                continue;
            }

            let (x, y) = (coord.x as usize, coord.y as usize);
            self.data[y][x] = match color {
                BinaryColor::On => true,
                BinaryColor::Off => false,
            };
        }

        Ok(())
    }
}

impl OriginDimensions for DisplayBuffer {
    fn size(&self) -> Size {
        Size::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
    }
}

#[host_fn]
extern "ExtismHost" {
    fn write_region(
        position_x: u32,
        position_y: u32,
        width: u32,
        height: u32,
        input_data: Vec<u8>,
    ) -> ();

    fn render(rows_to_update: Vec<u8>) -> ();
}

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let mut buffer = DisplayBuffer::new();

    let text = embedded_graphics::text::Text::new(
        "Hello",
        Point::new(0, 7),
        MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
    );
    text.draw(&mut buffer).unwrap();

    let text = embedded_graphics::text::Text::new(
        "world!",
        Point::new(0, 7 + 8),
        MonoTextStyle::new(&FONT_5X8, BinaryColor::On),
    );
    text.draw(&mut buffer).unwrap();
    let text_buffer_data = buffer.to_vec();
    let rows_to_update = (0..=SCREEN_HEIGHT as u8).into_iter().collect();

    unsafe {
        write_region(
            0,
            0,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            text_buffer_data,
        )?
    };
    unsafe { render(rows_to_update)? };
    Ok(())
}
