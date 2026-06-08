use leptos::prelude::*;
use web_sys::{MouseEvent, WheelEvent};

use crate::wynnmap::util::{apply_zoom_compensation, calculate_new_zoom};

pub struct MouseEventHandlers<MM, SM, EM, WH>
where
    MM: Fn(MouseEvent) + Copy + 'static,
    SM: Fn(MouseEvent) + Copy + 'static,
    EM: Fn(MouseEvent) + Copy + 'static,
    WH: Fn(WheelEvent) + Copy + 'static,
{
    pub mousemove: MM,
    pub start_mousemove: SM,
    pub end_mousemove: EM,
    pub wheel: WH,
}

pub fn handlers(
    position: RwSignal<[f64; 2]>,
    zoom: RwSignal<f64>,
    moving: RwSignal<bool>,
    transitioning: RwSignal<bool>,
) -> MouseEventHandlers<
    impl Fn(MouseEvent) + Copy + 'static,
    impl Fn(MouseEvent) + Copy + 'static,
    impl Fn(MouseEvent) + Copy + 'static,
    impl Fn(WheelEvent) + Copy + 'static,
> {
    // mouse position stored for zoom compensation
    let mousepos = RwSignal::new([0, 0]);

    let mousemove = move |e: MouseEvent| {
        e.prevent_default();

        // if we are dragging move the map
        if moving.get() {
            position.update(|[x, y]| {
                *x += f64::from(e.movement_x());
                *y += f64::from(e.movement_y());
            });
        }

        mousepos.set([e.client_x(), e.client_y()]);
    };

    // detect when a mouse drag starts
    let start_mousemove = move |e: MouseEvent| {
        e.prevent_default();

        moving.set(true);
    };

    // detect when a mouse drag ends
    let end_mousemove = move |e: MouseEvent| {
        e.prevent_default();

        moving.set(false);
    };

    // detect zooming using a mouse wheel
    let wheel = move |e: WheelEvent| {
        e.prevent_default();

        // enable the transition when zooming with the mousewheel
        transitioning.set(true);

        // get the mouse position
        let mpos = mousepos.get().map(f64::from);

        // calculate the new zoom level
        let old_zoom = zoom.get();
        let new_zoom = calculate_new_zoom(old_zoom, -e.delta_y() / 300.0);

        zoom.set(new_zoom);

        apply_zoom_compensation(mpos, old_zoom, new_zoom, position);
    };

    MouseEventHandlers {
        mousemove,
        start_mousemove,
        end_mousemove,
        wheel,
    }
}
