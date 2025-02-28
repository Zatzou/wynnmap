use core::panic;
use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use web_sys::{MouseEvent, TouchEvent, TouchList, WheelEvent};

#[component]
pub fn ShitMap(children: Children) -> impl IntoView {
    // is the map being dragged currently
    let (dragging, set_dragging) = signal(false);

    // position of the map
    let (position, set_pos) = signal((2000.0, 2200.0));

    // the current zoom level
    let (zoom, set_zoom) = signal(0.5);
    // transform used to compensate for zooming so that the zoom appears as if it's zooming into the mouse position
    let (zcomptrans, set_zcomptrans) = signal((0.0, 0.0));
    // are we using touch to zoom? this is used to disable the transition so zooming is not laggy on touch devices
    let (touchzoom, set_touchzoom) = signal(false);

    // mouse position stored for zoom compensation
    let mousepos = Arc::new(Mutex::new((0, 0)));
    let mousepos2 = mousepos.clone();

    // mouse position and drag events
    let ondrag = move |e: MouseEvent| {
        e.prevent_default();

        // if we are dragging move the map
        if dragging.get() {
            let pos = position.get();

            set_pos.set((
                pos.0 + e.movement_x() as f64 / zoom.get(),
                pos.1 + e.movement_y() as f64 / zoom.get(),
            ));
        }

        let mut mpos = mousepos.lock().unwrap();
        *mpos = (e.client_x(), e.client_y());
    };

    // detect when a mouse drag starts
    let dragstart = move |e: MouseEvent| {
        e.prevent_default();
        set_dragging.set(true);
    };

    // detect when a mouse drag ends
    let dragend = move |e: MouseEvent| {
        e.prevent_default();
        set_dragging.set(false);
    };

    // detect zooming using a mouse wheel
    let zoomchange = move |e: WheelEvent| {
        e.prevent_default();

        set_touchzoom.set(false);

        let mpos = mousepos2.lock().unwrap();
        let mpos = (mpos.0 as f64, mpos.1 as f64);
        let zoom = zoom.get();

        let newzoom = if e.delta_y() > 0.0 {
            zoom / 2.0
        } else {
            zoom * 2.0
        };

        set_zoom.set(newzoom);

        // calculate the zoom compensation transform
        // https://stackoverflow.com/a/27611642
        let ctrans = zcomptrans.get();
        let i = ((mpos.0 - ctrans.0) / zoom, (mpos.1 - ctrans.1) / zoom);
        let n = (i.0 * newzoom, i.1 * newzoom);
        let c = (mpos.0 - n.0, mpos.1 - n.1);
        set_zcomptrans.set(c);
    };

    // touch positions stored for touch events
    let tpos = Arc::new(Mutex::new(Vec::new()));
    let tpos2 = tpos.clone();

    // function for updating touch positions
    let updatetouchpos = |tl: &TouchList| -> Vec<(i32, i32)> {
        let mut positions = Vec::new();

        let mut x = 0;
        while x < tl.length() {
            let touch = tl.get(x).unwrap();
            positions.push((touch.client_x(), touch.client_y()));
            x += 1;
        }

        positions
    };

    // detect when a touch starts
    let touchstart = move |e: TouchEvent| {
        e.prevent_default();

        *tpos.lock().unwrap() = updatetouchpos(&e.touches());
    };

    let ontouchdrag = move |e: TouchEvent| {
        e.prevent_default();

        // get the touch positions
        let tl = e.touches();
        let mut tpos = tpos2.lock().unwrap();

        // if the touch positions are different from the stored touch positions update the stored touch positions
        if tl.length() as usize != tpos.len() {
            *tpos = updatetouchpos(&tl);
            return;
        }

        // match the number of touches to determine if it's a drag or zoom
        match tpos.len() {
            // drag
            1 => {
                let pos = position.get();
                let touch = tl.get(0).unwrap();
                let npos = (touch.client_x(), touch.client_y());

                set_pos.set((
                    pos.0 + ((npos.0 - tpos[0].0) as f64) / zoom.get(),
                    pos.1 + ((npos.1 - tpos[0].1) as f64) / zoom.get(),
                ));

                *tpos = updatetouchpos(&tl);
            }
            // zoom
            2 => {
                set_touchzoom.set(true);

                let touch1 = tl.get(0).unwrap();
                let touch2 = tl.get(1).unwrap();

                let npos = (
                    (touch1.client_x(), touch1.client_y()),
                    (touch2.client_x(), touch2.client_y()),
                );

                let dist =
                    (((npos.0.0 - npos.1.0).pow(2) + (npos.0.1 - npos.1.1).pow(2)) as f64).sqrt();

                let opos = (((tpos[0].0 - tpos[1].0).pow(2) + (tpos[0].1 - tpos[1].1).pow(2))
                    as f64)
                    .sqrt();

                let delta = dist - opos;

                let zoom = zoom.get();
                let newzoom = (zoom + delta / 200.0).max(0.1);

                set_zoom.set(newzoom);

                let mpos = (
                    (npos.0.0 + npos.1.0) as f64 / 2.0,
                    (npos.0.1 + npos.1.1) as f64 / 2.0,
                );

                let ctrans = zcomptrans.get();
                let i = ((mpos.0 - ctrans.0) / zoom, (mpos.1 - ctrans.1) / zoom);
                let n = (i.0 * newzoom, i.1 * newzoom);
                let c = (mpos.0 - n.0, mpos.1 - n.1);
                set_zcomptrans.set(c);

                *tpos = updatetouchpos(&tl);
            }
            _ => {}
        }
    };

    view! {
        <div class="bg-neutral-950 shitmap-container" style="height: 100vh;" on:mousemove={ondrag} on:mousedown={dragstart} on:mouseup={dragend} on:mouseleave={dragend} on:wheel={zoomchange} on:touchstart={touchstart} on:touchmove={ontouchdrag}>
            <div class="shitmap-zoomer" class:shitmap-zoomer-transitions={move || !touchzoom.get()} style={move || format!("transform: translate3D({}px, {}px, 0) scale({});", zcomptrans.get().0, zcomptrans.get().1, zoom.get())}>
                <div class="shitmap-inner" class:shitmap-zoomedin={move || zoom.get() > 1.0} style={move || format!("transform: translate({}px, {}px);", position.get().0, position.get().1)}>
                    {children()}
                </div>
            </div>
        </div>
    }
}
