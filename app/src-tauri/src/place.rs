use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

/// Enough that the picker does not open under the pointer, small enough that it
/// still reads as belonging to the click.
const OFFSET: i32 = 12;

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Bounds {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

/// Opens down-right of the pointer, flipping to the other side when that edge is
/// too close, and never leaves the monitor the pointer is on. Coordinates are
/// physical: a mixed-DPI desktop reports each screen in its own scale, and
/// working in logical pixels puts the window on the wrong monitor.
#[must_use]
pub fn beside(cursor: (i32, i32), window: (i32, i32), screen: Bounds) -> (i32, i32) {
    let (cx, cy) = cursor;
    let (w, h) = window;

    let mut x = cx + OFFSET;
    if x + w > screen.x + screen.width {
        x = cx - w - OFFSET;
    }
    let mut y = cy + OFFSET;
    if y + h > screen.y + screen.height {
        y = cy - h - OFFSET;
    }

    let max_x = (screen.x + screen.width - w).max(screen.x);
    let max_y = (screen.y + screen.height - h).max(screen.y);
    (x.clamp(screen.x, max_x), y.clamp(screen.y, max_y))
}

fn screen_under(window: &WebviewWindow, cursor: (i32, i32)) -> Option<Bounds> {
    let monitors = window.available_monitors().ok()?;
    let bounds = |m: &tauri::Monitor| {
        let p: &PhysicalPosition<i32> = m.position();
        let s: &PhysicalSize<u32> = m.size();
        Bounds {
            x: p.x,
            y: p.y,
            width: s.width as i32,
            height: s.height as i32,
        }
    };
    monitors
        .iter()
        .map(bounds)
        .find(|b| b.contains(cursor.0, cursor.1))
        .or_else(|| window.primary_monitor().ok().flatten().map(|m| bounds(&m)))
}

pub fn at_cursor(window: &WebviewWindow) {
    let Ok(cursor) = window.cursor_position() else {
        return;
    };
    let cursor = (cursor.x as i32, cursor.y as i32);
    let Ok(size) = window.outer_size() else {
        return;
    };
    let Some(screen) = screen_under(window, cursor) else {
        return;
    };

    let (x, y) = beside(cursor, (size.width as i32, size.height as i32), screen);
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Bounds = Bounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    const WINDOW: (i32, i32) = (364, 300);

    #[test]
    fn it_opens_just_off_the_pointer() {
        assert_eq!(beside((400, 300), WINDOW, SCREEN), (412, 312));
    }

    #[test]
    fn near_the_right_edge_it_flips_to_the_other_side() {
        let (x, _) = beside((1900, 300), WINDOW, SCREEN);
        assert!(x + WINDOW.0 <= SCREEN.width);
        assert!(x < 1900);
    }

    #[test]
    fn near_the_bottom_it_opens_upwards() {
        let (_, y) = beside((400, 1070), WINDOW, SCREEN);
        assert!(y + WINDOW.1 <= SCREEN.height);
        assert!(y < 1070);
    }

    #[test]
    fn a_corner_is_handled_on_both_axes_at_once() {
        let (x, y) = beside((1918, 1078), WINDOW, SCREEN);
        assert!(x >= 0 && x + WINDOW.0 <= SCREEN.width);
        assert!(y >= 0 && y + WINDOW.1 <= SCREEN.height);
    }

    /// A second screen to the left reports negative coordinates, and clamping to
    /// zero would drag the picker onto the primary monitor.
    #[test]
    fn a_monitor_left_of_the_primary_keeps_its_own_coordinates() {
        let left = Bounds {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let (x, y) = beside((-1000, 500), WINDOW, left);
        assert!(x < 0);
        assert!(x >= left.x);
        assert_eq!(y, 512);
    }

    #[test]
    fn a_window_larger_than_the_screen_still_lands_inside_it() {
        let tiny = Bounds {
            x: 0,
            y: 0,
            width: 300,
            height: 200,
        };
        let (x, y) = beside((150, 100), WINDOW, tiny);
        assert_eq!((x, y), (0, 0));
    }
}
