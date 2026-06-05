//! This module contains the components for the sidecard.
//!
//! The sidecard is used in many views where information is displayed at the side. For example territory or guild information.

use leptos::prelude::*;

pub mod terr;

/// Base sidecard component
///
/// This is the sidecard component which should be used for the info card on the right side of the screen.
/// If hover is true the card is only visible on platforms which support hover.
#[component]
pub fn SideCard(
    #[prop(optional)] children: Option<Children>,
    #[prop(optional)] hover: bool,
    #[prop(optional, into)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="sidecard" class:hover={hover}>
            {move || on_close.map(|f| view!{
                <div class="cursor-pointer absolute top-2 right-2 m-0" on:click={move |_| f.run(())}>
                    <lucide_leptos::X size=32/>
                </div>
            })}
            <div class="content">
                {children.map(|c| c())}
            </div>
        </div>
    }
}
