use components::sidebar::ShowSidebar;
use dialog::{DialogRenderer, provide_dialogs};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use modes::{planning::PlanningMap, war::WarMap};
use notfound::NotFound;
use settings::provide_settings;

use crate::wynnmap::maptile::ProvideDefaultMapTiles;

mod components;
mod datasource;
mod dialog;
mod error;
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
        <ProvideDefaultMapTiles>
            <Router>
                <Routes fallback=NotFound>
                    <Route path=path!("") view=WarMap />
                    <Route path=path!("plan") view=PlanningMap />
                </Routes>
            </Router>
        </ProvideDefaultMapTiles>

        <DialogRenderer />
    }
}
