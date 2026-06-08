use leptos::prelude::*;

use crate::wynnmap::{
    events::{mouse::MouseEventHandlers, pointer::PointerEventHandlers, touch::TouchEventHandlers},
    util::get_viewport_middle,
};

pub mod conns;
mod events;
pub mod maptile;
pub mod terrs;
mod util;

/// The minimum zoom level
const ZOOM_MIN: f64 = 0.0625;
/// The maximum zoom level
const ZOOM_MAX: f64 = 64.0;

/// Mouse position on the map atlas
#[derive(Clone)]
pub struct RelMousePos(pub RwSignal<Option<(i32, i32)>>);

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

    let MouseEventHandlers {
        mousemove,
        start_mousemove,
        end_mousemove,
        wheel,
    } = events::mouse::handlers(position, zoom, moving, transitioning);

    let TouchEventHandlers {
        touchstart,
        touchmove,
    } = events::touch::handlers(position, zoom, moving);

    events::key::handlers(position, zoom, transitioning);

    let PointerEventHandlers {
        pointermove,
        pointerdown,
        pointerup,
        pointerleave,
    } = events::pointer::handlers(position, zoom, moving, onclick);

    view! {
        // outermost container used for containing the map
        <div
            class="wynnmap-container"

            on:mousemove=mousemove
            on:mousedown=start_mousemove
            on:mouseup=end_mousemove
            on:mouseleave=end_mousemove
            on:wheel=wheel

            on:touchstart=touchstart
            on:touchmove=touchmove

            on:pointermove=pointermove
            on:pointerdown=pointerdown
            on:pointerup=pointerup
            on:pointerleave=pointerleave
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
