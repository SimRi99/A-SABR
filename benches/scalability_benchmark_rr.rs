use a_sabr::{
    bundle::Bundle, contact_manager::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan, node_manager::none::NoManagement,
    routing::aliases::*, types::NodeID,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::fs;

pub fn benchmark(c: &mut Criterion) {
    let percentages = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    let source: NodeID = 0;
    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];
    let spsn_opts = SpsnOptions {
        check_size: false,
        check_priority: false,
        max_entries: 10,
    };

    let router_types: Vec<(&str, &str)> = vec![
        ("VolCgrNodeParenting", "VolCgrNode"),
        ("VolCgrHybridParenting", "VolCgrHybrid"),
        #[cfg(feature = "contact_work_area")]
        ("VolCgrContactParenting", "VolCgrContact"),
        ("VolCgrNodeParentingOwlt", "VolCgr*Node"),
        ("VolCgrHybridParentingOwlt", "VolCgr*Hybrid"),
        #[cfg(feature = "contact_work_area")]
        ("VolCgrContactParentingOwlt", "VolCgr*Contact"),
    ];

    for pct in &percentages {
        let ptvg_filepath = format!("benches/astar_graphs/scalability/40_ipn/{}.json", pct);

        let (nodes_check, _, _, _, _) = TVGUtilContactPlan::parse::<
            NoManagement,
            SegmentationManager,
        >(&ptvg_filepath, 1.0)
        .unwrap();
        let num_nodes = nodes_check.len() as NodeID;
        let destinations: Vec<NodeID> = (1..num_nodes).collect();

        let results_base = format!("benches/results_scalability/40_ipn/{}", pct);
        fs::create_dir_all(&results_base).ok();

        let group_name = format!("40_ipn_{}", pct);
        let mut group = c.benchmark_group(&group_name);

        for (router_type, router_name) in &router_types {
            for destination in &destinations {
                let bundle = Bundle {
                    source,
                    destinations: vec![*destination],
                    priority: 0,
                    size: 0.0,
                    expiration: 24060000.0,
                };
                let experiment_name =
                    format!("{}{}", router_name, destination);
                let filepath = ptvg_filepath.clone();
                let results_path = results_base.clone();
                let rn = router_name.to_string();

                group.bench_function(&experiment_name, |b| {
                    b.iter_batched(
                        || {
                            let (nodes, contacts, distances, distances_per_time, min_distances) =
                                TVGUtilContactPlan::parse::<NoManagement, SegmentationManager>(
                                    &filepath, 1.0,
                                )
                                .unwrap();
                            build_generic_router(
                                router_type,
                                nodes,
                                contacts,
                                distances,
                                distances_per_time,
                                min_distances,
                                Some(spsn_opts.clone()),
                            )
                        },
                        |mut router| {
                            black_box(router.route(
                                black_box(source),
                                black_box(&bundle),
                                black_box(curr_time),
                                black_box(&excluded_nodes),
                                Some(format!("{}/{}.txt", results_path, rn)),
                            ));
                        },
                        BatchSize::SmallInput,
                    );
                });
            }
        }
        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark
}
criterion_main!(benches);
