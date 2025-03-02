use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use wynnmap_types::{ExTerrInfo, Territory};

pub fn create_route_paths(
    terrs: HashMap<Arc<str>, Territory>,
    extradata: HashMap<Arc<str>, ExTerrInfo>,
) -> String {
    let mut terr_mid_coords: HashMap<Arc<str>, (f64, f64)> = HashMap::new();
    for i in terrs {
        let terr_name = i.0;
        let coords_mid = i.1.location.get_midpoint();
        terr_mid_coords.insert(terr_name, coords_mid);
    }

    let mut terr_conns: HashSet<(Arc<str>, Arc<str>)> = HashSet::new();
    for (ke, va) in &extradata {
        let a = ke;
        for b in va.clone().conns {
            // a = orig terr
            // b = conn terr
            if a.clone() < b {
                terr_conns.insert((a.clone(), b));
            } else {
                terr_conns.insert((b, a.clone()));
            }
        }
    }

    let mut pathing = String::new();
    for i in terr_conns {
        let coords_start = terr_mid_coords.get(&i.0).unwrap_or(&(0.0, 0.0));
        let coords_end = terr_mid_coords.get(&i.1).unwrap_or(&(0.0, 0.0));
        pathing = pathing
            + &format!(
                "M{} {} L{} {} ",
                coords_start.0,
                coords_start.1, // x and y of starting point
                coords_end.0,
                coords_end.1 // x and y of ending point
            );
    }
    pathing
}
