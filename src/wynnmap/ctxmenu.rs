use leptos::{ev, prelude::*};
use web_sys::MouseEvent;

use crate::{util::as_px, wynnmap::context::RelMousePos};

pub type OnCtxMenu = Callback<(RwSignal<[i32; 2]>, Callback<()>), AnyView>;

pub fn context_menu(
    onctxmenu: Option<OnCtxMenu>,
) -> (impl Fn() -> AnyView, impl Fn(MouseEvent) + Copy + 'static) {
    let RelMousePos(relmousepos) = expect_context();

    let menu_pos = RwSignal::new([0, 0]);
    let menu_relpos = RwSignal::new([0, 0]);
    let menu_visible = RwSignal::new(false);

    let event_handler = move |event: MouseEvent| {
        event.prevent_default();

        let relpos = relmousepos.get_untracked().unwrap_or_default();

        menu_relpos.set(relpos);
        menu_pos.set([event.offset_x(), event.offset_y()]);
        menu_visible.set(true);
    };

    let close = move || {
        menu_visible.set(false);
    };

    // close the menu when a click happens or a key is pressed without stopping the propagation
    window_event_listener(ev::pointerdown, move |_| close());
    window_event_listener(ev::keydown, move |_| close());

    let view = move || {
        view! {
            <div
                tabindex=-1

                class="wynnmap-ctxmenu"
                class:hidden=move || !menu_visible.get()

                style:top=as_px(menu_pos.read()[1])
                style:left=as_px(menu_pos.read()[0])

                on:pointerdown=move |e| e.stop_propagation()
                on:keydown=move |e| e.stop_propagation()
            >
                <CopyCoordsBtn menu_relpos close/>
                {move || onctxmenu.map(|cb| cb.run((menu_relpos, close.into())))}
            </div>
        }
        .into_any()
    };

    (view, event_handler)
}

#[component]
fn CopyCoordsBtn(menu_relpos: RwSignal<[i32; 2]>, close: impl Fn() + 'static) -> impl IntoView {
    let click = move |_| {
        let [x, z] = menu_relpos.get();

        let clipboard = window().navigator().clipboard();

        let _promise = clipboard.write_text(&format!("/comp {x} {z}"));

        close();
    };

    view! {
        <button class="ctxmenu-btn" on:click=click>
            <icons::Clipboard/>
            "Copy position "{move || menu_relpos.read()[0]}" "{move || menu_relpos.read()[1]}
        </button>
    }
}
