use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use leptos::{attr::AttributeValue, prelude::*, tachys::html::property::IntoProperty};

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
        + FromStr
        + Sync
        + AttributeValue
        + IntoProperty
        + 'static,
{
    let increment = move |_| {
        if *value.read() != *max.read() {
            value.update(|v| *v = (*v + 1.into()).min(max.get()))
        }
    };
    let decrement = move |_| {
        if *value.read() != *min.read() {
            value.update(|v| *v = (*v - 1.into()).max(min.get()))
        }
    };

    view! {
        <div class="items-center">
            <button on:click=decrement class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"-"</button>
            <input class="inline-block w-7 text-center" style:appearance="textfield" r#type="number" min=min max=max prop:value=value on:input:target=move |ev| {
                let val = ev.target().value();

                if let Ok(v) = val.parse::<T>() {
                    value.set(v.clamp(min.get(), max.get()))
                }
            }
            on:click:target=move |ev| ev.target().select()
            />
            <button on:click=increment class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"+"</button>
        </div>
    }
}
