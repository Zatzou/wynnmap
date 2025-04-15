use components::sidebar::ShowSidebar;
use dialog::{DialogRenderer, provide_dialogs};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use modes::war::WarMap;
use notfound::NotFound;
use settings::provide_settings;

mod components;
mod datasource;
mod dialog;
mod modes;
mod notfound;
mod settings;
mod wynnmap;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| view! { <App /> });
}

#[component]
pub fn App() -> impl IntoView {
    provide_settings();
    provide_dialogs();

    provide_context(ShowSidebar(RwSignal::new(false)));

    view! {
        <Router>
            <Routes fallback=NotFound>
                <Route path=path!("") view=WarMap />
            </Routes>
        </Router>

        <DialogRenderer />
    }
}
