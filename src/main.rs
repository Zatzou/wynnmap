use components::sidebar::ShowSidebar;
use leptos::prelude::*;
use modes::war::WarMap;
use settings::provide_settings;

mod components;
mod datasource;
mod modes;
mod settings;
mod wynnmap;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });
}

#[component]
pub fn App() -> impl IntoView {
    provide_settings();

    provide_context(ShowSidebar(RwSignal::new(false)));

    view! {
        <WarMap/>
    }
}
