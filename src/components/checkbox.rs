use leptos::prelude::*;

#[component]
pub fn Checkbox(
    id: &'static str,
    checked: RwSignal<bool>,
    #[prop(optional)] disabled: Option<RwSignal<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="checkbox">
            <input id type="checkbox" bind:checked=checked disabled={move || if disabled.is_some() {disabled.unwrap().get()} else {false}}/>
            <div class="checkmark">
                <div/>
            </div>

            <label for=id>
                {children.map(|c| c())}
            </label>
        </div>
    }
}
