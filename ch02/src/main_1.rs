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
        info!("Found display {:?}", video_ctx.display_name(n)?);
    }
    let mut window = video_ctx
        .window("ROOM4DOOM", 640, 480)
        .position_centered()
        .build()?;

    if let Some(fullscreen) = options.fullscreen {
        if fullscreen {
            window.set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
        } else {
            window.set_fullscreen(sdl2::video::FullscreenType::Off)?;
        }
    }

    let canvas = window.into_canvas().software().build()?;
    let mut buffer = Framebuffer::new(canvas, 640, 480);

    // Main loop here
    let mut ticks = TimeStep::new();
    let mut running = true;
    loop {
        if !running {
            break;
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
                    _ => {}
                }
            }
        });

        let step = 15;
        let mut c = 0;
        for n in (0..480).step_by(step) {
            if c >= 32 {
                break;
            }
            for y in 0..step {
                buffer.set_pixel(0, n + y, &px16(c, 0, 0));
                buffer.set_pixel(1, n + y, &px16(c, 0, 0));
                buffer.set_pixel(2, n + y, &px16(c, 0, 0));
                buffer.set_pixel(3, n + y, &px16(0, c, 0));
                buffer.set_pixel(4, n + y, &px16(0, c, 0));
                buffer.set_pixel(5, n + y, &px16(0, c, 0));
                buffer.set_pixel(6, n + y, &px16(0, 0, c));
                buffer.set_pixel(7, n + y, &px16(0, 0, c));
                buffer.set_pixel(8, n + y, &px16(0, 0, c));
            }
            c += 1;
        }

        for _ in 0..1000 {
            unsafe {
                let x = rand() % 640;
                let y = rand() % 480;
                buffer.set_pixel(x as usize, y as usize, &px16(31, 0, 0));
            }
        }

        buffer.flip();
        buffer.blit();
        buffer.clear(&[0, 0]);
    }

    Ok(())
}
