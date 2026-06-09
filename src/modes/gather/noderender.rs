use std::{collections::BTreeMap, sync::Arc};

use leptos::prelude::*;
use wynnmap_types::gather::{GatherSpots, MatData, Material};

use crate::{
    modes::gather::clustering::cluster_all,
    wynnmap::{MapZoom, RelMousePos},
};

#[derive(Clone, PartialEq)]
pub struct GatherNode {
    pub pos: [i32; 2],
    pub radius: f64,

    pub count: usize,
    pub res: Material,
}

impl GatherNode {
    fn contains(&self, point: [i32; 2]) -> bool {
        let dist_x = self.pos[0].abs_diff(point[0]);
        let dist_z = self.pos[1].abs_diff(point[1]);

        let dist = f64::sqrt((dist_x.pow(2) + dist_z.pow(2)) as f64);

        dist <= self.radius
    }
}

#[component]
pub fn NodeRenderer(
    nodes: RwSignal<GatherSpots>,
    data: RwSignal<BTreeMap<Arc<str>, MatData>>,
    mouse_pos: RwSignal<Option<[i32; 2]>>,
    hovered: RwSignal<Vec<GatherNode>>,
) -> impl IntoView {
    let RelMousePos(mouse_rel) = expect_context::<RelMousePos>();
    let MapZoom(zoom) = expect_context();

    let clusters_far = Memo::new(move |_| cluster_all(nodes.get(), 20.0, 10.0));
    let clusters_mid = Memo::new(move |_| cluster_all(nodes.get(), 10.0, 6.0));
    let clusters_near = Memo::new(move |_| cluster_all(nodes.get(), 0.0, 2.0));

    let clusters = Memo::new(move |_| {
        let zoom = zoom.get();

        if zoom <= 1.0 {
            1
        } else if zoom <= 2.0 {
            2
        } else {
            3
        }
    });

    Effect::new(move || {
        mouse_pos.set(mouse_rel.get());

        let hov = if let Some(pos) = mouse_rel.get() {
            match clusters.get() {
                1 => clusters_far
                    .read()
                    .iter()
                    .filter(|n| n.contains(pos))
                    .cloned()
                    .collect(),
                2 => clusters_mid
                    .read()
                    .iter()
                    .filter(|n| n.contains(pos))
                    .cloned()
                    .collect(),
                3 => clusters_near
                    .read()
                    .iter()
                    .filter(|n| n.contains(pos))
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        hovered.set(hov);
    });

    view! {
        <svg style="position: absolute; overflow: visible" class:hidden=move || clusters.get() != 1>
            {move || clusters_far.get().into_iter().map(|cluster| {
                let matdata = data.read().get(&cluster.res.name).cloned().unwrap_or_default();
                view!{
                    <circle cx={cluster.pos[0]} cy={cluster.pos[1]} r={cluster.radius} fill=matdata.color.clone() stroke=matdata.prof.color() stroke-width=5 />
                }
            }).collect::<Vec<_>>()}
        </svg>
        <svg style="position: absolute; overflow: visible" class:hidden=move || clusters.get() != 2>
            {move || clusters_mid.get().into_iter().map(|cluster| {
                let matdata = data.read().get(&cluster.res.name).cloned().unwrap_or_default();
                view!{
                    <circle cx={cluster.pos[0]} cy={cluster.pos[1]} r={cluster.radius} fill=matdata.color.clone() stroke=matdata.prof.color() stroke-width=2.5 />
                }
            }).collect::<Vec<_>>()}
        </svg>
        <svg style="position: absolute; overflow: visible" class:hidden=move || clusters.get() != 3>
            {move || clusters_near.get().into_iter().map(|cluster| {
                let matdata = data.read().get(&cluster.res.name).cloned().unwrap_or_default();
                view!{
                    <circle cx={cluster.pos[0]} cy={cluster.pos[1]} r={cluster.radius} fill=matdata.color.clone() stroke=matdata.prof.color() stroke-width=1 />
                }
            }).collect::<Vec<_>>()}
        </svg>
    }
}
