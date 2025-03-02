use std::collections::{HashMap, HashSet};
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{Get, Memo};
use crate::types::{ExTerrInfo, Territory};

pub fn create_route_paths(terrs: Memo<HashMap<String, Territory>>, extradata: HashMap<String, ExTerrInfo>) -> String {
    let terrs = terrs.get();
    let terr_mid_coords = terr_mid_coords(terrs);
    let terr_conns = terr_conns(extradata);

    let mut pathing = String::new();
    for i in terr_conns {
        let coords_start = terr_mid_coords.get(&i.0).unwrap_or(&(0.0,0.0));
        let coords_end = terr_mid_coords.get(&i.1).unwrap_or(&(0.0,0.0));
        pathing = pathing + &line_draw(*coords_start, *coords_end);
    }
    pathing
}

fn terr_mid_coords(terrs: HashMap<String, Territory>) -> HashMap<String, (f64,f64)> {
    let mut out: HashMap<String, (f64,f64)> = HashMap::new();
    for i in terrs {
        let terr_name = i.0;
        let coords_mid = i.1.get_midpoint();
        out.insert(terr_name, coords_mid);
    }
    out
}

fn terr_conns(extradata: HashMap<String, ExTerrInfo>) -> HashSet<(String, String)> {
    let mut out: HashSet<(String, String)> = HashSet::new();
    let _ = extradata.iter().map(
        |(ke,va)| {
            for b in va.clone().conns.unwrap_or(Vec::from([])) {

                let a = ke;
                // a = orig terr
                // b = conn terr
                if a.clone() < b {
                    out.insert((a.clone(),b));
                } else {
                    out.insert((b, a.clone()));
                }

            }
        }
    );
    out
}

fn line_draw(start: (f64, f64), end: (f64, f64)) -> String {
    format!("M{} {} L{} {} ", start.0, start.1, end.0, end.1)
}