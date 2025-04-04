use leptos::prelude::*;

#[component]
pub fn Incrementor(
    value: RwSignal<i32>,
    #[prop(into, optional)] min: Signal<i32>,
    #[prop(into, optional)] max: Signal<i32>,
) -> impl IntoView {
    let increment = move |_| value.update(|v| *v = (*v + 1).min(max.get()));
    let decrement = move |_| value.update(|v| *v = (*v - 1).max(min.get()));

    view! {
        <div class="items-center">
            <button on:click=decrement class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"-"</button>
            <span class="inline-block w-7 text-center">{value}</span>
            <button on:click=increment class="bg-neutral-600 hover:bg-neutral-300 inline-block w-6 font-bold rounded">"+"</button>
        </div>
    }
}
