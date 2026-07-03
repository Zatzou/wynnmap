use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};

use crate::dialog::{self, Dialogs};

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
fn Modeswitch() -> impl IntoView {
    let toggle_modeswitch = RwSignal::new(false);
    let cur_path = use_location().pathname;

    view! {
        <div class="modeswitcher">
            <div class="title" on:click=move |_| toggle_modeswitch.update(|s| *s = !*s)>
                <div class="flex flex-row gap-1 items-center">
                    <ModeswitchTitle cur=cur_path />
                </div>
                <Show when=move || !toggle_modeswitch.get()><lucide_leptos::ChevronDown size=24/></Show>
                <Show when=move || toggle_modeswitch.get()><lucide_leptos::ChevronUp size=24/></Show>
            </div>

            <hr class:hidden=move || !toggle_modeswitch.get()/>

            <div class="options" class:hidden=move || !toggle_modeswitch.get()>
                <ModeswitchItem location="/" hide=move || cur_path.get() == "/"/>
                <ModeswitchItem location="/plan" hide=move || cur_path.get() == "/plan"/>
                <ModeswitchItem location="/gather" hide=move || cur_path.get() == "/gather"/>
            </div>
        </div>
    }
}

#[component]
fn ModeswitchItem(
    #[prop(into)] location: Signal<String>,
    #[prop(into)] hide: Signal<bool>,
) -> impl IntoView {
    view! {
        <A href=move || location.get() attr:class="flex flex-row gap-1 pl-2 items-center" class:hidden=hide>
            <ModeswitchTitle cur=location/>
        </A>
    }
}

#[component]
fn ModeswitchTitle(#[prop(into)] cur: Signal<String>) -> impl IntoView {
    let cur_title = move || match cur.get().as_str() {
        "/" => "War mode",
        "/plan" => "Planning mode",
        "/gather" => "Gather mode",
        _ => "Unknown mode",
    };
    let cur_icon = move || match cur.get().as_str() {
        "/" => view! {<lucide_leptos::Swords size=24/>}.into_any(),
        "/plan" => view! {<lucide_leptos::LandPlot size=24/>}.into_any(),
        "/gather" => view! {<lucide_leptos::Axe size=24/>}.into_any(),
        _ => view! {<lucide_leptos::CircleQuestionMark size=24/>}.into_any(),
    };

    view! {
        {cur_icon} {cur_title}
    }
}
