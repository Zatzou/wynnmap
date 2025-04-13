use leptos::prelude::*;

#[component]
pub fn Checkbox(
    id: &'static str,
    checked: RwSignal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <label for=id class="group flex items-center cursor-pointer select-none">
            // checkbox
            <input id=id type="checkbox" bind:checked=checked class="appearance-none relative peer mr-2 h-5 w-5 rounded-sm text-neutral-200 bg-neutral-700 group-hover:bg-neutral-500" />
            // checkmark
            <div class="absolute justify-center items-center h-5 w-5 hidden peer-checked:flex">
                <div class="w-[0.4em] h-[0.75em] mb-0.5 rotate-45 border-white border-0 border-r-3 border-b-3" />
            </div>
            // label
            <p>{children.map(|c| c())}</p>
        </label>
    }
}
