use leptos::prelude::*;
use leptos_router::components::A;

use crate::error::ErrorBox;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <ErrorBox title="404 Not Found">
            <p>"The page you are looking for does not exist."</p>
            <p class="text-blue-500 underline"><A href="/">"Go to Home"</A></p>
        </ErrorBox>
    }
}
