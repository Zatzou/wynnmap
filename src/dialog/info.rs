use leptos::prelude::*;

use crate::dialog::Dialogs;

/// Simple info dialog
pub fn info_dialog(title: String, children: impl IntoView) -> impl IntoView {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let close = move |_| {
        dialogs.close();
    };

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white flex flex-col">
            <h2 class="text-3xl p-2">{title}</h2>

            <hr class="border-neutral-600" />

            <div class="p-2">
                {children.into_view()}
            </div>

            <div class="flex justify-end p-2">
                <button on:click={close} class="button-small">"OK"</button>
            </div>
        </div>
    }
}
