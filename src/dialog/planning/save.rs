use std::{collections::HashMap, path::PathBuf, sync::Arc};

use leptos::prelude::*;
use web_sys::{
    HtmlAnchorElement, HtmlInputElement,
    js_sys::{ArrayBuffer, Uint8Array},
    wasm_bindgen::{JsCast, JsValue, prelude::Closure},
};
use wynnmap_types::Guild;

use crate::dialog::{DialogCloseButton, planning::formats};

#[derive(Clone, Debug, PartialEq, Eq)]
enum FileFormat {
    /// Wynnmap format
    Wynnmap = 1,
    /// fa-rog.github.io/economy format
    Farog = 2,
    /// Ruea economy studio format
    RueaES = 3,
}

pub fn save_dialog(
    terrs: Signal<HashMap<Arc<str>, wynnmap_types::Territory>>,
    guilds: RwSignal<Vec<ArcRwSignal<Guild>>>,
    owned: RwSignal<HashMap<Arc<str>, ArcRwSignal<Guild>>>,
) -> impl IntoView {
    let filename = RwSignal::new(String::new());
    let fileformat = RwSignal::new(FileFormat::Wynnmap);
    let sharestring = Memo::new(move |_| {
        let data =
            formats::urlshare::WynnmapData::from_data(&terrs.read(), &guilds.read(), &owned.read());

        let encoded = data.to_string();

        let location = window().location();

        let out = format!(
            "{}{}{}#{}",
            location.origin().unwrap_or_default(),
            location.pathname().unwrap_or_default(),
            location.search().unwrap_or_default(),
            encoded
        );

        out
    });

    let copystring = move |_| {
        let clipboard = window().navigator().clipboard();

        let _promise = clipboard.write_text(&sharestring.get());
    };

    let formatselect = move |e: String| {
        fileformat.update(|ff| match e.as_ref() {
            "1" => *ff = FileFormat::Wynnmap,
            "2" => *ff = FileFormat::Farog,
            "3" => *ff = FileFormat::RueaES,
            _ => {}
        });
    };

    let downloadbtn = move |_| {
        let bytes = match fileformat.get() {
            FileFormat::Wynnmap => formats::wynnmap::WynnmapData::from_data(
                &terrs.read(),
                &guilds.read(),
                &owned.read(),
            )
            .into_bytes(),
            FileFormat::Farog => todo!(),
            FileFormat::RueaES => todo!(),
        };

        // let jsbytes = JsValue::from(bytes);
        // let blob = Blob::new_with_u8_array_sequence(&jsbytes).unwrap();
        let blob = gloo_file::Blob::new_with_options(bytes.as_slice(), Some("application/json"));
        let url = gloo_file::ObjectUrl::from(blob);

        let doc = document();
        let href: HtmlAnchorElement = doc.create_element("a").unwrap().dyn_into().unwrap();
        href.set_download(&format!(
            "{}.{}",
            filename.get(),
            match fileformat.get() {
                FileFormat::Wynnmap => "wynnmap",
                FileFormat::Farog => todo!(),
                FileFormat::RueaES => todo!(),
            },
        ));
        href.set_href(&url);
        href.click();
        href.remove();
    };

    let file_load_error = RwSignal::new(None);

    let loadfile = move |e: leptos::ev::Event| {
        let input: HtmlInputElement = e.target().unwrap().unchecked_into();

        if let Some(file) = input.files().and_then(|f| f.get(0)) {
            let name = PathBuf::from(file.name());

            match name.extension().and_then(|s| s.to_str()) {
                Some("wynnmap") => {
                    let parse_wynnmap = RwSignal::new_local(Closure::new(move |ab: JsValue| {
                        let array_buffer = ab.dyn_into::<ArrayBuffer>().unwrap();
                        let bytes = Uint8Array::new(&array_buffer).to_vec();

                        let data = formats::wynnmap::WynnmapData::from_bytes(bytes);

                        let (gu, ow) = data.into_data();

                        guilds.set(gu);
                        owned.set(ow);
                    }));

                    _ = file.array_buffer().then(&parse_wynnmap.read());
                }

                _ => file_load_error.set(Some("Unknown file extension")),
            }
        }
    };

    view! {
        <div class="bg-neutral-900 md:rounded-xl text-white w-screen max-w-3xl h-dvh md:max-h-150 flex flex-col">
            <div class="flex justify-between p-2 items-center">
                <h1 class="text-4xl">"Import/Export"</h1>

                <DialogCloseButton />
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                <h2 class="text-xl">"Share URL"</h2>

                <div class="flex p-1">
                    <input type="text" class="grow border-1 border-neutral-600 p-1 px-2 rounded-l-lg" value={sharestring} readonly onfocus="this.select()"/>
                    <button class="border-1 border-l-0 border-neutral-600 p-1 px-2 rounded-r-lg hover:bg-neutral-700" on:click={copystring}>
                        "Copy"
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6 inline-block ml-1">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
                        </svg>
                    </button>
                </div>

                <p class="text-neutral-400">
                    "Note: sharing by URL only retains the minimum data to send the current territory plan. Additional information is not saved."
                </p>
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                <h2 class="text-xl">"Save to file"</h2>

                <div class="mb-2">
                    <p class="inline-block">"Format:"</p>
                    <select class="border-1 border-neutral-600 p-1 px-2 rounded-lg ml-1" on:input:target={move |e| formatselect(e.target().value()) }>
                        <option value=1 selected={move || *fileformat.read() == FileFormat::Wynnmap}>"Wynnmap"</option>
                        // <option value=2 selected={move || *fileformat.read() == FileFormat::Farog}>"fa-rog's economy simulator"</option>
                    </select>
                </div>

                <div class="flex">
                    <input type="text" class="border-1 border-r-0 border-neutral-600 p-1 pl-2 rounded-l-lg" placeholder="filename" bind:value={filename}/>
                    <span class="border-1 border-l-0 border-neutral-600 p-1 pr-2 text-neutral-400">
                        {move || match fileformat.get() {
                            FileFormat::Wynnmap => ".wynnmap",
                            FileFormat::Farog | FileFormat::RueaES => ".json"
                        }}
                    </span>
                    <button class="border-1 border-l-0 border-neutral-600 p-1 px-2 rounded-r-lg hover:bg-neutral-700" on:click={downloadbtn}>"Download"</button>
                </div>
            </div>

            <hr class="border-neutral-600" />

            <div class="p-2">
                <h2 class="text-xl">"Load from file"</h2>

                <input type="file" class="border-1 border-neutral-600 p-1 px-2 rounded-l" on:input={loadfile}/>

                <p class="text-neutral-400">
                    "Supported filetypes: Wynnmap"
                </p>
            </div>
        </div>
    }
}
