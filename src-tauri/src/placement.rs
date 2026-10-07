//! Where the caption overlay sits: read and set for meeting profiles and for the placement the
//! interface remembers per display layout. Only the caption window is ever moved, never
//! another application.
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
    // The far edge saturates: no position lies past `i32::MAX`, however far the area reaches.
    Placement {
        x: saved.x.clamp(
            bounds.x,
            bounds.x.saturating_add_unsigned(bounds.width - width),
        ),
        y: saved.y.clamp(
            bounds.y,
            bounds.y.saturating_add_unsigned(bounds.height - height),
        ),
        width,
        height,
    }
}

/// One display as a layout signature sees it: where it sits on the desktop, how many physical
/// pixels it has, and how far Windows scales them.
#[derive(Clone, Copy, Debug)]
struct Display {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f64,
}

/// A name for the arrangement of displays, so a remembered overlay placement belongs to the
/// layout it was made on: plugging the projector back in brings back the projector placement,
/// and the laptop on its own keeps its own.
///
/// Sorted, because the order the system lists monitors in is not part of the layout. The
/// scale is a whole percentage, as Windows offers it, so a factor that arrives a rounding
/// error away still names the same layout. The work area is left out: moving the taskbar does
/// not make it a different room, and a restore is clamped to the work area anyway.
fn layout_signature(displays: &[Display]) -> String {
    let mut parts: Vec<String> = displays
        .iter()
        .map(|d| {
            let (x, y, width, height) = (d.x, d.y, d.width, d.height);
            let percent = d.scale * 100.0;
            format!("{x},{y} {width}x{height} {percent:.0}%")
        })
        .collect();
    parts.sort_unstable();
    parts.join("; ")
}

/// The current display layout's signature; see `layout_signature`. The interface keeps the
/// remembered placements and asks for this when it saves one and when it restores at launch.
#[tauri::command]
pub async fn display_layout(app: AppHandle) -> Result<String, AppError> {
    let monitors = app
        .available_monitors()
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    let displays: Vec<Display> = monitors
        .iter()
        .map(|m| Display {
            x: m.position().x,
            y: m.position().y,
            width: m.size().width,
            height: m.size().height,
            scale: m.scale_factor(),
        })
        .collect();
    Ok(layout_signature(&displays))
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
    let within = |value: i32, start: i32, length: u32| {
        (i64::from(start)..i64::from(start) + i64::from(length)).contains(&i64::from(value))
    };
    let monitor = monitors.iter().find(|m| {
        let p = m.position();
        let s = m.size();
        within(placement.x, p.x, s.width) && within(placement.y, p.y, s.height)
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
    // Moved before it is sized. A window that lands on a display with another scale factor is
    // rescaled by Windows to keep its logical size (WM_DPICHANGED), which would undo a size
    // set beforehand — and a laptop at 150% beside a projector at 100% is the usual room. The
    // rescale can shift the window as well, so it is placed again once it has its size.
    let position = PhysicalPosition::new(safe.x, safe.y);
    win.set_position(position)
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    win.set_size(PhysicalSize::new(safe.width, safe.height))
        .map_err(|e| AppError::with(id::OVERLAY_WINDOW, e))?;
    win.set_position(position)
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

    const LAPTOP: Display = Display {
        x: 0,
        y: 0,
        width: 2560,
        height: 1600,
        scale: 1.5,
    };
    const PROJECTOR: Display = Display {
        x: 2560,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
    };

    #[test]
    fn the_layout_signature_ignores_the_order_monitors_are_listed_in() {
        assert_eq!(
            layout_signature(&[LAPTOP, PROJECTOR]),
            layout_signature(&[PROJECTOR, LAPTOP])
        );
        assert_eq!(
            layout_signature(&[PROJECTOR, LAPTOP]),
            "0,0 2560x1600 150%; 2560,0 1920x1080 100%"
        );
    }

    #[test]
    fn the_layout_signature_changes_with_a_display_position_size_or_scale() {
        let base = layout_signature(&[LAPTOP, PROJECTOR]);
        let moved = Display {
            x: -1920,
            ..PROJECTOR
        };
        let resized = Display {
            width: 1280,
            height: 720,
            ..PROJECTOR
        };
        let rescaled = Display {
            scale: 1.25,
            ..PROJECTOR
        };
        let variants = [
            layout_signature(&[LAPTOP]),
            layout_signature(&[LAPTOP, moved]),
            layout_signature(&[LAPTOP, resized]),
            layout_signature(&[LAPTOP, rescaled]),
        ];
        for variant in &variants {
            assert_ne!(variant, &base);
        }
        // …and from each other: no two of these layouts share a remembered placement.
        let mut distinct = variants.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), variants.len());
    }

    #[test]
    fn a_scale_factor_a_rounding_error_away_names_the_same_layout() {
        let drifted = Display {
            scale: 120.0 / 96.0 + 1e-9,
            ..PROJECTOR
        };
        let exact = Display {
            scale: 1.25,
            ..PROJECTOR
        };
        assert_eq!(layout_signature(&[drifted]), layout_signature(&[exact]));
    }
}
