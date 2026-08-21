use leptos::prelude::*;
use web_sys::{Touch, TouchEvent, TouchList};

use crate::wynnmap::util::{apply_zoom, zip_map};

pub struct TouchEventHandlers<TS, TM>
where
    TS: Fn(TouchEvent) + Copy + 'static,
    TM: Fn(TouchEvent) + Copy + 'static,
{
    pub touchstart: TS,
    pub touchmove: TM,
}

pub fn handlers(
    position: RwSignal<[f64; 2]>,
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
        match tpos.read()[..] {
            // drag
            [old_pos] => {
                // new delta
                let touch = tl.get(0).unwrap();
                let new_pos = touch_pos(&touch);

                let delta = zip_map(new_pos, old_pos, |n, o| n - o).map(f64::from);

                position.update(|[x, y]| {
                    *x += delta[0];
                    *y += delta[1];
                });
            }
            // zoom
            [old1, old2] => {
                // disable will-change to prevent flickering
                moving.set(false);

                // get the touch positions
                let touch1 = tl.get(0).unwrap();
                let touch2 = tl.get(1).unwrap();

                // new positions
                let [new1, new2] = [touch_pos(&touch1), touch_pos(&touch2)];

                let dist = |[x1, y1]: [i32; 2], [x2, y2]: [i32; 2]| {
                    f64::from((x1 - x2).pow(2) + (y1 - y2).pow(2)).sqrt()
                };

                let dist_old = dist(old1, old2);
                let dist_new = dist(new1, new2);

                // calculate the delta
                let delta = dist_new - dist_old;

                // calculate centerpoint for zoom
                let center = zip_map(new1, new2, |a, b| f64::from(a + b) / 2.0);

                apply_zoom(position, zoom, center, delta);
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
fn get_touch_positions(tl: &TouchList) -> Vec<[i32; 2]> {
    let mut positions = Vec::new();

    // iterate over the touches and store the positions
    for i in 0..tl.length() {
        let touch = tl.get(i).unwrap();

        positions.push(touch_pos(&touch));
    }

    positions
}

#[inline]
fn touch_pos(touch: &Touch) -> [i32; 2] {
    [touch.client_x(), touch.client_y()]
}
