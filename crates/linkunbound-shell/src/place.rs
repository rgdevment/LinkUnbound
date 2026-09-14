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

    /// The flipped side is the mirror of the offset one, not merely somewhere that fits: the
    /// window has to hang off the pointer by the same gap on whichever side it opens.
    #[test]
    fn near_the_right_edge_it_flips_to_the_other_side() {
        assert_eq!(beside((1900, 300), WINDOW, SCREEN), (1524, 312));
    }

    #[test]
    fn near_the_bottom_it_opens_upwards() {
        assert_eq!(beside((400, 1070), WINDOW, SCREEN), (412, 758));
    }

    /// Fitting exactly is still fitting. Flipping here would send the window to the far side of
    /// the pointer every time it landed on the edge, which is where a maximised window puts it.
    #[test]
    fn a_window_that_ends_flush_with_the_edge_does_not_flip() {
        assert_eq!(beside((1544, 300), WINDOW, SCREEN), (1556, 312));
        assert_eq!(beside((400, 768), WINDOW, SCREEN), (412, 780));
    }

    /// The taskbar is outside the work area and the pointer can be on top of it, which puts the
    /// flipped window past the last row that is still visible. The clamp is what pulls it back,
    /// and without it the picker opens underneath the taskbar.
    #[test]
    fn the_pointer_on_the_taskbar_does_not_push_the_window_under_it() {
        let side_bar = Bounds {
            x: 0,
            y: 0,
            width: 1860,
            height: 1080,
        };
        assert_eq!(beside((1910, 300), WINDOW, side_bar), (1496, 312));

        let bottom_bar = Bounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1040,
        };
        assert_eq!(beside((400, 1075), WINDOW, bottom_bar), (412, 740));
    }

    #[test]
    fn a_corner_is_handled_on_both_axes_at_once() {
        assert_eq!(beside((1918, 1078), WINDOW, SCREEN), (1542, 766));
    }

    #[test]
    fn a_monitor_left_of_the_primary_keeps_its_own_coordinates() {
        let left = Bounds {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        assert_eq!(beside((-1000, 500), WINDOW, left), (-988, 512));
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
