use leptos::prelude::*;

#[component]
pub fn Checkbox(
    id: &'static str,
    checked: RwSignal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <label for={id} class="items-center checkbox-contain">
            <input id={id} type="checkbox" bind:checked={checked} class="mr-2" />
            <span class="checkmark"></span>
            <p class="texts">{children.map(|c| c())}</p>
        </label>
    }
}
