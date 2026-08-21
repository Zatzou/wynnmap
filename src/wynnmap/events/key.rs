use leptos::{ev, prelude::*};
use web_sys::KeyboardEvent;

use crate::wynnmap::util::{apply_zoom, apply_zoom_compensation, get_viewport_middle};

pub fn handlers(position: RwSignal<[f64; 2]>, zoom: RwSignal<f64>, transitioning: RwSignal<bool>) {
    let onkeydown = move |e: KeyboardEvent| {
        match e.key().as_str() {
            // 0 key - reset zoom
            "0" => {
                let old_zoom = zoom.get();
                let new_zoom = 0.5;
                // reset the zoom
                zoom.set(new_zoom);

                // perform zoom compensation
                // get middle point of the screen
                let center = get_viewport_middle();
                // calculate the zoom compensation
                apply_zoom_compensation(center, old_zoom, new_zoom, position);

                // do transition
                transitioning.set(true);
            }
            // Home - reset position
            "Home" => {
                let screen_middle = get_viewport_middle();
                let zoom = zoom.get() * 2.0;

                position.set([
                    100.0f64.mul_add(zoom, screen_middle[0]),
                    1200.0f64.mul_add(zoom, screen_middle[1]),
                ]);

                transitioning.set(true);
            }
            // plus key - zoom in
            "+" => {
                // get middle point of the screen
                let center = get_viewport_middle();
                // apply the zoom
                apply_zoom(position, zoom, center, 0.3);

                // do transition
                transitioning.set(true);
            }
            // minus key - zoom out
            "-" => {
                // get middle point of the screen
                let center = get_viewport_middle();
                // apply the zoom
                apply_zoom(position, zoom, center, -0.3);

                // do transition
                transitioning.set(true);
            }
            // ArrowUp - move up
            "ArrowUp" => {
                position.update(|[_, y]| *y += 100.0 / zoom.get());

                transitioning.set(true);
            }
            // ArrowDown - move down
            "ArrowDown" => {
                position.update(|[_, y]| *y -= 100.0 / zoom.get());

                transitioning.set(true);
            }
            // ArrowLeft - move left
            "ArrowLeft" => {
                position.update(|[x, _]| *x += 100.0 / zoom.get());

                transitioning.set(true);
            }
            // ArrowRight - move right
            "ArrowRight" => {
                position.update(|[x, _]| *x -= 100.0 / zoom.get());

                transitioning.set(true);
            }
            // do nothing on unknown keys
            _ => {}
        }
    };

    window_event_listener(ev::keydown, onkeydown);
}
