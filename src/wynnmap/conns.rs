use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use leptos::prelude::*;
use wynnmap_types::{ExTerrInfo, Territory};

#[component]
pub fn Connections(
    #[prop(into)] terrs: Signal<HashMap<Arc<str>, Territory>>,
    extradata: impl Fn() -> HashMap<Arc<str>, ExTerrInfo> + Send + Sync + 'static,
) -> impl IntoView {
    let conn_path = move || create_route_paths(&*terrs.read(), extradata());

    view! {
        <svg style="position: absolute;overflow: visible;contain: layout;" >
            <path
                id="connpath"
                d={move || conn_path()}
                style="fill:none;"
                stroke-linecap="round"
            />
            <g inner_html=
            {
                "
                <use
                    href=\"#connpath\"
                    style=\"stroke:black;stroke-width:4;\"
                />
                <use
                    href=\"#connpath\"
                    style=\"stroke:white;stroke-width:2;\"
                />
                "
            }/>
        </svg>
    }
}

pub fn create_route_paths(
    terrs: &HashMap<Arc<str>, Territory>,
    extradata: HashMap<Arc<str>, ExTerrInfo>,
) -> String {
    let terr_mid_coords: HashMap<Arc<str>, (f64, f64)> = terrs
        .iter()
        .map(|(k, v)| (k.clone(), v.location.get_midpoint()))
        .collect();

    let mut terr_conns: HashSet<(Arc<str>, Arc<str>)> = HashSet::new();
    for (orig, v) in extradata {
        for conn in v.conns {
            if orig < conn {
                terr_conns.insert((orig.clone(), conn));
            } else {
                terr_conns.insert((conn, orig.clone()));
            }
        }
    }

    let mut pathing = String::new();
    for (start, end) in terr_conns {
        let coords_start = terr_mid_coords.get(&start).unwrap_or(&(0.0, 0.0));
        let coords_end = terr_mid_coords.get(&end).unwrap_or(&(0.0, 0.0));

        pathing.push_str(&format!(
            "M{} {} L{} {} ",
            coords_start.0,
            coords_start.1, // x and y of starting point
            coords_end.0,
            coords_end.1 // x and y of ending point
        ));
    }

    pathing
}
