use std::{thread, time};
use std::sync::{Arc, Mutex};
use gtk::{Application, ApplicationWindow, DrawingArea, Inhibit};
use gtk::prelude::*;
use glib;

mod chip8;

fn main() -> std::io::Result<()> {
    let mut chip8 = chip8::Chip8Emulator::new();
    chip8.initialize();
    chip8.load_game("pong.rom")?;
    let chip8 = Arc::new(Mutex::new(chip8));

    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    let screen = [false; 64 * 32];

    app.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(64*10)
            .default_height(32*10)
            .title("Chip 8 Emulator")
            .build();

        {
            let chip8 = chip8.clone();
            let draw_area = DrawingArea::new();
            win.add(&draw_area);
            draw_area.connect_draw(move |_unused, f| {
                let mut chip8 = chip8.lock().unwrap();

                chip8.emulate_cycle();

                f.set_source_rgb(0.0, 0.0, 0.0);
                f.paint();
                f.set_source_rgb(1.0, 1.0, 1.0);
                for col in 0..64 {
                    for row in 0..32 {
                        if chip8.screen[row * 32 + col] {
                            f.rectangle((col*10) as f64, (row*10) as f64, 10.0, 10.0);
                        } 
                    }
                }
                f.fill();

                chip8.set_keys();

                Inhibit(false)
            });

            glib::source::timeout_add_local(time::Duration::from_millis(17), move || {
                draw_area.queue_draw();
                Continue(true)
            });
        }
        win.show_all();
    });

    app.run();

    Ok(())
}
