use leptos::prelude::*;

#[derive(Clone)]
pub struct ShowSidebar(pub RwSignal<bool>);

#[component]
pub fn Sidebar(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let show_sidebar = use_context::<ShowSidebar>().unwrap().0;

    view! {
        // sidebar open button
        <div on:click={move |_| show_sidebar.set(!show_sidebar.get())} class="fixed top-0 left-0 p-2 cursor-pointer z-50 bg-neutral-900 rounded-e-full mt-2 p-2">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 text-white">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
            </svg>
        </div>

        // class="-translate-x-full"
        <div class="flex flex-col bg-neutral-900 w-full max-w-full h-screen z-50 absolute top-0 md:max-w-sm transition-transform text-white" class:-translate-x-full={move || !show_sidebar.get()}>
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

            // bottom text
            <div class="mt-auto">
                <hr class="border-neutral-600" />
                <h2 class="text-neutral-500 p-1 px-2"><a class="underline" href="https://github.com/Zatzou/wynnmap" target="_blank">"Wynnmap"</a>" "{env!("CARGO_PKG_VERSION")}</h2>
            </div>
        </div>
    }
}
