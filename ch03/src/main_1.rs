use std::error::Error;

use argh::FromArgs;
use buffer::{Framebuffer, px16};
use log::info;
use sdl2::{event::Event, keyboard::Scancode, libc::rand};
use simplelog::TermLogger;
use timestep::TimeStep;

/// CLI options for the example
#[derive(Debug, Clone, FromArgs)]
struct CLIOptions {
    /// verbose level: off, error, warn, info, debug
    #[argh(option, short = 'v')]
    verbose: Option<log::LevelFilter>,
    /// fullscreen?
    #[argh(option, short = 'f')]
    pub fullscreen: Option<bool>,
}

const NAME: &str = "ch03_1";
const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn main() -> Result<(), Box<dyn Error>> {
    let options: CLIOptions = argh::from_env();

    TermLogger::init(
        log::LevelFilter::Info,
        simplelog::ConfigBuilder::default()
            .set_time_level(options.verbose.unwrap_or(log::LevelFilter::Debug))
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    let sdl_ctx = sdl2::init()?;
    info!("Init SDL2 main");
    let _snd_ctx = sdl_ctx.audio()?;
    info!("Init SDL2 sound");
    let video_ctx = sdl_ctx.video()?;
    info!("Init SDL2 video");

    let num_disp = video_ctx.num_video_displays()?;
    for n in 0..num_disp {
        info!(
            "Found display {}, mode: {:?}",
            video_ctx.display_name(n)?,
            video_ctx.desktop_display_mode(n)?
        );
    }
    let mode = video_ctx.current_display_mode(0).unwrap();
    let window = {
        let fullscreen = options.fullscreen.unwrap_or_default();
        if fullscreen {
            video_ctx
                .window(
                    NAME,
                    // for plain desktop mode, 320x200 at 4:3 scale, it is normally 16:10
                    // (mode.w as f32 * (mode.h as f32 / mode.w as f32)) as u32,
                    WIDTH, HEIGHT,
                )
                .position_centered()
                .fullscreen_desktop()
                .hidden()
                .build()?
        } else {
            video_ctx
                .window(NAME, WIDTH, HEIGHT)
                .position_centered()
                .resizable()
                .hidden()
                .build()?
        }
    };
    dbg!(window.size());

    let canvas = window.into_canvas().software().build()?;
    let mut buffer = Framebuffer::new(canvas, mode.w as usize, mode.h as usize);

    // Main loop here
    let mut ticks = TimeStep::new();
    let mut running = true;
    let mut resized = false;
    loop {
        if !running {
            break;
        }
        if resized {
            let mut canvas = buffer.canvas;
            canvas.clear();
            buffer = Framebuffer::new(canvas, WIDTH as usize, HEIGHT as usize);
            resized = false;
        }

        ticks.run_this(|| {
            while let Some(event) = sdl_ctx.event_pump().unwrap().poll_event() {
                match event {
                    Event::KeyDown {
                        scancode: Some(sc), ..
                    } => match sc {
                        _ => {}
                    },
                    Event::KeyUp {
                        scancode: Some(sc), ..
                    } => match sc {
                        Scancode::Escape => running = false,
                        _ => {}
                    },

                    Event::Quit { .. } => running = false, // Early out if Quit

                    Event::Window { win_event, .. } => match win_event {
                        sdl2::event::WindowEvent::Moved(_, _) => {
                            resized = true;
                        }
                        sdl2::event::WindowEvent::Resized(_, _) => {
                            resized = true;
                        }
                        sdl2::event::WindowEvent::DisplayChanged(_) => {
                            resized = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        });

        let step = 15;
        let mut c: u16 = 0;
        let pitch = buffer.pitch();
        let buf = buffer.buf_mut();
        for n in (0..HEIGHT as usize).step_by(step) {
            if n + step as usize >= HEIGHT as usize {
                break;
            }
            let red = px16(c, 0, 0);
            let gre = px16(0, c, 0);
            let blu = px16(0, 0, c);
            for y in 0..step {
                let mut pos = (n + y) * pitch + 0 * 2;
                unsafe {
                    for _ in 0..5 {
                        buf.get_unchecked_mut(pos..pos + 2).copy_from_slice(&red);
                        pos += 2;
                    }
                    for _ in 0..5 {
                        buf.get_unchecked_mut(pos..pos + 2).copy_from_slice(&gre);
                        pos += 2;
                    }
                    for _ in 0..5 {
                        buf.get_unchecked_mut(pos..pos + 2).copy_from_slice(&blu);
                        pos += 2;
                    }
                }
                // buffer.set_pixel(0, n + y, &px16(c, 0, 0));
            }
            c += 1;
        }

        for _ in 0..1000 {
            unsafe {
                let x = rand() % WIDTH as i32;
                let y = rand() % HEIGHT as i32;
                let pos = y as usize * pitch + x as usize * 2;
                buf.get_unchecked_mut(pos..pos + 2).copy_from_slice(&px16(
                    rand() as u16 % 31,
                    rand() as u16 % 31,
                    rand() as u16 % 31,
                ));
                // buffer.set_pixel(x as usize, y as usize, &px16(31, 0, 0));
            }
        }

        buffer.flip();
        buffer.blit();
        buffer.clear(&[0, 0]);

        // if let Some(fps) = ticks.frame_rate() {
        //     info!("FPS = {}", fps.frames);
        // }
    }

    Ok(())
}
