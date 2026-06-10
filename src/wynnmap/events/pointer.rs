use leptos::prelude::*;
use web_sys::PointerEvent;

use crate::wynnmap::{RelMousePos, util::zip_map};

pub struct PointerEventHandlers<PM, PD, PU, PL>
where
    PM: Fn(PointerEvent) + Copy + 'static,
    PD: Fn(PointerEvent) + Copy + 'static,
    PU: Fn(PointerEvent) + Copy + 'static,
    PL: Fn(PointerEvent) + Copy + 'static,
{
    pub pointermove: PM,
    pub pointerdown: PD,
    pub pointerup: PU,
    pub pointerleave: PL,
}

pub fn handlers(
    position: RwSignal<[f64; 2]>,
    zoom: RwSignal<f64>,
    moving: RwSignal<bool>,
    onclick: Option<Callback<[i32; 2]>>,
) -> PointerEventHandlers<
    impl Fn(PointerEvent) + Copy + 'static,
    impl Fn(PointerEvent) + Copy + 'static,
    impl Fn(PointerEvent) + Copy + 'static,
    impl Fn(PointerEvent) + Copy + 'static,
> {
    // use pointer events to handle clicks on the map
    let relmousepos = RwSignal::new(None);
    provide_context(RelMousePos(relmousepos));

    let pointermove = move |e: PointerEvent| {
        if !moving.get() {
            let pos = [e.client_x(), e.client_y()].map(f64::from);

            // calculate the compensation
            let zoom = zoom.get();
            let map_pos = position.get();
            let rel = zip_map(pos, map_pos, |p, m| ((p - m) / zoom) as i32);
            relmousepos.set(Some(rel));
        }
    };

    let dragstartpos = RwSignal::new([0; 2]);

    let pointerdown = move |e: PointerEvent| {
        dragstartpos.set([e.client_x(), e.client_y()]);

        // also update position on down to support touch events
        pointermove(e);
    };

    let pointerup = move |e: PointerEvent| {
        let pos = [e.client_x(), e.client_y()];
        let startpos = dragstartpos.get();

        let diff = zip_map(pos, startpos, |p, s| s.abs_diff(p));

        // emit clicks when the map hasnt been moved meaningfully
        if let Some(cb) = onclick
            && diff[0] < 5
            && diff[1] < 5
        {
            cb.run(relmousepos.get().unwrap_or_default());
        }
    };

    let pointerleave = move |_| {
        relmousepos.set(None);
    };

    PointerEventHandlers {
        pointermove,
        pointerdown,
        pointerup,
        pointerleave,
    }
}
