use core::panic;
use std::sync::{Arc, Mutex};

use leptos::{ev, leptos_dom::{logging::*, helpers}, prelude::*};
use web_sys::{KeyboardEvent, MouseEvent, TouchEvent, TouchList, WheelEvent};

pub mod conns;
pub mod maptile;
pub mod terrs;

#[component]
pub fn WynnMap(children: Children) -> impl IntoView {
    // grab user agent
    let user_agent = helpers::window().navigator().user_agent()
        .unwrap_or_default();
    // test if Gecko engine. If it contains `like Gecko` then its probably not Gecko. 
    let is_gecko = user_agent.contains("Gecko/") && !user_agent.contains("like Gecko");
    // print to console if detecting is gecko
    if is_gecko {
        console_log("You seem to be using Gecko. Some optimizations have been applied to try and fix a certain bug. If you're spoofing your user agent and you are currently using a Chromium-based browser, you should stop, or this thing breaks to bits.")
    }


    // is the map being dragged currently
    let (dragging, set_dragging) = signal(false);
    // is the map being moved currently
    let (moving, set_moving) = signal(false);

    // position of the map
    let screen_middle = get_viewport_middle();
    // use the midpoint to position the map so that it is centered
    let position = RwSignal::new((100.0 + screen_middle.0, 1200.0 + screen_middle.1));

    // the current zoom level
    let (zoom, set_zoom) = signal(0.5);

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

            position.set((
                pos.0 + f64::from(e.movement_x()),
                pos.1 + f64::from(e.movement_y()),
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

        let newzoom = calculate_new_zoom(zoom, -e.delta_y() / 300.0);

        set_zoom.set(newzoom);

        apply_zoom_compensation(mpos, zoom, newzoom, position);
    };

    // touch positions stored for touch events
    let tpos = Arc::new(Mutex::new(Vec::new()));
    let tpos2 = tpos.clone();

    // detect when a touch starts and update the active touches
    let touchstart = move |e: TouchEvent| {
        e.prevent_default();

        let mut tpos = tpos.lock().unwrap();
        *tpos = get_touch_positions(&e.touches());

        if tpos.is_empty() {
            set_moving.set(false);
        } else {
            set_moving.set(true);
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
            *tpos = get_touch_positions(&tl);
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
                position.set((
                    pos.0 + f64::from(npos.0 - tpos[0].0),
                    pos.1 + f64::from(npos.1 - tpos[0].1),
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
                let newzoom = calculate_new_zoom(zoom, delta / 300.0);

                set_zoom.set(newzoom);

                let mpos = (
                    f64::from(npos.0.0 + npos.1.0) / 2.0,
                    f64::from(npos.0.1 + npos.1.1) / 2.0,
                );

                apply_zoom_compensation(mpos, zoom, newzoom, position);
            }
            _ => {}
        }

        // update the touch positions after the event
        // this ensures that we can calculate deltas correctly
        *tpos = get_touch_positions(&tl);
    };

    let onkeydown = move |e: KeyboardEvent| {
        match e.key().as_str() {
            // 0 key - reset zoom
            "0" => {
                // perform zoom compensation
                // get middle point of the screen
                let mpos = get_viewport_middle();
                // calculate the zoom compensation
                let zcomp = calculate_zoom_compensation(mpos, zoom.get(), 1.0);
                // apply the zoom compensation
                position.set((position.get().0 + zcomp.0, position.get().1 + zcomp.1));

                // reset the zoom
                set_zoom.set(1.0);

                // do transition
                set_transitioning.set(true);
            }
            // plus key - zoom in
            "+" => {
                let oldzoom = zoom.get();

                // calculate the new zoom level
                let newzoom = calculate_new_zoom(oldzoom, 0.3);
                set_zoom.set(newzoom);

                // get middle point of the screen
                let mpos = get_viewport_middle();
                // apply the zoom compensation
                apply_zoom_compensation(mpos, oldzoom, newzoom, position);

                // do transition
                set_transitioning.set(true);
            }
            // minus key - zoom out
            "-" => {
                let oldzoom = zoom.get();

                // calculate the new zoom level
                let newzoom = calculate_new_zoom(oldzoom, -0.3);
                set_zoom.set(newzoom);

                // get middle point of the screen
                let mpos = get_viewport_middle();
                // apply the zoom compensation
                apply_zoom_compensation(mpos, oldzoom, newzoom, position);

                // do transition
                set_transitioning.set(true);
            }
            // ArrowUp - move up
            "ArrowUp" => {
                position.update(|p| {
                    *p = (p.0, p.1 + 100.0 / zoom.get());
                });

                set_transitioning.set(true);
            }
            // ArrowDown - move down
            "ArrowDown" => {
                position.update(|p| {
                    *p = (p.0, p.1 - 100.0 / zoom.get());
                });

                set_transitioning.set(true);
            }
            // ArrowLeft - move left
            "ArrowLeft" => {
                position.update(|p| {
                    *p = (p.0 + 100.0 / zoom.get(), p.1);
                });

                set_transitioning.set(true);
            }
            // ArrowRight - move right
            "ArrowRight" => {
                position.update(|p| {
                    *p = (p.0 - 100.0 / zoom.get(), p.1);
                });

                set_transitioning.set(true);
            }
            // do nothing on unknown keys
            _ => {}
        }
    };

    window_event_listener(ev::keydown, onkeydown);

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
                class:wynnmap-zoomedout={move || zoom.get() < 0.3}
                class:wynnmap-transitions={move || transitioning.get()}
                // disable the transition after it has run
                on:transitionend=move |_| {set_transitioning.set(false);}

                // will-change:transform if using gecko (according to user agent) or you're currently holding down (moving.get())
                style:will-change=move || {if is_gecko || moving.get() {"transform"} else {""}}
                style:transform=move || {
                    format!(
                        "matrix3d({z},0,0,0,0,{z},0,0,0,0,1,0,{x},{y},0,1)",
                        x = position.get().0,
                        y = position.get().1,
                        z = zoom.get(),
                    )
                }
            >
                {children()}
            </div>
        </div>
    }
}

/// Convinience function for getting the touch positions out of a DOM [`TouchList`]
fn get_touch_positions(tl: &TouchList) -> Vec<(i32, i32)> {
    let mut positions = Vec::new();

    // iterate over the touches and store the positions
    let mut x = 0;
    while x < tl.length() {
        let touch = tl.get(x).unwrap();

        positions.push((touch.client_x(), touch.client_y()));

        x += 1;
    }

    positions
}

/// The minimum zoom level
const ZOOM_MIN: f64 = 0.0625;
/// The maximum zoom level
const ZOOM_MAX: f64 = 64.0;

/// Calculate the new zoom level based on the current zoom level and the delta and clamp it to the min and max zoom levels
fn calculate_new_zoom(current_zoom: f64, delta: f64) -> f64 {
    (delta.mul_add(current_zoom, current_zoom)).clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Calculate the transform that has to be applied such that the zoom appears to be centered around the mouse position
///
/// This is based on the stackoverflow answer here: <https://stackoverflow.com/a/27611642>
fn calculate_zoom_compensation(
    position: (f64, f64),
    current_zoom: f64,
    new_zoom: f64,
) -> (f64, f64) {
    let i = (position.0 / current_zoom, position.1 / current_zoom);

    let n = (i.0 * new_zoom, i.1 * new_zoom);

    (position.0 - n.0, position.1 - n.1)
}

/// Helper function to apply the zoom compensation to the current position
fn apply_zoom_compensation(
    center: (f64, f64),
    old_zoom: f64,
    new_zoom: f64,
    pos: RwSignal<(f64, f64)>,
) {
    let zcomp = calculate_zoom_compensation(center, old_zoom, new_zoom);

    pos.update(|p| {
        *p = (
            p.0.mul_add(new_zoom, zcomp.0 * old_zoom) / old_zoom,
            p.1.mul_add(new_zoom, zcomp.1 * old_zoom) / old_zoom,
        );
    });
}

/// Helper function for getting the middle point of the viewport
fn get_viewport_middle() -> (f64, f64) {
    let window = web_sys::window().unwrap();

    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();

    (width / 2.0, height / 2.0)
}
