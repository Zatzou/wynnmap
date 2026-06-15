use leptos::prelude::*;

#[component]
pub fn Checkbox(
    #[prop(into)] id: Signal<String>,
    checked: RwSignal<bool>,
    #[prop(optional)] children: Option<Children>,
    #[prop(into, default = false.into())] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="checkbox">
            <input id=id type="checkbox" bind:checked=checked disabled=disabled/>
            <div class="checkmark">
                <div/>
            </div>

            <label for=id>
                {children.map(|c| c())}
            </label>
        </div>
    }
}
