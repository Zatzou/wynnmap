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
                <div class="dialogcontainer">
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
                    <div class="dialogbackground" />

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
        <div class="dialogframe">
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
            <div class="cursor-pointer" on:click=close>
                <lucide_leptos::X size=32/>
            </div>
        }
        .into_any()
    }
}
