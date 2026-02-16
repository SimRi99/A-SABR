use a_sabr::{
    bundle::Bundle, contact_manager::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan, node_manager::none::NoManagement,
    routing::aliases::*, types::NodeID,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

pub fn benchmark(c: &mut Criterion) {
    let ptvg_filepath = "benches/data.json";

    let source = 0;
    let bundle = Bundle {
        source,
        destinations: vec![189],
        priority: 0,
        size: 0.0,
        expiration: 24060.0,
    };
    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];
    let spsn_opts = SpsnOptions {
        check_size: false,
        check_priority: false,
        max_entries: 10,
    };


    let mut router_types = vec![
        ("VolCgrHybridParenting", "VolCgrHybrid"),
        ("VolCgrNodeParenting", "VolCgrNode"),
        ("VolCgrHybridParentingOwlt", "VolCgr*Hybrid"),
        ("VolCgrNodeParentingOwlt", "VolCgr*Node"),
        ("SpsnHybridParenting", "SpsnHybrid"),
        ("SpsnNodeParenting", "SpsnNode"),
        ("SpsnHybridParentingOwlt", "Spsn*Hybrid"),
        ("SpsnNodeParentingOwlt", "Spsn*Node")
    ];

    #[cfg(feature = "contact_work_area")]
    router_types.extend([("VolCgrContactParenting", "VolCgrContact"), ("VolCgrContactParentingOwlt", "VolCgr*Contact"), ("SpsnContactParenting", "SpsnContact"), ("SpsnContactParentingOwlt", "Spsn*Contact")]);


    let mut group = c.benchmark_group("Routers");

    for (router_type, router_name) in router_types {
        group.bench_function(router_name, |b| {
            b.iter_batched(
                || {
                    let (nodes, contacts, distances) = TVGUtilContactPlan::parse::<
                        NoManagement,
                        SegmentationManager,
                    >(ptvg_filepath)
                        .unwrap();

                    build_generic_router(router_type, nodes, contacts, distances, Some(spsn_opts.clone()))
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