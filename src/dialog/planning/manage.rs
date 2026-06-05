use leptos::prelude::*;
use wynnmap_types::guild::Guild;

use crate::dialog::{
    DialogCloseButton, Dialogs,
    planning::{add::add_guild, edit::edit_guild},
};

pub fn manage_guilds(guilds: RwSignal<Vec<ArcRwSignal<Guild>>>) -> impl IntoView {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

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
                    move |_| {
                        dialogs.add("add_guild", move || add_guild(guilds));
                    }
                }>
                    "Add guild"
                </button>

                // TODO: allow importing guilds from the map/api
                // <button class="p-2 m-2 border-neutral-600 border rounded-md hover:bg-neutral-700" on:click={
                //     let owner = Owner::new();
                //     move |_| {
                //         owner.with(move || {
                //             // show_dialog(dialog::planning::manage_terrs);
                //         });
                //     }
                // }>
                //     "Import guild"
                // </button>
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
                            key=|guild| guild.get()
                            children=move |idx, guild| {
                                view! {
                                    <tr class="divide-x-1 divide-neutral-600">
                                        <td class="border-b-1 border-neutral-600 p-2 font-mono size-fit">"["{guild.get().prefix}"]"</td>
                                        <td class="border-b-1 border-neutral-600 p-2">{guild.get().name}</td>
                                        <td class="border-b-1 border-neutral-600 p-2 size-fit">
                                            <span class="pr-2" style:color={guild.read().hex_color()}>"⬤"</span>
                                            {guild.read().hex_color()}
                                        </td>
                                        // class="text-neutral-600" - help the tailwind cli a bit
                                        <td class="border-b-1 border-neutral-600 flex p-2 gap-1 size-fit" class:text-neutral-600={move || idx.read() == 0}>
                                            // edit button
                                            <div class="cursor-pointer" on:click={
                                                    let idx = idx.get();
                                                    move |_| {
                                                        if idx != 0 {
                                                            dialogs.add("edit_guild", move || edit_guild(guilds, idx));
                                                        }
                                                    }
                                                }
                                            >
                                                <lucide_leptos::SquarePen size=24/>
                                            </div>

                                            // delete button
                                            <div class="cursor-pointer" on:click={ move |_|
                                                    if idx.read() != 0 {
                                                        guilds.update(|guilds| {
                                                            guilds.remove(idx.get());
                                                        });
                                                    }
                                                }
                                            >
                                                <lucide_leptos::Trash size=24/>
                                            </div>
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
