use std::sync::Arc;

use leptos::prelude::*;
use wynnmap_types::Guild;

use crate::dialog::{DialogCloseButton, Dialogs, close_dialog, planning::GuildFields};

pub(super) fn edit_guild(guilds: RwSignal<Vec<ArcRwSignal<Guild>>>, n: usize) -> impl IntoView {
    let guild = guilds.read_untracked().get(n).cloned().unwrap_or_default();

    let tag = RwSignal::new(guild.get_untracked().prefix.unwrap_or_default().to_string());
    let name = RwSignal::new(guild.get_untracked().name.unwrap_or_default().to_string());
    let color = RwSignal::new(guild.get_untracked().color.unwrap_or_default().to_string());

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white flex flex-col">
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-2xl">"Edit guild"</h1>
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
                            if let Some(guild) = guilds.get_mut(n) {
                                guild.update(|guild| {
                                    guild.prefix = Some(Arc::from(tag.get()));
                                    guild.name = Some(Arc::from(name.get()));
                                    guild.color = Some(Arc::from(color.get()));
                                });
                            }
                        });

                        close_dialog(dialogs);
                    }
                }>
                    "Save"
                </button>
            </div>
        </div>
    }
}
