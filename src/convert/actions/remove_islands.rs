use crate::convert::{Config, LockFile};
use crate::Database;
use anyhow::Result;
use std::collections::HashSet;

fn get_size(graph: &HashSet<(u64, u64)>) -> (u64, u64) {
    let first = graph.iter().next().unwrap();
    let mut max = first.clone();
    let mut min = first.clone();
    for node in graph {
        let (x, y) = node;
        if *x > max.0 {
            max.0 = *x;
        }
        if *x < min.0 {
            min.0 = *x;
        }
        if *y > max.1 {
            max.1 = *y;
        }
        if *y < min.1 {
            min.1 = *y;
        }
    }
    let size = (max.0 - min.0 + 1, max.1 - min.1 + 1);
    size
}

fn find_connected_subgraphs(
    mut graph: HashSet<(u64, u64)>,
    lock: &mut LockFile,
) -> Result<Vec<HashSet<(u64, u64)>>> {
    let mut subgraphs = Vec::new();
    lock.state("Island removal")?;
    lock.start(graph.len() as u64)?;

    while !graph.is_empty() {
        let first = graph.iter().next().unwrap();
        let mut stack = vec![first.clone()];

        let mut subgraph = HashSet::new();
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            match graph.take(&node) {
                Some(n) => {
                    lock.inc()?;
                    subgraph.insert(n);
                    let (ux, uy) = n;
                    let x = ux as i64;
                    let y = uy as i64;
                    let neighbors = vec![(x, y + 1), (x + 1, y), (x, y - 1), (x - 1, y)];
                    for n in neighbors {
                        if n.0 < 0 || n.1 < 0 {
                            continue;
                        }
                        stack.push((n.0 as u64, n.1 as u64));
                    }
                }
                None => {}
            };
        }
        subgraphs.push(subgraph);
    }
    lock.finish()?;
    Ok(subgraphs)
}

pub fn remove_islands(db: &Database, c: &Config, lock: &mut LockFile) -> Result<()> {
    let level = db.levels() - 1;

    let tiles = db.list_tiles(level)?;
    let graph: HashSet<(u64, u64)> = tiles.into_iter().collect();
    let subgraphs = find_connected_subgraphs(graph, lock)?;

    for graph in subgraphs {
        let size = get_size(&graph);
        let n = graph.len();
        if n as u64 > c.island_tiles {
            continue;
        }
        if size.0 < c.island_size || size.1 < c.island_size {
            log::debug!("Removing island {}x{} {} tiles", size.0, size.1, n,);
            for node in &graph {
                db.delete(node.clone(), level)?;
            }
        }
    }
    Ok(())
}
