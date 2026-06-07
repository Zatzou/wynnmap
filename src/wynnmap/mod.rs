use leptos::{ev, prelude::*};
use web_sys::{KeyboardEvent, MouseEvent, PointerEvent, TouchEvent, TouchList, WheelEvent};

pub mod conns;
pub mod maptile;
pub mod terrs;

/// The minimum zoom level
const ZOOM_MIN: f64 = 0.0625;
/// The maximum zoom level
const ZOOM_MAX: f64 = 64.0;

/// Mouse position on the map atlas
#[derive(Clone)]
pub struct RelMousePos(pub RwSignal<(i32, i32)>);

#[component]
pub fn WynnMap(
    children: Children,
    #[prop(optional)] onclick: Option<Callback<(i32, i32)>>,
) -> impl IntoView {
    // is the map being moved currently
    let moving = RwSignal::new(false);

    // position of the map
    let screen_middle = get_viewport_middle();
    // use the midpoint to position the map so that it is centered
    let position = RwSignal::new((100.0 + screen_middle.0, 1200.0 + screen_middle.1));

    // the current zoom level
    let zoom = RwSignal::new(0.5);

    // are we currently transitioning? transitions can occur from zooming
    let transitioning = RwSignal::new(false);

    // mouse position stored for zoom compensation
    let mousepos = RwSignal::new((0, 0));

    // mouse position and drag events
    let mousemove = move |e: MouseEvent| {
        e.prevent_default();

        // if we are dragging move the map
        if moving.get() {
            let pos = position.get();

            position.set((
                pos.0 + f64::from(e.movement_x()),
                pos.1 + f64::from(e.movement_y()),
            ));
        }

        mousepos.set((e.client_x(), e.client_y()));
    };

    // detect when a mouse drag starts
    let mousedown = move |e: MouseEvent| {
        e.prevent_default();
        moving.set(true);
    };

    // detect when a mouse drag ends
    let mouseup = move |e: MouseEvent| {
        e.prevent_default();
        moving.set(false);
    };

    // detect zooming using a mouse wheel
    let wheel = move |e: WheelEvent| {
        e.prevent_default();

        // enable the transition when zooming with the mousewheel
        transitioning.set(true);

        // get the mouse position
        let mpos1 = mousepos.get();
        let mpos = (f64::from(mpos1.0), f64::from(mpos1.1));

        // calculate the new zoom level
        let old_zoom = zoom.get();
        let new_zoom = calculate_new_zoom(old_zoom, -e.delta_y() / 300.0);

        zoom.set(new_zoom);

        apply_zoom_compensation(mpos, old_zoom, new_zoom, position);
    };

    // touch positions stored for touch events
    let tpos = RwSignal::new(Vec::new());

    // detect when a touch starts and update the active touches
    let touchstart = move |e: TouchEvent| {
        e.prevent_default();

        tpos.set(get_touch_positions(&e.touches()));

        if tpos.read().is_empty() {
            moving.set(false);
        } else {
            moving.set(true);
        }
    };

    // handle the touch events for dragging and zooming
    let touchmove = move |e: TouchEvent| {
        e.prevent_default();

        // get the touch positions
        let tl = e.touches();

        // if the touch positions are different from the stored touch positions update the stored touch positions
        if tl.length() as usize != tpos.read().len() {
            tpos.set(get_touch_positions(&tl));
            return;
        }

        // match the number of touches to determine if it's a drag or zoom
        match tpos.read().len() {
            // drag
            1 => {
                // current position
                let pos = position.get();

                // new delta
                let touch = tl.get(0).unwrap();
                let npos = (touch.client_x(), touch.client_y());

                let tpos = tpos.read();

                // update the position
                position.set((
                    pos.0 + f64::from(npos.0 - tpos[0].0),
                    pos.1 + f64::from(npos.1 - tpos[0].1),
                ));
            }
            // zoom
            2 => {
                // disable will-change to prevent flickering
                moving.set(false);

                // get the touch positions
                let touch1 = tl.get(0).unwrap();
                let touch2 = tl.get(1).unwrap();

                let npos = (
                    (touch1.client_x(), touch1.client_y()),
                    (touch2.client_x(), touch2.client_y()),
                );

                let tpos = tpos.read();

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
                let old_zoom = zoom.get();
                let new_zoom = calculate_new_zoom(old_zoom, delta / 300.0);

                zoom.set(new_zoom);

                let mpos = (
                    f64::from(npos.0.0 + npos.1.0) / 2.0,
                    f64::from(npos.0.1 + npos.1.1) / 2.0,
                );

                apply_zoom_compensation(mpos, old_zoom, new_zoom, position);
            }
            _ => {}
        }

        // update the touch positions after the event
        // this ensures that we can calculate deltas correctly
        tpos.set(get_touch_positions(&tl));
    };

    let onkeydown = move |e: KeyboardEvent| {
        match e.key().as_str() {
            // 0 key - reset zoom
            "0" => {
                let oldzoom = zoom.get();
                // reset the zoom
                zoom.set(0.5);

                // perform zoom compensation
                // get middle point of the screen
                let mpos = get_viewport_middle();
                // calculate the zoom compensation
                apply_zoom_compensation(mpos, oldzoom, 0.5, position);

                // do transition
                transitioning.set(true);
            }
            // Home - reset position
            "Home" => {
                let screen_middle = get_viewport_middle();
                let zoom = zoom.get() * 2.0;

                position.set((
                    100.0f64.mul_add(zoom, screen_middle.0),
                    1200.0f64.mul_add(zoom, screen_middle.1),
                ));

                transitioning.set(true);
            }
            // plus key - zoom in
            "+" => {
                let oldzoom = zoom.get();

                // calculate the new zoom level
                let newzoom = calculate_new_zoom(oldzoom, 0.3);
                zoom.set(newzoom);

                // get middle point of the screen
                let mpos = get_viewport_middle();
                // apply the zoom compensation
                apply_zoom_compensation(mpos, oldzoom, newzoom, position);

                // do transition
                transitioning.set(true);
            }
            // minus key - zoom out
            "-" => {
                let oldzoom = zoom.get();

                // calculate the new zoom level
                let newzoom = calculate_new_zoom(oldzoom, -0.3);
                zoom.set(newzoom);

                // get middle point of the screen
                let mpos = get_viewport_middle();
                // apply the zoom compensation
                apply_zoom_compensation(mpos, oldzoom, newzoom, position);

                // do transition
                transitioning.set(true);
            }
            // ArrowUp - move up
            "ArrowUp" => {
                position.update(|p| {
                    *p = (p.0, p.1 + 100.0 / zoom.get());
                });

                transitioning.set(true);
            }
            // ArrowDown - move down
            "ArrowDown" => {
                position.update(|p| {
                    *p = (p.0, p.1 - 100.0 / zoom.get());
                });

                transitioning.set(true);
            }
            // ArrowLeft - move left
            "ArrowLeft" => {
                position.update(|p| {
                    *p = (p.0 + 100.0 / zoom.get(), p.1);
                });

                transitioning.set(true);
            }
            // ArrowRight - move right
            "ArrowRight" => {
                position.update(|p| {
                    *p = (p.0 - 100.0 / zoom.get(), p.1);
                });

                transitioning.set(true);
            }
            // do nothing on unknown keys
            _ => {}
        }
    };

    window_event_listener(ev::keydown, onkeydown);

    // use pointer events to handle clicks on the map
    let relmousepos = RwSignal::new((0, 0));
    provide_context(RelMousePos(relmousepos));

    let pointermove = move |e: PointerEvent| {
        if !moving.get() {
            let pos = (e.client_x(), e.client_y());

            // calculate the compensation
            let zoom = zoom.get();
            let map_pos = position.get();
            let rel = (
                (pos.0 as f64 - map_pos.0) / zoom,
                (pos.1 as f64 - map_pos.1) / zoom,
            );
            relmousepos.set((rel.0 as i32, rel.1 as i32));
        }
    };

    let dragstartpos = RwSignal::new((0, 0));

    let pointerdown = move |e: PointerEvent| {
        dragstartpos.set((e.client_x(), e.client_y()));

        // also update position on down to support touch events
        pointermove(e);
    };

    let pointerup = move |e: PointerEvent| {
        let pos = (e.client_x(), e.client_y());
        let startpos = dragstartpos.get();

        // emit clicks when the map hasnt been moved meaningfully
        if let Some(cb) = onclick
            && startpos.0.abs_diff(pos.0) < 5
            && startpos.1.abs_diff(pos.1) < 5
        {
            cb.run(relmousepos.get())
        }
    };

    view! {
        // outermost container used for containing the map
        <div
            class="wynnmap-container"
            on:mousemove=mousemove
            on:mousedown=mousedown
            on:mouseup=mouseup
            on:mouseleave=mouseup
            on:wheel=wheel
            on:touchstart=touchstart
            on:touchmove=touchmove
            on:pointermove=pointermove
            on:pointerdown=pointerdown
            on:pointerup=pointerup
        >
            // the inner container used for moving the map
            // this container contains the map contents and is moved when the map is dragged
            <div
                class="wynnmap-inner"
                class:wynnmap-zoomedin={move || zoom.get() > 1.0}
                class:wynnmap-zoomedout={move || zoom.get() < 0.3}
                class:wynnmap-transitions=move || transitioning.get() && !moving.get()
                // disable the transition after it has run
                on:transitionend=move |_| transitioning.set(false)

                // will-change:transform if using gecko (according to user agent) or you're currently holding down (moving.get())
                style:will-change=move || moving.get().then_some("transform")

                style:transform=move ||
                    format!("matrix3d({z},0,0,0,0,{z},0,0,0,0,1,0,{x},{y},0,1)",
                        z = zoom.read(),
                        x = position.read().0,
                        y = position.read().1
                    )
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

/// Calculate the new zoom level based on the current zoom level and the delta and clamp it to the min and max zoom levels
const fn calculate_new_zoom(current_zoom: f64, delta: f64) -> f64 {
    (delta.mul_add(current_zoom, current_zoom)).clamp(ZOOM_MIN, ZOOM_MAX)
}

/// Calculate the transform that has to be applied such that the zoom appears to be centered around the mouse position
///
/// This is based on the stackoverflow answer here: <https://stackoverflow.com/a/27611642>
const fn calculate_zoom_compensation(
    center: (f64, f64),
    old_zoom: f64,
    new_zoom: f64,
) -> (f64, f64) {
    let i = (center.0 / old_zoom, center.1 / old_zoom);

    let n = (i.0 * new_zoom, i.1 * new_zoom);

    (center.0 - n.0, center.1 - n.1)
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
