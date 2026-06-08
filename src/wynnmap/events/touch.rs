use leptos::prelude::*;
use web_sys::{TouchEvent, TouchList};

use crate::wynnmap::util::{apply_zoom_compensation, calculate_new_zoom};

pub struct TouchEventHandlers<TS, TM>
where
    TS: Fn(TouchEvent) + Copy + 'static,
    TM: Fn(TouchEvent) + Copy + 'static,
{
    pub touchstart: TS,
    pub touchmove: TM,
}

pub fn handlers(
    position: RwSignal<(f64, f64)>,
    zoom: RwSignal<f64>,
    moving: RwSignal<bool>,
) -> TouchEventHandlers<impl Fn(TouchEvent) + Copy + 'static, impl Fn(TouchEvent) + Copy + 'static>
{
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

    TouchEventHandlers {
        touchstart,
        touchmove,
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
