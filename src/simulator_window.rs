//pub mod display;
//
use std::cell::RefCell;
use std::println;
//use std::rc::Rc;
use once_cell::sync::OnceCell;
use std::thread_local;
use std::vec;
use std::vec::Vec;
use std::string::String;
//use std::{ panic, boxed::Box, process };
use std::{ process };

use minifb::{Key, MouseButton, Window, WindowOptions};
//use std::fs;
//use font8x8::UnicodeFonts;

//use resvg::tiny_skia::Pixmap;
//use resvg::usvg::{Options, Tree};
//use resvg::render;
//use resvg::tiny_skia::{Transform};
//use resvg::usvg::fontdb::Database;

const GLYPH_W: u32 = 8;
const GLYPH_H: u32 = 8;

/* ---------- global holder ---------- */
/*
thread_local! {
static SIMULATOR_WINDOW: RefCell<SimulatorWindow> = 
    RefCell::new(SimulatorWindow::new());
}

fn simulator_window_options() -> minifb::WindowOptions {
    WindowOptions {
        resize: false,
        scale: minifb::Scale::X1,
        ..WindowOptions::default()
    }
}
fn simulator_window() -> &'static RefCell<SimulatorWindow> {
    //SIMULATOR_WINDOW.get_or_init( || RefCell::new(SimulatorWindow::new()));
    SIMULATOR_WINDOW
}
*/

/*
type SharedWin = Rc<RefCell<SimulatorWindow>>;

static mut SINGLETON: Option<SharedWin> = None;

pub fn simulator_window() -> SharedWin {
    unsafe {
        SINGLETON.get_or_insert_with(|| {
            let win = SimulatorWindow::new();
            Rc::new(RefCell::new(win))
        }).clone()
    }
}
*/

thread_local! {
    static WINDOW: OnceCell<RefCell<SimulatorWindow>> = OnceCell::new();
}

pub fn simulator_window_init() {
    WINDOW.with(|cell| {
        if cell.get().is_none() {
            //panic::set_hook(Box::new(|info| {
            //    error!("Panic occurred: {}", info);
            //    process::exit(1);
            //}));

            debug!("simulator_window_init()");
            let _ = cell.set(RefCell::new(SimulatorWindow::new()));
            simulator_window_update();
            simulator_window_update();
            simulator_window_update();
            simulator_window_update();
            simulator_window_update();
        }
    });
}

pub fn simulator_window_update() {
    /*WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut win = win_cell.borrow_mut();

        win.update();
    });*/

    WINDOW.with(|cell| {
    if let Some(win_cell) = cell.get() {
        let mut win = win_cell.borrow_mut();
        win.update();
    }
});

}

pub fn simulator_window_set_memory_stats(heap_bytes: usize, stack_bytes: usize) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut win = win_cell.borrow_mut();

        win.heap_bytes = heap_bytes;
        win.stack_bytes = stack_bytes;
    });
}

pub fn simulator_window_has_touch() -> bool {

    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        // TODO this function - find a proper name - it sets mouse_clicked flag
        window.is_mouse_pressed();
        window.mouse_clicked
    })
}

pub fn simulator_window_touch_position() -> (u16, u16) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let window = win_cell.borrow_mut();
        window.mouse_position_screen()
    })
}

pub fn simulator_window_draw_rect(x: u16, y: u16, w: u16, h: u16, color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_rect(x, y, w, h, color);
        window.update_if_needed();
    });
}

pub fn simulator_window_fill_rect(x: u16, y: u16, w: u16, h: u16, color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.fill_rect(x, y, w, h, color);
        window.update_if_needed();
    });
}

pub fn simulator_window_draw_led(color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_led(color);
        window.update_if_needed();
    });
}

pub fn simulator_window_draw_line(x: u16, y: u16, width: u16, pixels: &[u16]) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_line(y, x, x + width, pixels);
        window.update_if_needed();
    });
}

pub fn simulator_window_is_mouse_pressed() -> Option<bool> {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        let is_pressed = window.is_mouse_pressed();
        is_pressed
    })
}

// TODO check if it's working properly
pub fn simulator_window_is_q_pressed() -> bool {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let window = win_cell.borrow_mut();

        let is_key_pressed = window.window.is_key_pressed(Key::Q, minifb::KeyRepeat::No);
        is_key_pressed
    })
}

pub fn simulator_window_draw_image() {
}

const WINDOW_WIDTH: u32 = 480;
const WINDOW_HEIGHT: u32 = 720;
const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;
const SCREEN_OFFSET_X: u32 = 80;
const SCREEN_OFFSET_Y: u32 = 80;
const NAME: &str = "Hito Simulator";

struct SimulatorWindow {
    window: Window,
    buffer: Vec<u32>,
    mouse_clicked: bool,
    heap_bytes: usize,
    stack_bytes: usize,
    last_update: std::time::Instant,
}

use std::fmt;

//use crate::log_info;

impl fmt::Debug for SimulatorWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "HitoSimulator", self.width, self.height)
        write!(f, "HitoSimulator")
    }
}

impl SimulatorWindow {

    pub fn new() -> Self {

        let opts = WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        };
        let window = Window::new(NAME, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize, opts).unwrap();
        let mut this = Self {
            //window: Rc::new(window),
            window: window,
            buffer: vec![0u32; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize],
            mouse_clicked: false,
            heap_bytes: 0,
            stack_bytes: 0,
            last_update: std::time::Instant::now(),
        };
        this.init_background_png();
        this
    }

    /*
    fn init_background_svg(&mut self) {
        //let svg_data = fs::read("./res/simulator/hito_simulator_background.svg").unwrap();
        let svg_data = include_bytes!("../res/hito_simulator_background.svg");

        let options = resvg::usvg::Options::default();
        //let fontdb = Database::new();
        let tree = resvg::usvg::Tree::from_data(svg_data, &options).unwrap();

        let mut pixmap = Pixmap::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).unwrap();
        render(&tree, Transform::identity(), &mut pixmap.as_mut());

        for y in 0..WINDOW_HEIGHT {
            for x in 0..WINDOW_WIDTH {
                let pixel = pixmap.pixel(x, y).unwrap();
                //println!("({}, {}): r={}, g={}, b={}, a={}", x, y, pixel.red(), pixel.green(), pixel.blue(), pixel.alpha());
                let i = y * WINDOW_WIDTH + x;
                self.buffer[i as usize] = ((pixel.red() as u32) << 16) | ((pixel.green() as u32) << 8) | (pixel.blue() as u32);

                //self.buffer[i as usize] = 0xFF0000;
            }
        }
    }
    */

    fn init_background_png(&mut self) {
        //let img = image::open("./res/simulator/hito_simulator_background.png").expect("Failed to open image");
        const IMAGE_DATA: &[u8] = include_bytes!("../res/hito_simulator_background.png");

        //let img = image::load_from_memory(
            //include_bytes!("../res/simulator/hito_simulator_background.png")
        //).expect("Failed to decode image");

        let img = image::load_from_memory(IMAGE_DATA)
            .expect("Failed to decode background image");

        let rgba = img.to_rgba8(); // converts to ImageBuffer<Rgba<u8>, Vec<u8>>

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let i = (y as usize) * WINDOW_WIDTH as usize + (x as usize);
            if i < self.buffer.len() {
                let i = (y as usize) * WINDOW_WIDTH as usize + (x as usize);
                let rgba = pixel.0;
                self.buffer[i] = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | (rgba[2] as u32);
            }
        }
    }

    fn draw_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u32) {
        for i in 0..h {
            for j in 0..w {
                let px = (x + j) as u32 + (SCREEN_OFFSET_X as u32);
                let py = (y + i) as u32 + (SCREEN_OFFSET_X as u32);
                if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                    let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                    self.buffer[index] = color;
                }
            }
        }
    }

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u32) {
        for i in 0..h {
            for j in 0..w {
                let px = (x + j) as u32 + (SCREEN_OFFSET_X as u32);
                let py = (y + i) as u32 + (SCREEN_OFFSET_X as u32);
                if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                    let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                    self.buffer[index] = color;
                }
            }
        }
    }

    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]) {
        let py = (y as u32) + SCREEN_OFFSET_Y;
        if py >= WINDOW_HEIGHT {
            println!("draw_line: y={} is out of bounds", y);
            return;
        }

        let line_len = (x_end - x_start) as usize;
        for (i, &rgb565_pixel) in pixels.iter().take(line_len).enumerate() {
            let px = (x_start as u32 + i as u32) + SCREEN_OFFSET_X;
            if px < WINDOW_WIDTH {
                let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                // Convert RGB565 to RGB888
                let r8 = ((rgb565_pixel >> 11) & 0x1F) << 3;
                let g8 = ((rgb565_pixel >> 5) & 0x3F) << 2;
                let b8 = (rgb565_pixel & 0x1F) << 3;
                let color32 = ((r8 as u32) << 16) | ((g8 as u32) << 8) | (b8 as u32);
                self.buffer[index] = color32;
            }
        }
    }

    fn draw_led(&mut self, color: u32) {
        // LED position in the simulator window (outside the screen area)
        let led_center_x = 82i32;
        let led_center_y = 58i32;
        let led_radius = 4i32;

        // Draw a filled circle for the LED
        for dy in -led_radius..=led_radius {
            for dx in -led_radius..=led_radius {
                let distance_squared = dx * dx + dy * dy;
                if distance_squared <= led_radius * led_radius {
                    let px = led_center_x + dx;
                    let py = led_center_y + dy;
                    if px >= 0 && py >= 0 && (px as u32) < WINDOW_WIDTH && (py as u32) < WINDOW_HEIGHT {
                        let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                        self.buffer[index] = color;
                    }
                }
            }
        }
    }

    pub fn is_mouse_pressed(&mut self) -> Option<bool> {
        if self.window.get_mouse_down(MouseButton::Left) && !self.mouse_clicked {
            // Mouse button just pressed
            //println!("Mouse pressed");
            self.mouse_clicked = true;
            return Some(true);
        } else if !self.window.get_mouse_down(MouseButton::Left) && self.mouse_clicked {
            // Mouse button just released
            //println!("Mouse released");
            self.mouse_clicked = false;
            //log_info!("Mouse released");
            return Some(false);
        }
        None
    }

    pub fn mouse_position_screen(&self) -> (u16, u16) {
        if let Some((x, y)) = self.window.get_mouse_pos(minifb::MouseMode::Discard) {
            // Adjust for the screen offset
            let adjusted_x = (x as u32).saturating_sub(SCREEN_OFFSET_X);
            let adjusted_y = (y as u32).saturating_sub(SCREEN_OFFSET_Y);
            return (adjusted_x as u16, adjusted_y as u16);
        }
        (0, 0)
        /*
        if self.window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = self.window.get_mouse_pos(minifb::MouseMode::Discard) {
                return Some((x as u16, y as u16));
            }
        }
        None
        */
    }

    /*
    pub fn draw_text(&mut self, text: &str, mut x: u32, mut y: u32, color: u32) {
        for ch in text.chars() {
            if ch == '\n' { y += GLYPH_H + 1; x = 0; continue; }

            // Try blocks in order (Basic Latin, Latin-1, Cyrillic, etc.)
            let glyph_rows = font8x8::BASIC_FONTS.get(ch);
                // .or_else(|| font8x8::Cyrillic::new().get(ch))
                // .or_else(|| font8x8::BoxDrawing::new().get(ch));

            if let Some(rows) = glyph_rows {
                for (row, bits) in rows.iter().enumerate() {
                    let py = y + row as u32;
                    if py >= WINDOW_HEIGHT { continue; }
                    for col in 0..8 {
                        let px = x + col;
                        if px >= WINDOW_WIDTH { continue; }
                        if (bits >> col) & 0x01 != 0 {
                            let idx = (py * WINDOW_WIDTH + px) as usize;
                            if idx < self.buffer.len() {
                                self.buffer[idx] = color;
                            }
                        }
                    }
                }
            }
            x += GLYPH_W + 1;
        }
    }
    */

    fn get_memory_usage_labels(&self) -> (String, String, String) {
        let to_label = |i: usize| -> String {
            if i < 10_000 {
                std::format!("{} B", i)
            } else if i < 1_000_000 {
                std::format!("{:.1} KB", (i as f32) / 1024.0)
            } else {
                std::format!("{:.1} MB", (i as f32) / 1_048_576.0)
            }
        };

        let label1 = std::format!("Heap  : {}", to_label(self.heap_bytes));
        let label2 = std::format!("Stack : {}", to_label(self.stack_bytes));
        let label3 = std::format!("Total : {}", to_label(self.heap_bytes + self.stack_bytes));

        (label1, label2, label3)
    }


    fn draw_memory_overlay(&mut self) {
        let (label1, label2, label3) = self.get_memory_usage_labels();

        // Draw semi-transparent background for text
        let overlay_x = 160;
        let overlay_y = 400;
        let overlay_width = 200;
        let overlay_height = 40;

        for y in overlay_y..overlay_y + overlay_height {
            for x in overlay_x..overlay_x + overlay_width {
                if x < WINDOW_WIDTH && y < WINDOW_HEIGHT {
                    let index = (y * WINDOW_WIDTH + x) as usize;
                    if index < self.buffer.len() {
                        // Semi-transparent dark background
                        self.buffer[index] = 0x202020;
                    }
                }
            }
        }

        //self.draw_text(&label1, overlay_x + 5, overlay_y + 5, 0x00FF00);
        //self.draw_text(&label2, overlay_x + 5, overlay_y + 15, 0x00FF00);
        //self.draw_text(&label3, overlay_x + 5, overlay_y + 25, 0xFFFF00);
    }

    pub fn update(&mut self) {
        if self.window.is_open() {
            self.draw_memory_overlay();

            self.window.update_with_buffer(&self.buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
        } else {
            process::exit(0);
        }
    }

    pub fn update_if_needed(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_update);
        
        // Update at most every 40ms (25 FPS) to avoid excessive updates
        if elapsed >= std::time::Duration::from_millis(40) {
            if self.window.is_open() {
                self.draw_memory_overlay();
                self.window.update_with_buffer(&self.buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
            }
            self.last_update = now;
        }
    }

}
