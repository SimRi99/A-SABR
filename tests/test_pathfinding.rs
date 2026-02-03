use criterion::black_box;
use a_sabr::bundle::Bundle;
use a_sabr::contact_manager::seg::SegmentationManager;
use a_sabr::contact_plan::from_tvgutil_file::TVGUtilContactPlan;
use a_sabr::node_manager::none::NoManagement;
use a_sabr::routing::aliases::{build_generic_router, SpsnOptions};
use a_sabr::types::NodeID;

#[test]
fn test_pathfinding_spsn() {
    let ptvg_filepath = "benches/ptvg_files/sample1.json";

    let source = 178;
    let bundle = Bundle {
        source: 178,
        destinations: vec![159],
        priority: 0,
        size: 47419533.0,
        expiration: 24060.0,
    };
    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];
    let spsn_opts = SpsnOptions {
        check_size: false,
        check_priority: false,
        max_entries: 10,
    };

    let (nodes, contacts, durations) = TVGUtilContactPlan::parse::<
        NoManagement,
        SegmentationManager,
    >(ptvg_filepath)
        .unwrap();

    let router_type= "SpsnHybridParentingKLookAheadOwlt";
    let mut router = build_generic_router(router_type, nodes, contacts, durations, Some(spsn_opts.clone()));
    router.route(
        black_box(source),
        black_box(&bundle),
        black_box(curr_time),
        black_box(&excluded_nodes),
    );
}