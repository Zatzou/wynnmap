use leptos::{leptos_dom::logging::console_log, prelude::*};
use leptos_router::hooks::use_location;

use crate::{dialog::{self, Dialogs}, settings::use_toggle};

#[derive(Clone)]
pub struct ShowSidebar(pub RwSignal<bool>);

#[component]
pub fn Sidebar(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let dialogs = expect_context::<Dialogs>();
    let show_sidebar = expect_context::<ShowSidebar>().0;
    let toggle_sidebar = move |_| show_sidebar.update(|s| *s = !*s);
    view! {
        // sidebar open button
        <div on:click={toggle_sidebar} class="sidebar-btn">
            <lucide_leptos::Menu size=32/>
        </div>

        <div class="sidebar" class:closed={move || !show_sidebar.get()}>
            // top text
            <div class="title">
                <h1>Wynnmap</h1>

                // close button
                <div class="cursor-pointer" on:click=toggle_sidebar>
                    <lucide_leptos::X size=32/>
                </div>
            </div>

            <Modeswitch/>

            <div class="content">
                {children.map(|c| c())}
            </div>

            // settings button
            <div class="settings-btn"
                on:click={
                    move |_| {
                        dialogs.add("settings", dialog::settings::settings_dialog);
                    }
                }
            >
                <lucide_leptos::Settings size=24/>
                <h2>"Settings"</h2>
            </div>

            // bottom text
            <div>
                <h2 class="text-neutral-500 p-1 px-2">
                    <a class="underline" href="https://github.com/Zatzou/wynnmap" target="_blank">"Wynnmap"</a>" "{env!("CARGO_PKG_VERSION")}
                </h2>
            </div>
        </div>
    }
}

#[component]
pub fn Modeswitch() -> impl IntoView {
    let toggle_modeswitch = use_toggle("modeswitch", false);
    let cur_path = use_location().pathname;
    view! {
        <div class="flex flex-col min-h-0">
            // <hr class="border-neutral-600" />
            <div class="flex justify-between items-center text-xl p-2 py-1 cursor-pointer" on:click={move |_| toggle_modeswitch.set(!toggle_modeswitch.get())}>
                <h2>{move || switch_title(cur_path.get())}</h2>
                <Show when=move || !toggle_modeswitch.get()><lucide_leptos::ChevronUp size=24/></Show>
                <Show when=move || toggle_modeswitch.get()><lucide_leptos::ChevronDown size=24/></Show>
            </div>
            <div class="overflow-y-auto shrink min-h-0" class:hidden={move || !toggle_modeswitch.get()}>
                // <hr class="border-neutral-600"/>
                <div class="switchmode">
                    <p> <a class:hidden={move || {cur_path.get() == "/"}} href="/">"Main"</a> </p>
                    <p> <a class:hidden={move || {cur_path.get() == "/plan"}} href="plan">"Planning"</a> </p>
                    <p> <a class:hidden={move || {cur_path.get() == "/gather"}} href="gather">"Gathering"</a> </p> 
                </div>
            </div>
        </div>
    }
}

pub fn switch_title(cur: String) -> &'static str {
    match cur.as_str() {
        "/" => "War mode",
        "/plan" => "Planning mode",
        "/gather" => "Gather nodes mode",
        _ => "??? Mode"
    }
}