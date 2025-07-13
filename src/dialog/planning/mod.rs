use leptos::prelude::*;

/// Dialog to add a new guild
mod add;
/// Dialog to edit an existing guild
mod edit;
/// The whole manage guilds dialog
mod manage;

pub use manage::manage_guilds;

/// Component for the fields that are shared by add guild and edit guild dialogs
#[component]
pub(self) fn guild_fields(
    tag: RwSignal<String>,
    name: RwSignal<String>,
    color: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="p-2 flex flex-col">
            <label class="flex flex-col">
                "Guild tag:"
                <input type="text" placeholder="Some" bind:value=tag maxlength="4" class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" />
            </label>
            <label class="flex flex-col">
                "Guild name:"
                <input type="text" placeholder="Some" bind:value=name class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" />
            </label>
            <label class="flex flex-col">
                "Guild color:"
                <input type="color" placeholder="Some" bind:value=color class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" />
            </label>
        </div>
    }
}
