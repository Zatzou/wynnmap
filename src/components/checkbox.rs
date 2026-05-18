use leptos::prelude::*;

#[component]
pub fn Checkbox(
    id: &'static str,
    checked: RwSignal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="checkbox">
            <input id=id type="checkbox" bind:checked=checked/>
            <div class="checkmark">
                <div/>
            </div>

            <label for=id>
                {children.map(|c| c())}
            </label>
        </div>
    }
}
