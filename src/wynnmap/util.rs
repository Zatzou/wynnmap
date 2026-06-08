use leptos::prelude::*;

use crate::wynnmap::{ZOOM_MAX, ZOOM_MIN};

/// Calculate the new zoom level based on the current zoom level and the delta and clamp it to the min and max zoom levels
pub const fn calculate_new_zoom(current_zoom: f64, delta: f64) -> f64 {
    (delta.mul_add(current_zoom, current_zoom)).clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Calculate the transform that has to be applied such that the zoom appears to be centered around the mouse position
///
/// This is based on the stackoverflow answer here: <https://stackoverflow.com/a/27611642>
pub fn calculate_zoom_compensation(center: [f64; 2], old_zoom: f64, new_zoom: f64) -> [f64; 2] {
    let i = center.map(|c| c / old_zoom);

    let n = i.map(|i| i * new_zoom);

    zip_map(center, n, |c, n| c - n)
}

/// Helper function to apply the zoom compensation to the current position
pub fn apply_zoom_compensation(
    center: [f64; 2],
    old_zoom: f64,
    new_zoom: f64,
    pos: RwSignal<[f64; 2]>,
) {
    let zcomp = calculate_zoom_compensation(center, old_zoom, new_zoom);

    pos.update(|p| {
        *p = zip_map(*p, zcomp, |p, zcomp| {
            p.mul_add(new_zoom, zcomp * old_zoom) / old_zoom
        });
    });
}

/// Helper function for getting the middle point of the viewport
pub fn get_viewport_middle() -> [f64; 2] {
    let window = web_sys::window().unwrap();

    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();

    [width / 2.0, height / 2.0]
}

/// Map over 2 arrays
pub fn zip_map<T: Copy, O: Default + Copy, const N: usize>(
    lhs: [T; N],
    rhs: [T; N],
    mut f: impl FnMut(T, T) -> O,
) -> [O; N] {
    let mut out = [Default::default(); N];

    for i in 0..N {
        out[i] = f(lhs[i], rhs[i]);
    }

    out
}
