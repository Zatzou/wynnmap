use std::ops::{Add, Sub};

use leptos::prelude::*;

#[component]
pub fn Incrementor<T>(
    value: RwSignal<T>,
    #[prop(into, optional)] min: Signal<T>,
    #[prop(into, optional)] max: Signal<T>,
) -> impl IntoView
where
    T: RenderHtml
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + From<u8>
        + Ord
        + Copy
        + Default
        + Sync
        + 'static,
{
    let increment = move |_| value.update(|v| *v = (*v + 1.into()).min(max.get()));
    let decrement = move |_| value.update(|v| *v = (*v - 1.into()).max(min.get()));

    view! {
        <div class="items-center">
            <button on:click=decrement class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"-"</button>
            <span class="inline-block w-7 text-center">{value}</span>
            <button on:click=increment class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"+"</button>
        </div>
    }
}
