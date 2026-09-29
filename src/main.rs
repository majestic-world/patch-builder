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
    #[cfg(windows)]
    center_on_work_area(&ui);

    ui.run()?;
    Ok(())
}

/// Places the window in the middle of the primary monitor's work area (the screen minus the taskbar).
/// On failure the system's default placement stays.
#[cfg(windows)]
fn center_on_work_area(ui: &AppWindow) {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETWORKAREA, SystemParametersInfoW};

    let mut area = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    // SAFETY: SPI_GETWORKAREA writes exactly one RECT through the pointer, which outlives the call.
    let found = unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, (&raw mut area).cast(), 0) } != 0;
    if !found {
        return;
    }
    // The work area is in the process's DPI view; dividing by that DPI gives Slint's logical pixels.
    // SAFETY: plain query without arguments.
    let scale = unsafe { GetDpiForSystem() } as f32 / 96.0;
    let (left, top) = (area.left as f32 / scale, area.top as f32 / scale);
    let (width, height) = ((area.right - area.left) as f32 / scale, (area.bottom - area.top) as f32 / scale);
    let x = left + ((width - ui.get_initial_width()) / 2.0).max(0.0);
    let y = top + ((height - ui.get_initial_height()) / 2.0).max(0.0);
    ui.window().set_position(slint::LogicalPosition::new(x, y));
}

/// Borderless window that keeps the system shadow and Windows 11 rounded corners.
fn native_frame(attributes: WindowAttributes) -> WindowAttributes {
    #[cfg(windows)]
    {
        use slint::winit_030::winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};
        attributes
            .with_undecorated_shadow(true)
            // DWM rounds the window with a hard, jagged clip on GPU-rendered windows. The app paints its
            // own anti-aliased frame with a slightly larger radius inside a transparent window, so that
            // clip only ever cuts pixels that are already transparent; DWM still rounds the shadow.
            .with_corner_preference(CornerPreference::Round)
            .with_border_color(None)
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
