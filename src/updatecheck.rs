use std::time::Duration;

use gloo_net::http::Request;
use jiff::{SignedDuration, Timestamp};
use leptos::{ev, prelude::*, task::spawn_local};

use crate::{
    datasource::NetworkError,
    dialog::{DialogCloseButton, Dialogs},
};

const CURRENT: &str = env!("CARGO_PKG_VERSION");

pub fn checker() {
    let dialogs = expect_context::<Dialogs>();

    let last_checked = RwSignal::new(Timestamp::now());
    let ignore = RwSignal::new(false);

    let check = move || async move {
        if ignore.get_untracked()
            && Timestamp::now().duration_since(last_checked.get_untracked())
                < SignedDuration::from_hours(10)
        {
            return;
        }

        let ver = get_version().await;

        if let Ok(ver) = ver {
            last_checked.set(Timestamp::now());
            if ver.as_str() > CURRENT {
                dialogs.add_if_not_exist("update", move || update_dialog(ver.clone(), ignore));
            }
        }
    };

    spawn_local(check());

    set_interval(move || spawn_local(check()), Duration::from_hours(1));

    let check_if_expired = move |_| {
        if Timestamp::now().duration_since(last_checked.get_untracked())
            > SignedDuration::from_hours(1)
        {
            spawn_local(check());
        }
    };

    // do the check when the tab is opened (mainly for mobile browsers)
    window_event_listener(ev::visibilitychange, check_if_expired);
}

async fn get_version() -> Result<String, NetworkError> {
    let res = Request::get("/version.json")
        .header("Cache-Control", "no-store")
        .header("Accept", "application/json")
        .send()
        .await?;

    if (200..=299).contains(&res.status().into()) {
        Ok(res.json().await?)
    } else {
        Err(NetworkError::BadStatus(res.status(), res.status_text()))
    }
}

fn update_dialog(version: String, ignore: RwSignal<bool>) -> impl IntoView {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let update = move |_| {
        let _ = window().location().reload();
    };

    let ignore = move |_| {
        ignore.set(true);
        dialogs.close();
    };

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white flex flex-col">
            <div class="flex justify-between p-2 items-center">
                <h2 class="text-4xl">"Update available"</h2>

                <DialogCloseButton />
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                <p>"Version "{version}" of Wynnmap is available."</p>
                <p>"You can update by refreshing the page or by clicking the button bellow."</p>
                <p>"You can also not update, however staying on old versions may cause issues."</p>
            </div>

            <div class="p-2 flex gap-2 justify-end">
                <button on:click=update class="p-1 px-2 border-1 border-neutral-600 hover:bg-neutral-700 rounded-lg cursor-pointer">Update</button>
                <button on:click=ignore class="p-1 px-2 border-1 border-neutral-600 hover:bg-neutral-700 rounded-lg cursor-pointer">Ignore</button>
            </div>
        </div>
    }
}
