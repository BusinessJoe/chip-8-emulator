use game_loop::{game_loop, Time, TimeTrait as _};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use std::env;
use std::time::Duration;
use winit::{
    dpi::LogicalSize, event::VirtualKeyCode, event_loop::EventLoop, window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod chip8;

struct Game {
    /// Emulator.
    emu: chip8::Chip8Emulator,
    /// Software renderer.
    pixels: Pixels,
    /// Event manager.
    input: WinitInputHelper,
}

impl Game {
    fn new(pixels: Pixels, rom_path: &str) -> Self {
        let chip8 = {
            let mut chip8 = chip8::Chip8Emulator::new();
            chip8.initialize();
            chip8.load_game(rom_path).unwrap();
            chip8
        };

        Self {
            emu: chip8,
            pixels,
            input: WinitInputHelper::new(),
        }
    }

    fn update_keys(&mut self) {
        let keys: [bool; 16] = [
            self.input.key_held(VirtualKeyCode::Key0),
            self.input.key_held(VirtualKeyCode::Key1),
            self.input.key_held(VirtualKeyCode::Key2),
            self.input.key_held(VirtualKeyCode::Key3),
            self.input.key_held(VirtualKeyCode::Key4),
            self.input.key_held(VirtualKeyCode::Key5),
            self.input.key_held(VirtualKeyCode::Key6),
            self.input.key_held(VirtualKeyCode::Key7),
            self.input.key_held(VirtualKeyCode::Key8),
            self.input.key_held(VirtualKeyCode::Key9),
            self.input.key_held(VirtualKeyCode::A),
            self.input.key_held(VirtualKeyCode::B),
            self.input.key_held(VirtualKeyCode::C),
            self.input.key_held(VirtualKeyCode::D),
            self.input.key_held(VirtualKeyCode::E),
            self.input.key_held(VirtualKeyCode::F),
        ];

        self.emu.set_keys(&keys);
    }
}

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

const FPS: usize = 500;
const TIME_STEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);

fn main() -> std::io::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 10.0, HEIGHT as f64 * 10.0);
        WindowBuilder::new()
            .with_title("Chip 8 Emulator")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let game = Game::new(pixels, &args[1]);

    game_loop(
        event_loop,
        window,
        game,
        FPS as u32,
        0.1,
        move |g| {
            g.game.emu.emulate_cycle();
        },
        move |g| {
            // Drawing
            g.game.emu.draw_screen(g.game.pixels.get_frame());
            if let Err(e) = g.game.pixels.render() {
                error!("pixels.render() failed: {}", e);
                g.exit();
            }

            // Sleep the main thread to limit drawing to the fixed time step.
            // See: https://github.com/parasyte/pixels/issues/174
            let dt = TIME_STEP.as_secs_f64() - Time::now().sub(&g.current_instant());
            if dt > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(dt));
            }
        },
        |g, event| {
            // Let winit_input_helper collect events to build its state.
            if g.game.input.update(event) {
                // Update controls
                g.game.update_keys();

                // Close events
                if g.game.input.key_pressed(VirtualKeyCode::Escape) || g.game.input.quit() {
                    g.exit();
                    return;
                }

                // Resize the window
                if let Some(size) = g.game.input.window_resized() {
                    g.game.pixels.resize_surface(size.width, size.height);
                }
            }
        },
    );
}
