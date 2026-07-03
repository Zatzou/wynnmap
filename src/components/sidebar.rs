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
        <div class="flex flex-col">
            // <hr class="border-neutral-600" />
            <div class="flex justify-between items-center text-xl pl-2 pr-2 py-1 cursor-pointer" on:click={move |_| toggle_modeswitch.set(!toggle_modeswitch.get())}>
                <div class="flex flex-row items-center pt-0.5 pb-0.5 gap-1">
                    <ModeswitchTitle cur=cur_path />
                </div>
                <Show when=move || !toggle_modeswitch.get()><lucide_leptos::ChevronUp size=24/></Show>
                <Show when=move || toggle_modeswitch.get()><lucide_leptos::ChevronDown size=24/></Show>
            </div>
            <hr class="border-neutral-600" class:hidden={move || !toggle_modeswitch.get()}/>
            <div class="flex flex-col overflow-y-auto shrink min-h-0 text-xl switchmode " class:hidden={move || !toggle_modeswitch.get()}>
                <a href="/" class="flex flex-row gap-1 pl-2 items-center" class:hidden={move || {cur_path.get() == "/"}}>
                    <lucide_leptos::Swords size=24/>
                    <h2> "Main" </h2>
                </a>
                <a href="plan" class="flex flex-row gap-1 pl-2 items-center" class:hidden={move || {cur_path.get() == "/plan"}}>
                    <lucide_leptos::LandPlot size=24/>
                    <h2> "Planning" </h2>
                </a>
                <a href="gather" class="flex flex-row gap-1 pl-2 items-center" class:hidden={move || {cur_path.get() == "/gather"}}>
                    <lucide_leptos::Axe size=24/>
                    <h2> "Gathering" </h2>
                </a>
                
                
            </div>
        </div>
    }
}
#[component]
pub fn ModeswitchTitle(cur: Memo<String>) -> impl IntoView {
    let cur_title = move || {
        match cur.get().as_str() {
            "/" => "War mode",
            "/plan" => "Planning mode",
            "/gather" => "Gather nodes mode",
            _ => "??? Mode"
        }
    };
    let cur_icon = move || {
        match cur.get().as_str() {
            "/" => view! {<lucide_leptos::Swords size=24/>}.into_any(),
            "/plan" => view! {<lucide_leptos::LandPlot size=24/>}.into_any(),
            "/gather" => view! {<lucide_leptos::Axe size=24/>}.into_any(),
            _ => view! {<lucide_leptos::CircleQuestionMark size=24/>}.into_any()
        }
    };

    view! {
        {cur_icon} {move || cur_title}
    }
}