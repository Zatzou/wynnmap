use leptos::prelude::*;

use crate::wynnmap::{ZOOM_MAX, ZOOM_MIN};

/// Calculate the new zoom level based on the current zoom level and the delta and clamp it to the min and max zoom levels
#[inline]
pub const fn calculate_new_zoom(current_zoom: f64, delta: f64) -> f64 {
    (delta
        .algebraic_mul(current_zoom)
        .algebraic_add(current_zoom))
    .clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Calculate the transform that has to be applied such that the zoom appears to be centered around the mouse position
///
/// This is based on the stackoverflow answer here: <https://stackoverflow.com/a/27611642>
#[inline]
pub fn calculate_zoom_compensation(center: [f64; 2], old_zoom: f64, new_zoom: f64) -> [f64; 2] {
    let i = center.map(|c| c.algebraic_div(old_zoom));

    let n = i.map(|i| i.algebraic_mul(new_zoom));

    zip_map(center, n, |c, n| c.algebraic_sub(n))
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
            p.algebraic_mul(new_zoom)
                .algebraic_add(zcomp.algebraic_mul(old_zoom))
                .algebraic_div(old_zoom)
        });
    });
}

/// Perform an zoom operation given the signals, zoom center point and a delta
pub fn apply_zoom(position: RwSignal<[f64; 2]>, zoom: RwSignal<f64>, center: [f64; 2], delta: f64) {
    let old_zoom = zoom.get();
    let new_zoom = calculate_new_zoom(old_zoom, delta);

    zoom.set(new_zoom);

    apply_zoom_compensation(center, old_zoom, new_zoom, position);
}

/// Helper function for getting the middle point of the viewport
pub fn get_viewport_middle() -> [f64; 2] {
    let window = web_sys::window().unwrap();

    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();

    [width / 2.0, height / 2.0]
}

/// Map over 2 arrays
#[inline]
pub fn zip_map<T1: Copy, T2: Copy, O, const N: usize>(
    lhs: [T1; N],
    rhs: [T2; N],
    mut f: impl FnMut(T1, T2) -> O,
) -> [O; N] {
    std::array::from_fn(|i| f(lhs[i], rhs[i]))
}
