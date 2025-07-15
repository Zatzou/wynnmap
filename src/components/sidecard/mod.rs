//! This module contains the components for the sidecard.
//!
//! The sidecard is used in many views where information is displayed at the side. For example territory or guild information.

use leptos::prelude::*;

pub mod terr;

/// Base sidecard component for non interactive cards.
///
/// This component should be used for cards which appear on hover and are not interactive.
#[component]
pub fn SideCardHover(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="fixed top-4 right-4 bg-neutral-900 text-white rounded-md w-sm terrinfo-hoverbox pointer-events-none">
            {children.map(|c| c())}
        </div>
    }
}

/// Base sidecard component
///
/// This is the sidecard component which should be used for interactive cards. This component also properly supports mobile devices.
/// This component also has a close button which closes the card when clicked.
#[component]
pub fn SideCard(
    #[prop(optional)] children: Option<Children>,
    closefn: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="fixed top-0 right-0 bg-neutral-900 text-white w-full max-w-full md:max-w-sm md:top-4 md:right-4 md:rounded max-h-dvh overflow-x-hidden overflow-y-auto">
            // close button
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer absolute top-2 right-2" on:click={move |_| closefn()}>
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>

            {children.map(|c| c())}
        </div>
    }
}
