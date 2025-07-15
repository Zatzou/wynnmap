use leptos::prelude::*;

use crate::dialog::{DialogCloseButton, Dialogs, close_dialog};

/// Simple info dialog
pub fn info(title: String, children: impl IntoView) -> impl IntoView {
    let Dialogs(dialogs) = use_context::<Dialogs>().expect("Dialogs context not found");

    let close = move |_| {
        close_dialog(dialogs);
    };

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white flex flex-col">
            <div class="flex justify-between p-2 items-center">
                <h1 class="text-4xl">{title}</h1>

                <DialogCloseButton />
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                {children.into_view()}
            </div>

            <div class="flex justify-end p-2">
                <button on:click={close} class="p-1 px-2 border-1 border-neutral-600 hover:bg-neutral-700 rounded-lg">"Ok"</button>
            </div>
        </div>
    }
}
