use leptos::prelude::*;
use web_sys::PointerEvent;

use crate::wynnmap::RelMousePos;

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
    position: RwSignal<(f64, f64)>,
    zoom: RwSignal<f64>,
    moving: RwSignal<bool>,
    onclick: Option<Callback<(i32, i32)>>,
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
            let pos = (e.client_x(), e.client_y());

            // calculate the compensation
            let zoom = zoom.get();
            let map_pos = position.get();
            let rel = (
                (pos.0 as f64 - map_pos.0) / zoom,
                (pos.1 as f64 - map_pos.1) / zoom,
            );
            relmousepos.set(Some((rel.0 as i32, rel.1 as i32)));
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
            cb.run(relmousepos.get().unwrap_or_default())
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
