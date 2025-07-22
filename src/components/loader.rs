use leptos::prelude::*;

use crate::{components::sidebar::Sidebar, error::ErrorBox};

pub fn loader<T: Clone + 'static>(
    res: LocalResource<Result<T, String>>,
    f: impl Fn(T) -> AnyView,
) -> AnyView {
    match res.get() {
        None => view! {<LoadBox/> <Sidebar/>}.into_any(),
        Some(Err(e)) => view! {
            <ErrorBox title="Failed to load api data">
                <p>"An error occured while loading api data"</p>
                <pre class="p-2 bg-neutral-800 rounded my-1">{e}</pre>
            </ErrorBox>
        }
        .into_any(),
        Some(Ok(data)) => f(data),
    }
}

#[component]
pub fn LoadBox() -> impl IntoView {
    view! {
        <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
            <div class="p-8 rounded-full border-neutral-50/10 border-5 border-r-neutral-50 animate-spin" />
        </div>
    }
}
