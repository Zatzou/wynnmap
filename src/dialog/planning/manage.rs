use leptos::prelude::*;
use wynnmap_types::Guild;

use crate::dialog::{
    DialogCloseButton,
    planning::{add::add_guild, edit::edit_guild},
    show_dialog,
};

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
                            show_dialog(move || add_guild(guilds));
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

            <div class="overflow-y-auto shrink relative">
                <table class="table-auto w-full border-separate border-spacing-0">
                    <thead class="sticky top-0 bg-neutral-900">
                        <tr class="divide-x-1 divide-neutral-600">
                            <th class="border-b-1 border-neutral-600 w-0">"Tag"</th>
                            <th class="border-b-1 border-neutral-600">"Name"</th>
                            <th class="border-b-1 border-neutral-600 w-0">"Color"</th>
                            <th class="border-b-1 border-neutral-600 w-0"></th>
                        </tr>
                    </thead>
                    <tbody class="divide-y-1 divide-neutral-600">
                        <ForEnumerate
                            each=move || guilds.get()
                            key=|guild| guild.clone()
                            children=move |idx, guild| {
                                view! {
                                    <tr class="divide-x-1 divide-neutral-600">
                                        <td class="border-b-1 border-neutral-600 p-2 font-mono size-fit">"["{guild.prefix.clone()}"]"</td>
                                        <td class="border-b-1 border-neutral-600 p-2">{guild.name.clone()}</td>
                                        <td class="border-b-1 border-neutral-600 p-2 size-fit">
                                            <span class="pr-2" style:color={guild.hex_color()}>"⬤"</span>
                                            {guild.hex_color()}
                                        </td>
                                        // class="text-neutral-600" - help the tailwind cli a bit
                                        <td class="border-b-1 border-neutral-600 flex p-2 gap-1 size-fit" class:text-neutral-600={move || idx.read() == 0}>
                                            // edit button
                                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6"
                                                on:click={
                                                    let owner = Owner::new();
                                                    let idx = idx.get();
                                                    move |_| {
                                                        if idx != 0 {
                                                            owner.with(move || {
                                                                show_dialog(move || edit_guild(guilds, idx as u8));
                                                            });
                                                        }
                                                    }
                                                }
                                            >
                                                <path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10" />
                                            </svg>

                                            // delete button
                                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6"
                                                on:click={move |_|
                                                    if *idx.read() != 0 {
                                                        guilds.update(|guilds| {
                                                            guilds.remove(*idx.read());
                                                        });
                                                    }
                                                }
                                            >
                                                <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                                            </svg>
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
