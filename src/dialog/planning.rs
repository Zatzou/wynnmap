use std::sync::Arc;

use leptos::prelude::*;
use wynnmap_types::Guild;

use crate::dialog::{self, DialogCloseButton, Dialogs, close_dialog, show_dialog};

pub fn manage_guilds(guilds: RwSignal<Vec<Guild>>) -> impl IntoView {
    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white w-screen max-w-3xl h-dvh md:max-h-150 flex flex-col">
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-4xl">"Manage guilds"</h1>

                    <DialogCloseButton />
                </div>

                <hr class="border-neutral-600" />
            </div>

            <div class="p-2">
                <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                    let owner = Owner::new();
                    move |_| {
                        owner.with(move || {
                            show_dialog(move || dialog::planning::add_guild(guilds));
                        });
                    }
                }>
                    "Add guild"
                </button>

                <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                    let owner = Owner::new();
                    move |_| {
                        owner.with(move || {
                            // show_dialog(dialog::planning::manage_terrs);
                        });
                    }
                }>
                    "Import guild"
                </button>
            </div>

            <hr class="border-neutral-600" />

            <div class="flex-1 overflow-y-auto">
                <table class="table-auto w-full">
                    <thead>
                        <tr>
                            <th>"Tag"</th>
                            <th>"Name"</th>
                            <th>"Color"</th>
                            <th/>
                        </tr>
                    </thead>
                    <tbody>
                        <ForEnumerate
                            each=move || guilds.get()
                            key=|guild| guild.clone()
                            children=move |idx, guild| {
                                view! {
                                    <tr>
                                        <td class="p-2 font-mono">"["{guild.prefix}"]"</td>
                                        <td class="p-2">{guild.name}</td>
                                        <td>{guild.color}</td>
                                        <td class="flex">
                                            <Show when={move || idx.get() != 0}>
                                                // edit button
                                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6"
                                                    on:click={
                                                        let owner = Owner::new();
                                                        let idx = idx.get();
                                                        move |_| {
                                                            owner.with(move || {
                                                                show_dialog(move || edit_guild(guilds, idx as u8));
                                                            });
                                                        }
                                                    }
                                                >
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                                                </svg>

                                                // delete button
                                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                                                </svg>
                                            </Show>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}

fn add_guild(guilds: RwSignal<Vec<Guild>>) -> impl IntoView {
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
                            guilds.push(Guild {
                                uuid: None,
                                prefix: Some(Arc::from(tag.get())),
                                name: Some(Arc::from(name.get())),
                                color: Some(Arc::from(color.get())),
                            });
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

fn edit_guild(guilds: RwSignal<Vec<Guild>>, n: u8) -> impl IntoView {
    let guild = guilds
        .read_untracked()
        .get(n as usize)
        .cloned()
        .unwrap_or_default();

    let tag = RwSignal::new(guild.prefix.unwrap_or_default().to_string());
    let name = RwSignal::new(guild.name.unwrap_or_default().to_string());
    let color = RwSignal::new(guild.color.unwrap_or_default().to_string());

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
                            if let Some(guild) = guilds.get_mut(n as usize) {
                                guild.prefix = Some(Arc::from(tag.get()));
                                guild.name = Some(Arc::from(name.get()));
                                guild.color = Some(Arc::from(color.get()));
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

#[component]
fn guild_fields(
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
