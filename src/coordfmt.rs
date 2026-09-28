use std::fmt::Display;

use leptos::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum CoordFmt {
    #[default]
    CompassShort,
    CompassLong,
    Raw,
    Commas,
    Lettered,
}

impl CoordFmt {
    pub fn format_xz(&self, pos: [impl Display; 2]) -> String {
        let [x, z] = pos;
        match self {
            CoordFmt::CompassShort => format!("/comp {x} {z}"),
            CoordFmt::CompassLong => format!("/compass at {x} 0 {z}"),
            CoordFmt::Raw => format!("{x} {z}"),
            CoordFmt::Commas => format!("{x}, {z}"),
            CoordFmt::Lettered => format!("X{x} Z{z}"),
        }
    }

    pub fn format_xyz(&self, pos: [impl Display; 3]) -> String {
        let [x, y, z] = pos;
        match self {
            CoordFmt::CompassShort => format!("/comp {x} {y} {z}"),
            CoordFmt::CompassLong => format!("/compass at {x} {y} {z}"),
            CoordFmt::Raw => format!("{x} {y} {z}"),
            CoordFmt::Commas => format!("{x}, {y}, {z}"),
            CoordFmt::Lettered => format!("X{x} Y{y} Z{z}"),
        }
    }
}

#[component]
pub fn CoordFmtSelector(fmt: RwSignal<CoordFmt>) -> impl IntoView {
    view! {
        <select
            class="select"
            on:change:target=move |ev| {
                fmt.set(match ev.target().value().as_str() {
                    "CompassShort" => CoordFmt::CompassShort,
                    "CompassLong" => CoordFmt::CompassLong,
                    "Raw" => CoordFmt::Raw,
                    "Commas" => CoordFmt::Commas,
                    "Lettered" => CoordFmt::Lettered,
                    _ => unreachable!()
                })
            }
            prop:value=move || match fmt.get() {
                CoordFmt::CompassShort => "CompassShort",
                CoordFmt::CompassLong => "CompassLong",
                CoordFmt::Raw => todo!(),
                CoordFmt::Commas => todo!(),
                CoordFmt::Lettered => todo!(),
            }
        >
            <option value="CompassShort">"/comp {x} (y) {z}"</option>
            <option value="CompassLong">"/compass at {x} {y} {z}"</option>
            <option value="Raw">"{x} (y) {z}"</option>
            <option value="Commas">"{x}, {y}, {z}"</option>
            <option value="Lettered">"X{x} Y{y} Z{z}"</option>
        </select>
    }
}
