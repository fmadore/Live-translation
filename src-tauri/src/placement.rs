//! Meeting profiles only move the caption window, never another application.
use crate::{
    errors::{id, AppError},
    overlay::OVERLAY_LABEL,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn fit(saved: &Placement, bounds: &Placement) -> Placement {
    let width = saved.width.clamp(320, 16384).min(bounds.width);
    let height = saved.height.clamp(160, 16384).min(bounds.height);
    Placement {
        x: (saved.x as i64).clamp(
            bounds.x as i64,
            bounds.x as i64 + bounds.width as i64 - width as i64,
        ) as i32,
        y: (saved.y as i64).clamp(
            bounds.y as i64,
            bounds.y as i64 + bounds.height as i64 - height as i64,
        ) as i32,
        width,
        height,
    }
}

#[tauri::command]
pub async fn get_overlay_placement(app: AppHandle) -> Result<Placement, AppError> {
    let win = app
        .get_webview_window(OVERLAY_LABEL)
        .ok_or_else(|| AppError::with(id::OVERLAY_WINDOW, "Overlay unavailable"))?;
    let pos = win
        .outer_position()
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    let size = win
        .inner_size()
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    Ok(Placement {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    })
}

#[tauri::command]
pub async fn set_overlay_placement(app: AppHandle, placement: Placement) -> Result<(), AppError> {
    let win = app
        .get_webview_window(OVERLAY_LABEL)
        .ok_or_else(|| AppError::with(id::OVERLAY_WINDOW, "Overlay unavailable"))?;
    let monitors = win
        .available_monitors()
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    // A disconnected projector must not leave captions outside every display.
    let monitor = monitors.iter().find(|m| {
        let p = m.position();
        let s = m.size();
        placement.x as i64 >= p.x as i64
            && (placement.x as i64) < p.x as i64 + s.width as i64
            && placement.y as i64 >= p.y as i64
            && (placement.y as i64) < p.y as i64 + s.height as i64
    });
    let primary = win
        .primary_monitor()
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    let monitor = monitor
        .or(primary.as_ref())
        .or(monitors.first())
        .ok_or_else(|| AppError::with(id::OVERLAY_WINDOW, "No display available"))?;
    let area = monitor.work_area();
    let safe = fit(
        &placement,
        &Placement {
            x: area.position.x,
            y: area.position.y,
            width: area.size.width,
            height: area.size.height,
        },
    );
    win.set_size(PhysicalSize::new(safe.width, safe.height))
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    win.set_position(PhysicalPosition::new(safe.x, safe.y))
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_monitor_and_extreme_sizes_stay_in_work_area() {
        let bounds = Placement {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1040,
        };
        let safe = fit(
            &Placement {
                x: i32::MAX,
                y: i32::MIN,
                width: u32::MAX,
                height: 0,
            },
            &bounds,
        );
        assert_eq!(
            (safe.x, safe.y, safe.width, safe.height),
            (-1920, 0, 1920, 160)
        );
    }
}
