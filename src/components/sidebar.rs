use leptos::prelude::*;

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
