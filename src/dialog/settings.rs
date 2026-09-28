use std::fmt::Display;

use leptos::prelude::*;

use crate::{
    components::checkbox::Checkbox, dialog::DialogCloseButton, settings::SettingsCtx,
    util::CoordFmtSelector,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsView {
    General,
    GuildMap,
}

impl Display for SettingsView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General => f.write_str("General"),
            Self::GuildMap => f.write_str("Guild map"),
        }
    }
}

pub fn settings_dialog() -> impl IntoView {
    let settings_view = RwSignal::new(SettingsView::General);

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white w-screen max-w-3xl h-dvh md:max-h-150 flex flex-col">
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h2 class="text-4xl">"Settings"</h2>

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
    let SettingsCtx(settings) = expect_context();
    let show_non_main = settings.map.show_non_main_areas;
    let copy_coordfmt = settings.copy_coordfmt;

    view! {
        <div class="flex-1 flex flex-col p-2 gap-2 text-lg">
            <Checkbox id="nonmains" checked={show_non_main}>"Show non-main map areas"</Checkbox>
            <div>
                <span class="mr-2">"Prefered copy coordinate format"</span>
                <CoordFmtSelector fmt=copy_coordfmt/>
            </div>
        </div>
    }
}

#[component]
fn GuildMapSettings() -> impl IntoView {
    let SettingsCtx(settings) = expect_context();
    let show_terrs = settings.map.show_terrs;
    let show_conns = settings.map.show_conns;

    let show_gtag = settings.terrs.show_gtags;
    let show_res = settings.terrs.show_resicons;
    let show_timers = settings.terrs.show_timers;

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
