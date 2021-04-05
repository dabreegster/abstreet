//! Structures related to the new road-based pathfinding
//! (https://github.com/a-b-street/abstreet/issues/555) live here. When the transition is done,
//! things here will probably move into pathfind/mod.rs.

use anyhow::Result;

use crate::{DirectedRoadID, Map, Path, PathRequest, PathStep, TurnID};

/// Transform a sequence of roads representing a path into the current lane-based path, by picking
/// particular lanes and turns to use.
pub fn path_v2_to_v1(req: PathRequest, road_steps: Vec<DirectedRoadID>, map: &Map) -> Result<Path> {
    // This is a somewhat brute-force method: run Dijkstra's algorithm on a graph of lanes and
    // turns, but only build the graph along the path of roads we've already found. This handles
    // arbitrary lookahead needed, and forces use of the original start/end lanes requested.
    //
    // Eventually we'll directly return road-based paths. Most callers will actually just use that
    // directly, and mainly the simulation will need to expand to specific lanes, but it'll do so
    // dynamically/lazily to account for current traffic conditions.
    let mut graph = petgraph::graphmap::DiGraphMap::new();
    for pair in road_steps.windows(2) {
        for src in pair[0].lanes(req.constraints, map) {
            for dst in pair[1].lanes(req.constraints, map) {
                let turn = TurnID {
                    parent: map.get_l(src).dst_i,
                    src,
                    dst,
                };
                if map.maybe_get_t(turn).is_some() {
                    graph.add_edge(src, dst, turn);
                }
            }
        }
    }

    match petgraph::algo::astar(
        &graph,
        req.start.lane(),
        |l| l == req.end.lane(),
        // TODO We could include the old lane-changing penalties here, but I'm not sure it's worth
        // the complication. The simulation layer will end up tuning those anyway.
        |_| 1,
        |_| 0,
    ) {
        Some((_, path)) => {
            let mut steps = Vec::new();
            for pair in path.windows(2) {
                steps.push(PathStep::Lane(pair[0]));
                // We don't need to look for this turn in the map; we know it exists.
                steps.push(PathStep::Turn(TurnID {
                    parent: map.get_l(pair[0]).dst_i,
                    src: pair[0],
                    dst: pair[1],
                }));
            }
            steps.push(PathStep::Lane(req.end.lane()));
            assert_eq!(steps[0], PathStep::Lane(req.start.lane()));
            // TODO No uber-turns yet!
            Ok(Path::new(map, steps, req, Vec::new()))
        }
        None => bail!(
            "path_v2_to_v1 found road-based path, but not a lane-based path matching it for {}",
            req
        ),
    }
}

// TODO This is an attempt that looks at windows of 2 roads at a time to pick particular lanes and
// turns. It doesn't work in most cases with multiple lane choices -- I think we need at least a
// window of 3 roads. I'll write that function in the future, for the simulation layer to use
// "lazily".
fn _broken_path_v2_to_v1(
    req: PathRequest,
    mut road_steps: Vec<DirectedRoadID>,
    map: &Map,
) -> Result<Path> {
    let mut path_steps = Vec::new();

    // Pick the starting lane.
    {
        let lanes = road_steps.remove(0).lanes(req.constraints, map);
        // TODO During the transition, try to use the original requested start lane. Relax this
        // later to produce more realistic paths!
        if !lanes.contains(&req.start.lane()) {
            bail!(
                "path_v2_to_v1 found a case where we can't start at the requested lane: {}",
                req
            );
        }
        path_steps.push(PathStep::Lane(req.start.lane()));
    }
    let last_road = map.get_l(req.end.lane()).get_directed_parent(map);

    for road in road_steps {
        let prev_lane = if let Some(PathStep::Lane(l)) = path_steps.last() {
            *l
        } else {
            unreachable!()
        };

        let mut current_lanes = road.lanes(req.constraints, map);
        // Filter current_lanes based on available turns.
        let parent = map.get_l(prev_lane).dst_i;
        current_lanes.retain(|dst| {
            map.maybe_get_t(TurnID {
                parent,
                src: prev_lane,
                dst: *dst,
            })
            .is_some()
        });
        if current_lanes.is_empty() {
            error!("Lookahead failed. Req: {}", req);
            error!("Path so far:");
            for x in &path_steps {
                error!("- {:?}", x);
            }

            bail!(
                "path_v2_to_v1 found a case where lookahead failed at {}: {}",
                parent,
                req
            );
        }
        if road == last_road {
            current_lanes.retain(|l| *l == req.end.lane());
        }

        // TODO We could include the old lane-changing penalties here, but I'm not sure it's worth
        // the complication. The simulation layer will end up tuning those anyway.
        let next_lane = current_lanes[0];
        path_steps.push(PathStep::Turn(TurnID {
            parent,
            src: prev_lane,
            dst: next_lane,
        }));
        path_steps.push(PathStep::Lane(next_lane));
    }

    // Sanity check we end in the right place.
    assert_eq!(
        Some(PathStep::Lane(req.end.lane())),
        path_steps.last().cloned()
    );
    // TODO No uber-turns yet!
    Ok(Path::new(map, path_steps, req, Vec::new()))
}
