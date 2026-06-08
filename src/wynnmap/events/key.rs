use leptos::{ev, prelude::*};
use web_sys::KeyboardEvent;

use crate::wynnmap::util::{apply_zoom_compensation, calculate_new_zoom, get_viewport_middle};

pub fn handlers(
    position: RwSignal<(f64, f64)>,
    zoom: RwSignal<f64>,
    transitioning: RwSignal<bool>,
) {
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
}
