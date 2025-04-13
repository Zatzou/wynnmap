use leptos::prelude::*;

use crate::dialog::{self, show_dialog};

#[derive(Clone)]
pub struct ShowSidebar(pub RwSignal<bool>);

#[component]
pub fn Sidebar(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let show_sidebar = use_context::<ShowSidebar>().unwrap().0;

    view! {
        // sidebar open button
        <div on:click={move |_| show_sidebar.set(!show_sidebar.get())} class="fixed top-0 left-0 p-2 cursor-pointer bg-neutral-900 rounded-e-full mt-2 p-2">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 text-white">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
            </svg>
        </div>

        // class="-translate-x-full"
        <div class="flex flex-col bg-neutral-900 w-full max-w-full h-dvh fixed top-0 md:max-w-sm transition-transform text-white" class:-translate-x-full={move || !show_sidebar.get()}>
            // top text
            <div>
                <div class="flex justify-between p-2 items-center">
                    <h1 class="text-4xl">Wynnmap</h1>

                    // close button
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer" on:click={move |_| show_sidebar.set(!show_sidebar.get())}>
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </div>
                <hr class="border-neutral-600" />
            </div>


            {children.map(|c| c())}

            // settings button
            <div class="mt-auto">
                <hr class="border-neutral-600" />
                <div class="cursor-pointer p-2 flex gap-2 items-center" on:click={
                    let owner = Owner::new();
                    move |_| {
                        owner.with(move || {
                            show_dialog(dialog::settings::settings_dialog);
                        })
                    }
                }>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                    </svg>
                    <h2 class="text-xl">"Settings"</h2>
                </div>
            </div>

            // bottom text
            <div>
                <hr class="border-neutral-600" />
                <h2 class="text-neutral-500 p-1 px-2"><a class="underline" href="https://github.com/Zatzou/wynnmap" target="_blank">"Wynnmap"</a>" "{env!("CARGO_PKG_VERSION")}</h2>
            </div>
        </div>
    }
}
