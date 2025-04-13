use leptos::prelude::*;

pub mod settings;

#[derive(Clone)]
pub struct Dialogs(pub RwSignal<Vec<ViewFn>>);

pub fn provide_dialogs() {
    let dialogs = RwSignal::new(Vec::new());

    provide_context(Dialogs(dialogs));
}

#[component]
pub fn DialogRenderer() -> impl IntoView {
    let Dialogs(dialogs) = use_context::<Dialogs>().expect("Dialogs context not found");

    let top_dialog = move || {
        let mut dialogs = dialogs.get();
        dialogs.reverse();
        dialogs.pop()
    };

    let rest = move || {
        let mut dialogs = dialogs.get();
        dialogs.reverse();
        dialogs.pop();
        dialogs
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
pub fn DialogCloseButton() -> impl IntoView {
    let Dialogs(dialogs) = use_context::<Dialogs>().expect("Dialogs context not found");

    let close = move |_| {
        close_dialog(dialogs);
    };

    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 cursor-pointer" on:click=close>
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
        </svg>
    }
}

pub fn show_dialog<F: Into<ViewFn>>(dialog: F) {
    let Dialogs(dialogs) = use_context::<Dialogs>().expect("Dialogs context not found");

    dialogs.update(|d| {
        d.push(dialog.into());
    });
}

pub fn close_dialog(dialogs: RwSignal<Vec<ViewFn>>) {
    dialogs.update(|d| {
        d.pop();
    });
}
