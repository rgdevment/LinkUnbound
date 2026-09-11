const OFFSET: i32 = 12;

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Physical coordinates: on a mixed-DPI desktop, logical ones land the window
/// on the wrong screen.
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
