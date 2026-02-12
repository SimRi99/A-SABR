use a_sabr::{
    bundle::Bundle, contact_manager::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan, node_manager::none::NoManagement,
    routing::aliases::*, types::NodeID,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

pub fn benchmark(c: &mut Criterion) {
    let ptvg_filepath = "benches/astar_graphs/1000.json";

    let source = 0;
    let bundle = Bundle {
        source,
        destinations: vec![99],
        priority: 0,
        size: 0.0,
        expiration: 24060.0,
    };
    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];


    let mut router_types = vec![
        "VolCgrHybridParenting",
        "VolCgrNodeParenting",
        "VolCgrHybridParentingOwlt",
        "VolCgrNodeParentingOwlt",
        "VolCgrHybridParentingKLookAheadOwlt",
        "VolCgrNodeParentingKLookAheadOwlt",
    ];

    #[cfg(feature = "contact_work_area")]
    router_types.extend(["VolCgrContactParenting", "VolCgrContactParentingOwlt", "VolCgrContactParentingKLookAheadOwlt"]);


    let mut group = c.benchmark_group("Routers");

    for router_type in router_types {
        group.bench_function(router_type, |b| {
            b.iter_batched(
                || {
                    let (nodes, contacts, distances) = TVGUtilContactPlan::parse::<
                        NoManagement,
                        SegmentationManager,
                    >(ptvg_filepath)
                        .unwrap();

                    build_generic_router(router_type, nodes, contacts, distances, None)
                },
                |mut router| {
                    black_box(router.route(
                        black_box(source),
                        black_box(&bundle),
                        black_box(curr_time),
                        black_box(&excluded_nodes),
                    ));
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group! {
    name=benches;
    config=Criterion::default().sample_size(50);
    targets=benchmark
}
criterion_main!(benches);