use std::sync::Arc;

use leptos::prelude::*;

pub mod info;
pub mod planning;
pub mod settings;

#[derive(Clone, Copy)]
pub struct Dialogs {
    pub dialogs: RwSignal<Vec<(Arc<str>, ViewFn)>>,
}

impl Dialogs {
    /// Close the current top dialog
    pub fn close(&self) {
        self.dialogs.update(|d| {
            d.pop();
        });
    }

    pub fn add(&self, name: impl Into<Arc<str>>, view: impl Into<ViewFn>) {
        self.dialogs.update(|d| {
            d.push((name.into(), view.into()));
        });
    }

    pub fn contains(&self, name: impl Into<Arc<str>>) -> bool {
        let name = name.into();

        self.dialogs.read().iter().any(|(n, _)| *n == name)
    }
}

pub fn provide_dialogs() {
    let dialogs = RwSignal::new(Vec::new());

    provide_context(Dialogs { dialogs });
}

#[component]
pub fn DialogRenderer() -> impl IntoView {
    let Dialogs { dialogs } = use_context::<Dialogs>().expect("Dialogs context not found");

    let top_dialog = move || {
        let mut dialogs = dialogs.get();
        dialogs.pop().map(|(_, d)| d)
    };

    let rest = move || {
        let mut dialogs = dialogs.get();
        dialogs.pop();
        dialogs.into_iter().map(|(_, d)| d).collect::<Vec<_>>()
    };

    move || {
        top_dialog().map(|top_dialog| {
            view! {
                <div class="fixed top-0 left-0 w-full h-full">
                    // display dialogs below the topmost one
                    {move || {
                        rest().into_iter().map(|d|
                            view! {
                                <DialogFrame>
                                    {d.run()}
                                </DialogFrame>
                            }).collect::<Vec<_>>()
                    }}

                    // background
                    <div class="fixed top-0 left-0 w-full h-full bg-black opacity-75" />

                    // topmost dialog
                    <DialogFrame>
                        {top_dialog.run()}
                    </DialogFrame>
                </div>
            }
        })
    }
}

#[component]
fn DialogFrame(children: Children) -> impl IntoView {
    view! {
        <div class="fixed top-1/2 left-1/2 -translate-1/2">
            {children()}
        </div>
    }
}

#[component]
pub fn DialogCloseButton(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    let dialogs = use_context::<Dialogs>().expect("Dialogs context not found");

    let close = move |_| {
        dialogs.close();
    };

    if let Some(children) = children {
        view! {
            <div on:click=close>
                {children()}
            </div>
        }
        .into_any()
    } else {
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer" on:click=close>
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
            </svg>
        }.into_any()
    }
}
