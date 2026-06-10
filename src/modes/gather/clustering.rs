use std::collections::{BTreeMap, btree_map::Entry};

use wynnmap_types::gather::GatherSpots;

use crate::modes::gather::noderender::GatherNode;

pub fn cluster_all(data: GatherSpots, max_d: f64, min_r: f64) -> Vec<GatherNode> {
    let mut pos_by_type: BTreeMap<usize, Vec<[i32; 2]>> = BTreeMap::new();

    for spot in data.spots {
        match pos_by_type.entry(spot.resource) {
            Entry::Vacant(entry) => {
                entry.insert(vec![[spot.pos[0], spot.pos[2]]]);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().push([spot.pos[0], spot.pos[2]]);
            }
        }
    }

    let mut clusters = Vec::new();

    for (type_id, nodes) in pos_by_type {
        let mut visited = vec![false; nodes.len()];

        while let Some(&p) = nodes
            .iter()
            .zip(visited.iter_mut())
            .filter(|(_, v)| !**v)
            .map(|(p, v)| {
                *v = !*v;
                p
            })
            .next()
        {
            let cluster = cluster(p, &nodes, &mut visited, max_d);

            let mid = cluster_midpoint(&cluster);
            let r = cluster_size(&cluster);

            clusters.push(GatherNode {
                pos: mid,
                radius: r.max(min_r),

                count: cluster.len(),
                res: data.resources[type_id].clone(),
            });
        }
    }

    clusters.sort_by(|a, b| {
        b.radius
            .partial_cmp(&a.radius)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    clusters
}

fn cluster(
    start: [i32; 2],
    points: &[[i32; 2]],
    visited: &mut [bool],
    max_d: f64,
) -> Vec<[i32; 2]> {
    let mut set = Vec::with_capacity(points.len());
    set.push(start);

    if max_d < 10.0 {
        return set;
    }

    let mut unexplored = vec![start];

    while let Some(pos) = unexplored.pop() {
        let included: Vec<_> = points
            .iter()
            .zip(visited.iter_mut())
            .filter(|(_, v)| !**v)
            .filter(|(p, _)| xz_dist_to(pos, **p) <= max_d)
            .map(|(p, v)| {
                *v = !*v;
                p
            })
            .collect();

        for p in included {
            set.push(*p);
            unexplored.push(*p);
        }
    }

    set
}

fn xz_dist_to(lhs: [i32; 2], rhs: [i32; 2]) -> f64 {
    let dist_x = lhs[0].abs_diff(rhs[0]);
    let dist_z = lhs[1].abs_diff(rhs[1]);

    f64::from(dist_x.pow(2) + dist_z.pow(2)).sqrt()
}

fn cluster_midpoint(cluster: &[[i32; 2]]) -> [i32; 2] {
    let count = cluster.len() as i32;

    let x = cluster.iter().map(|[x, _]| *x).sum::<i32>() / count;
    let z = cluster.iter().map(|[_, z]| *z).sum::<i32>() / count;

    [x, z]
}

fn cluster_size(cluster: &[[i32; 2]]) -> f64 {
    let left_side = cluster.iter().map(|[x, _]| *x).min().unwrap_or_default();
    let right_side = cluster.iter().map(|[x, _]| *x).max().unwrap_or_default();
    let bottom_side = cluster.iter().map(|[_, z]| *z).min().unwrap_or_default();
    let top_side = cluster.iter().map(|[_, z]| *z).max().unwrap_or_default();

    let width = f64::from(left_side.abs_diff(right_side));
    let height = f64::from(bottom_side.abs_diff(top_side));

    f64::max(f64::max(width / 2.0, height / 2.0), 2.0)
}
