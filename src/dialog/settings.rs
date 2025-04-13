use std::fmt::Display;

use leptos::prelude::*;

use crate::{components::checkbox::Checkbox, dialog::DialogCloseButton, settings::use_toggle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsView {
    General,
    GuildMap,
}

impl Display for SettingsView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsView::General => f.write_str("General"),
            SettingsView::GuildMap => f.write_str("Guild map"),
        }
    }
}

pub fn settings_dialog() -> impl IntoView {
    let settings_view = RwSignal::new(SettingsView::General);

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white w-screen max-w-3xl h-dvh md:max-h-150 flex flex-col">
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-4xl">"Settings"</h1>

                    <DialogCloseButton />
                </div>

                <hr class="border-neutral-600" />

                <div class="flex">
                    {[
                        SettingsView::General,
                        SettingsView::GuildMap,
                    ].into_iter().map(|sv| {
                        view! {
                            // class="bg-neutral-800"
                            <p
                                class="p-2 hover:bg-neutral-700 cursor-pointer not-first:border-l-1 border-neutral-600 "
                                class:bg-neutral-800={move || settings_view.read() == sv}
                                on:click={move |_| settings_view.set(sv)}
                            >{sv.to_string()}</p>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <hr class="border-neutral-600" />
            </div>

            <div class="p-2 overflow-y-auto grow">
                <Show when={move || settings_view.read() == SettingsView::General}>
                    <GeneralSettings />
                </Show>

                <Show when={move || settings_view.read() == SettingsView::GuildMap}>
                    <GuildMapSettings />
                </Show>
            </div>
        </div>
    }
}

#[component]
fn GeneralSettings() -> impl IntoView {
    let show_non_main = use_toggle("show_non_main_maps", false);

    view! {
        <div class="flex-1 flex flex-col p-2 text-lg">
            <Checkbox id="nonmains" checked={show_non_main}>"Show non-main map areas"</Checkbox>
        </div>
    }
}

#[component]
fn GuildMapSettings() -> impl IntoView {
    let show_terrs = use_toggle("terrs", true);
    let show_conns = use_toggle("conns", true);
    let show_gtag = use_toggle("terrs_show_guildtag", true);
    let show_res = use_toggle("resico", true);
    let show_timers = use_toggle("timers", true);

    view! {
        <div class="flex-1 flex flex-col gap-2 p-2 text-lg">
            <div>
                <Checkbox id="terrs" checked={show_terrs}>"Territories"</Checkbox>
                <div class="flex flex-col gap-1 ml-6">
                    <Checkbox id="gtag" checked={show_gtag}>"Show guild tags"</Checkbox>
                    <Checkbox id="resico" checked={show_res}>"Show resource icons"</Checkbox>
                    <Checkbox id="timers" checked={show_timers}>"Show timers"</Checkbox>
                </div>
            </div>
            <div>
                <Checkbox id="conns" checked={show_conns}>"Connections"</Checkbox>
            </div>
        </div>
    }
}
