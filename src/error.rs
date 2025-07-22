use std::fmt::Debug;

use leptos::prelude::*;

use crate::components::sidebar::Sidebar;

pub fn debug_fmt_error<T: Debug>(err: T) -> String {
    format!("{err:?}")
}

#[component]
pub fn ErrorBox(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <div class="bg-neutral-900 text-white absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-lg">
            <div class="flex justify-between p-2 items-center">
                <h1 class="text-4xl">{title}</h1>
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                {children()}
            </div>
        </div>

        <Sidebar />
    }
}
