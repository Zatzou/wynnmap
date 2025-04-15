use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::sidebar::Sidebar;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="bg-neutral-900 text-white absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 p-4 rounded-lg">
            <h1 class="text-3xl">"404 Not Found"</h1>
            <p>"The page you are looking for does not exist."</p>
            <p class="text-blue-500 underline"><A href="/">"Go to Home"</A></p>
        </div>

        <Sidebar />
    }
}
