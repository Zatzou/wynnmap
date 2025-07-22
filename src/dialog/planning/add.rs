use std::sync::Arc;

use leptos::prelude::*;
use wynnmap_types::guild::Guild;

use crate::dialog::{DialogCloseButton, Dialogs, close_dialog, planning::GuildFields};

pub(super) fn add_guild(guilds: RwSignal<Vec<ArcRwSignal<Guild>>>) -> impl IntoView {
    let tag = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let color = RwSignal::new(String::from("#FFFFFF"));

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white flex flex-col">
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-2xl">"Add guild"</h1>
                </div>

                <hr class="border-neutral-600" />
            </div>

            <GuildFields tag=tag name=name color=color />

            <hr class="border-neutral-600" />

            <div class="p-2 flex justify-between">
                <DialogCloseButton>
                    <button class="p-2 m-2 border-red-600 border rounded-md hover:bg-neutral-700">
                        "Cancel"
                    </button>
                </DialogCloseButton>

                <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                    let Dialogs(dialogs) = use_context::<Dialogs>().expect("Dialogs context not found");
                    move |_| {
                        // TODO: validate inputs
                        guilds.update(|guilds| {
                            guilds.push(ArcRwSignal::new(Guild {
                                uuid: None,
                                prefix: Arc::from(tag.get()),
                                name: Arc::from(name.get()),
                                color: Some(Arc::from(color.get())),
                            }));
                        });

                        close_dialog(dialogs);
                    }
                }>
                    "Add guild"
                </button>
            </div>
        </div>
    }
}
