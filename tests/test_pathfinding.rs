use std::time::Instant;
use criterion::black_box;
use a_sabr::bundle::Bundle;
use a_sabr::contact_manager::seg::SegmentationManager;
use a_sabr::contact_plan::from_tvgutil_file::TVGUtilContactPlan;
use a_sabr::node_manager::none::NoManagement;
use a_sabr::routing::aliases::{build_generic_router, SpsnOptions};
use a_sabr::types::NodeID;

#[test]
fn test_pathfinding_cgr() {
    let ptvg_filepath = "benches/astar_graphs/test_ptvg.json";

    let source = 0;
    let bundle = Bundle {
        source,
        destinations: vec![5],
        priority: 0,
        size: 0.0,
        expiration: 1240600000000.0,
    };
    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];
    let spsn_opts = SpsnOptions {
        check_size: false,
        check_priority: false,
        max_entries: 10,
    };

    let (nodes, contacts, durations, distances_per_time, min_distances) = TVGUtilContactPlan::parse::<
        NoManagement,
        SegmentationManager,
    >(ptvg_filepath, 1.0)
        .unwrap();

    //let router_type = "VolCgrHybridParentingHeuristicGeoDistance";
    //let router_type = "VolCgrHybridParentingGeoDistance";
    //let router_type= "VolCgrHybridParentingKLookAheadOwlt";
    //let router_type= "VolCgrHybridParentingOwlt";
    //let router_type= "VolCgrHybridParenting";
    //let router_type = "VolCgrHybridParentingKLookAheadMinOwlt";
    //let router_type = "VolCgrNodeParenting";
    //let router_type = "VolCgrNodeParentingOwlt";
   // #[cfg(feature = "contact_work_area")]
    //let router_type = "SpsnContactParentingOwlt";
    //let router_type = "SpsnContactParenting";
   // let router_type = "SpsnNodeParentingOwlt";
    //let router_type = "SpsnNodeParenting";
    //let router_type = "VolCgrContactParenting";
    //let router_type = "VolCgrContactParentingOwlt";
    let mut router = build_generic_router(router_type, nodes, contacts, durations, distances_per_time, min_distances, Some(spsn_opts.clone()));
    router.route(
        black_box(source),
        black_box(&bundle),
        black_box(curr_time),
        black_box(&excluded_nodes),
        None
    );
}