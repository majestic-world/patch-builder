#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build;
mod comparison;
mod error;
mod format;
mod manifest;
mod preview;
mod scan;
mod session;
mod settings;
mod tree;
mod view;

use slint::ComponentHandle;
use slint::winit_030::WinitWindowAccessor;
use slint::winit_030::winit::window::{ResizeDirection, WindowAttributes};

use session::Session;
use view::{SUMMARY_PROMPT, Table};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .with_winit_window_attributes_hook(native_frame)
        .select()?;

    let ui = AppWindow::new()?;
    let table = Table::install(&ui);
    if std::env::args().skip(1).any(|arg| arg == "--preview") {
        ui.set_source_path(preview::SOURCE_PATH.into());
        ui.set_update_tree_path(preview::UPDATE_TREE_PATH.into());
        table.show(&ui, preview::files(), SUMMARY_PROMPT);
        table.select(&ui, preview::SELECTED_PATH);
    } else {
        Session::start(&ui, table);
    }
    install_window_chrome(&ui);

    ui.run()?;
    Ok(())
}

/// Borderless window that keeps the system shadow and Windows 11 rounded corners.
fn native_frame(attributes: WindowAttributes) -> WindowAttributes {
    #[cfg(windows)]
    {
        use slint::winit_030::winit::platform::windows::{Color, CornerPreference, WindowAttributesExtWindows};
        attributes
            .with_undecorated_shadow(true)
            .with_corner_preference(CornerPreference::Round)
            .with_border_color(Some(Color::from_rgb(0xD9, 0xDE, 0xE8)))
    }
    #[cfg(not(windows))]
    attributes
}

/// Moving and resizing a borderless window; minimize, maximize and close are Slint builtins.
fn install_window_chrome(ui: &AppWindow) {
    // A failed drag or resize request leaves the window where it is; nothing to recover.
    ui.on_start_drag({
        let ui = ui.as_weak();
        move || {
            let Some(ui) = ui.upgrade() else { return };
            ui.window().with_winit_window(|window| {
                let _ = window.drag_window();
            });
        }
    });
    ui.on_start_resize({
        let ui = ui.as_weak();
        move |edge| {
            let Some(ui) = ui.upgrade() else { return };
            ui.window().with_winit_window(|window| {
                let _ = window.drag_resize_window(resize_direction(edge));
            });
        }
    });
}

fn resize_direction(edge: ResizeEdge) -> ResizeDirection {
    match edge {
        ResizeEdge::North => ResizeDirection::North,
        ResizeEdge::South => ResizeDirection::South,
        ResizeEdge::East => ResizeDirection::East,
        ResizeEdge::West => ResizeDirection::West,
        ResizeEdge::NorthEast => ResizeDirection::NorthEast,
        ResizeEdge::NorthWest => ResizeDirection::NorthWest,
        ResizeEdge::SouthEast => ResizeDirection::SouthEast,
        ResizeEdge::SouthWest => ResizeDirection::SouthWest,
    }
}
