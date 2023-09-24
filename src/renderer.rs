use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use std::ops::Add;

const GRID_X_SIZE: u32 = 64;
const GRID_Y_SIZE: u32 = 32;
const DOT_SIZE: u32 = 20;
const BACKGROUND_COLOR: Color = Color::RGB(97, 134, 169);
const PIXEL_COLOR: Color = Color::RGB(33, 41, 70);

#[derive(Copy, Clone)]
pub struct Point(pub i32, pub i32);
impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

pub trait Renderer {
    fn render_screen_ram(&mut self, screen_ram: [[u8; 64]; 32]) -> Result<(), String>;
}

pub struct Sdl2Renderer {
    canvas: WindowCanvas,
}

impl Sdl2Renderer {
    pub fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("CHIP-8", GRID_X_SIZE * DOT_SIZE, GRID_Y_SIZE * DOT_SIZE)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        canvas.set_draw_color(BACKGROUND_COLOR);
        canvas.clear();
        canvas.present();

        Ok(Self { canvas })
    }

    fn draw_pixel(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;
        self.canvas.set_draw_color(PIXEL_COLOR);
        self.canvas.fill_rect(Rect::new(
            x * DOT_SIZE as i32,
            y * DOT_SIZE as i32,
            DOT_SIZE,
            DOT_SIZE,
        ))?;
        Ok(())
    }

    fn draw_background(&mut self) {
        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();
    }
}

impl Renderer for Sdl2Renderer {
    fn render_screen_ram(&mut self, screen_ram: [[u8; 64]; 32]) -> Result<(), String> {
        self.draw_background();
        for x in 0..64 {
            for y in 0..32 {
                if screen_ram[y][x] == 1 {
                    self.draw_pixel(&Point(x as i32, y as i32))?;
                }
            }
        }
        self.canvas.present();
        Ok(())
    }
}
