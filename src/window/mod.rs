mod keyboard;
mod settings;

#[cfg_attr(feature = "sdl2", path = "sdl2/mod.rs")]
#[cfg_attr(feature = "winit", path = "winit/mod.rs")]
mod window_wrapper;

use crate::{
    bridge::UiCommand,
    editor::{DrawCommand, WindowCommand},
    renderer::Renderer,
    settings::SETTINGS,
    INITIAL_DIMENSIONS,
};
use crossfire::mpsc::TxUnbounded;
use skulpin::LogicalSize;
use std::sync::{atomic::AtomicBool, mpsc::Receiver, Arc};

#[cfg(feature = "sdl2")]
pub use window_wrapper::start_loop;
#[cfg(feature = "winit")]
pub use window_wrapper::start_loop;

pub use settings::*;

pub fn window_geometry() -> Option<Result<(u64, u64), String>> {
    let prefix = "--geometry=";

    std::env::args().find_map(|arg| {
        if let Some(input) = &arg.strip_prefix(prefix) {
            Some(parse_dimension_str(input))
        } else {
            None
        }
    })
}

fn parse_dimension_str(dimension_str: &str) -> Result<(u64, u64), String> {
    let invalid_parse_err = format!(
        "Invalid geometry: {}\nValid format: <width>x<height>",
        dimension_str
    );

    dimension_str
        .split('x')
        .map(|dimension| {
            dimension
                .parse::<u64>()
                .map_err(|_| invalid_parse_err.as_str())
                .and_then(|dimension| {
                    if dimension > 0 {
                        Ok(dimension)
                    } else {
                        Err("Invalid geometry: Window dimensions should be greater than 0.")
                    }
                })
        })
        .collect::<Result<Vec<_>, &str>>()
        .and_then(|dimensions| {
            if let [width, height] = dimensions[..] {
                Ok((width, height))
            } else {
                Err(invalid_parse_err.as_str())
            }
        })
        .map_err(|msg| msg.to_owned())
}

fn window_geometry_saved() -> Option<Result<(u64, u64), String>> {
    // TODO: Fix settings
    let remember_dimensions = SETTINGS.get::<WindowSettings>().remember_dimensions;

    if remember_dimensions {
        let mut stdpath = SETTINGS.get::<CacheSettings>().stdpath;
        while stdpath.is_empty() {
            stdpath = SETTINGS.get::<CacheSettings>().stdpath;
        }
        let stdpath = std::path::PathBuf::from(stdpath);
        let position_file = stdpath.join("neovide_settings.txt");
        println!("position_file = {:?}", position_file);
        if !position_file.exists() {
            std::fs::write(&position_file, "window_dimensions=120x110")
                .map_err(|err| err.to_string())
                .ok()?;
        }
        let content = std::fs::read_to_string(position_file)
            .map_err(|err| err.to_string())
            .ok()?;
        let dimensions = content.split('\n').find_map(|line| {
            if let Some(dimensions_str) = line.strip_prefix("window_dimensions=") {
                Some(parse_dimension_str(dimensions_str))
            } else {
                None
            }
        });

        dimensions
    } else {
        None
    }
}

pub fn window_geometry_or_default() -> (u64, u64) {
    match window_geometry() {
        Some(Ok(dimension)) => dimension,
        None => match window_geometry_saved() {
            Some(Ok(dimensions)) => {
                println!("using save dimensions = {:?}", dimensions);
                dimensions
            }
            Some(Err(err)) => {
                eprintln!("error in saved_dimensions: {}", err);
                INITIAL_DIMENSIONS
            }
            None => INITIAL_DIMENSIONS,
        },
        Some(Err(err)) => {
            eprintln!("error in --geometry: {}", err);
            INITIAL_DIMENSIONS
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_fix_dpi() {
    println!("dpi fix applied");
    use winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
    use winapi::um::winuser::SetProcessDpiAwarenessContext;
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

fn handle_new_grid_size(
    new_size: LogicalSize,
    renderer: &Renderer,
    ui_command_sender: &TxUnbounded<UiCommand>,
) {
    if new_size.width > 0 && new_size.height > 0 {
        // Add 1 here to make sure resizing doesn't change the grid size on startup
        let new_width = ((new_size.width + 1) as f32 / renderer.font_width) as u32;
        let new_height = ((new_size.height + 1) as f32 / renderer.font_height) as u32;
        ui_command_sender
            .send(UiCommand::Resize {
                width: new_width,
                height: new_height,
            })
            .ok();
    }
}

pub fn create_window(
    batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
    window_command_receiver: Receiver<WindowCommand>,
    ui_command_sender: TxUnbounded<UiCommand>,
    running: Arc<AtomicBool>,
) {
    let (width, height) = window_geometry_or_default();

    let renderer = Renderer::new(batched_draw_command_receiver);
    let logical_size = LogicalSize {
        width: (width as f32 * renderer.font_width) as u32,
        height: (height as f32 * renderer.font_height + 1.0) as u32,
    };

    #[cfg(target_os = "windows")]
    windows_fix_dpi();

    start_loop(
        window_command_receiver,
        ui_command_sender,
        running,
        logical_size,
        renderer,
    );
}
