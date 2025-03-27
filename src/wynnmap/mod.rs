use core::panic;
use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use web_sys::{MouseEvent, TouchEvent, TouchList, WheelEvent};

pub mod conns;
pub mod maptile;
pub mod terrs;

const ZOOM_MIN: f64 = 0.0625;
const ZOOM_MAX: f64 = 64.0;

#[component]
pub fn WynnMap(children: Children) -> impl IntoView {
    // is the map being dragged currently
    let (dragging, set_dragging) = signal(false);
    // is the map being moved currently
    let (moving, set_moving) = signal(false);

    // position of the map
    let (position, set_pos) = signal((2000.0, 2200.0));

    // the current zoom level
    let (zoom, set_zoom) = signal(0.5);
    // transform used to compensate for zooming so that the zoom appears as if it's zooming into the mouse position
    let (zcomptrans, set_zcomptrans) = signal((0.0, 0.0));

    // are we currently transitioning? transitions can occur from zooming
    let (transitioning, set_transitioning) = signal(false);

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
                pos.0 + f64::from(e.movement_x()) / zoom.get(),
                pos.1 + f64::from(e.movement_y()) / zoom.get(),
            ));
        }

        let mut mpos = mousepos.lock().unwrap();
        *mpos = (e.client_x(), e.client_y());
    };

    // detect when a mouse drag starts
    let dragstart = move |e: MouseEvent| {
        e.prevent_default();
        set_dragging.set(true);
        set_moving.set(true);
    };

    // detect when a mouse drag ends
    let dragend = move |e: MouseEvent| {
        e.prevent_default();
        set_dragging.set(false);
        set_moving.set(false);
    };

    // detect zooming using a mouse wheel
    let zoomchange = move |e: WheelEvent| {
        e.prevent_default();

        // enable the transition when zooming with the mousewheel
        set_transitioning.set(true);

        // get the mouse position
        let mpos = mousepos2.lock().unwrap();
        let mpos = (f64::from(mpos.0), f64::from(mpos.1));

        // calculate the new zoom level
        let zoom = zoom.get();

        let newzoom = (zoom - (e.delta_y() / 300.0) * zoom).clamp(ZOOM_MIN, ZOOM_MAX);

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

        // iterate over the touches and store the positions
        let mut x = 0;
        while x < tl.length() {
            let touch = tl.get(x).unwrap();

            positions.push((touch.client_x(), touch.client_y()));

            x += 1;
        }

        positions
    };

    // detect when a touch starts and update the active touches
    let touchstart = move |e: TouchEvent| {
        e.prevent_default();

        let mut tpos = tpos.lock().unwrap();
        *tpos = updatetouchpos(&e.touches());

        if tpos.len() > 0 {
            set_moving.set(true);
        } else {
            set_moving.set(false);
        }
    };

    // handle the touch events for dragging and zooming
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
                // current position
                let pos = position.get();

                // new delta
                let touch = tl.get(0).unwrap();
                let npos = (touch.client_x(), touch.client_y());

                // update the position
                set_pos.set((
                    pos.0 + f64::from(npos.0 - tpos[0].0) / zoom.get(),
                    pos.1 + f64::from(npos.1 - tpos[0].1) / zoom.get(),
                ));
            }
            // zoom
            2 => {
                // disable will-change to prevent flickering
                set_moving.set(false);

                // get the touch positions
                let touch1 = tl.get(0).unwrap();
                let touch2 = tl.get(1).unwrap();

                let npos = (
                    (touch1.client_x(), touch1.client_y()),
                    (touch2.client_x(), touch2.client_y()),
                );

                // calculate the distance between the touches
                let dist =
                    f64::from((npos.0.0 - npos.1.0).pow(2) + (npos.0.1 - npos.1.1).pow(2)).sqrt();

                // calculate the distance between the touches before the zoom
                let opos =
                    f64::from((tpos[0].0 - tpos[1].0).pow(2) + (tpos[0].1 - tpos[1].1).pow(2))
                        .sqrt();

                // calculate the delta
                let delta = dist - opos;

                // calculate the new zoom level
                let zoom = zoom.get();
                let newzoom = (zoom + (delta / 300.0 * zoom)).clamp(ZOOM_MIN, ZOOM_MAX);

                set_zoom.set(newzoom);

                // calculate the zoom compensation transform
                // https://stackoverflow.com/a/27611642
                let mpos = (
                    f64::from(npos.0.0 + npos.1.0) / 2.0,
                    f64::from(npos.0.1 + npos.1.1) / 2.0,
                );

                let ctrans = zcomptrans.get();
                let i = ((mpos.0 - ctrans.0) / zoom, (mpos.1 - ctrans.1) / zoom);
                let n = (i.0 * newzoom, i.1 * newzoom);
                let c = (mpos.0 - n.0, mpos.1 - n.1);
                set_zcomptrans.set(c);
            }
            _ => {}
        }

        // update the touch positions after the event
        // this ensures that we can calculate deltas correctly
        *tpos = updatetouchpos(&tl);
    };

    view! {
        // outermost container used for containing the map
        <div
            class="wynnmap-container"
            style="height: 100vh;"
            on:mousemove=ondrag
            on:mousedown=dragstart
            on:mouseup=dragend
            on:mouseleave=dragend
            on:wheel=zoomchange
            on:touchstart=touchstart
            on:touchmove=ontouchdrag
        >
            // the inner container used for moving the map
            // this container contains the map contents and is moved when the map is dragged
            <div
                class="wynnmap-inner"
                class:wynnmap-zoomedin={move || zoom.get() > 1.0}
                class:wynnmap-transitions={move || transitioning.get()}
                // disable the transition after it has run
                on:transitionend=move |_| {set_transitioning.set(false);}

                style:will-change=move || {if moving.get() {"transform"} else {""}}
                style:transform=move || {
                    format!(
                        "matrix3d({z},0,0,0,0,{z},0,0,0,0,{z},0,{x},{y},0,1)",
                        x = position.get().0 * zoom.get() + zcomptrans.get().0,
                        y = position.get().1 * zoom.get() + zcomptrans.get().1,
                        z = zoom.get(),
                    )
                }
            >
                {children()}
            </div>
        </div>
    }
}
