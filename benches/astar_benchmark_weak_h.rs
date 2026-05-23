use a_sabr::{
    bundle::Bundle, contact_manager::seg::SegmentationManager,
    contact_plan::from_tvgutil_file::TVGUtilContactPlan, node_manager::none::NoManagement,
    routing::aliases::*, types::NodeID,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::fs;
use std::iter::FromIterator;

pub fn benchmark(c: &mut Criterion) {
    let ptvg_filepath = "benches/astar_graphs/1week.json";

    let source = 0;
    let bundle = Bundle {
        source,
        destinations: vec![2], // Fac_Ariel
        priority: 0,
        size: 0.0,
        expiration: 24060000.0,
    };
    let distance_modifiers = [0.0, 0.1]; // 0.2, 0.4, 0.6, 0.8, 1.0];

    let curr_time = 60.0;
    let excluded_nodes: Vec<NodeID> = vec![];
    let spsn_opts = SpsnOptions {
        check_size: false,
        check_priority: false,
        max_entries: 10,
    };

    let mut router_types = vec![
       // ("VolCgrHybridParenting", "VolCgrHybrid"),
        //("VolCgrNodeParenting", "VolCgrNode"),
        #[cfg(feature = "contact_work_area")]
        //("VolCgrContactParenting", "VolCgrContact"),
        ("VolCgrHybridParentingOwlt", "VolCgr*Hybrid"),
        ("VolCgrNodeParentingOwlt", "VolCgr*Node"),
        #[cfg(feature = "contact_work_area")]
        ("VolCgrContactParentingOwlt", "VolCgr*Contact")
    //    ("SpsnHybridParenting", "SpsnHybrid"),
      //  ("SpsnNodeParenting", "SpsnNode"),
      //  ("SpsnHybridParentingOwlt", "Spsn*Hybrid"),
       // ("SpsnNodeParentingOwlt", "Spsn*Node")
    ];

    let mut group = c.benchmark_group("RouterDestinations");

    for (router_type, router_name) in router_types {
        for modifier in distance_modifiers.iter() {
            let experiment_name = router_name.to_owned() + &modifier.to_string();
            group.bench_function(&experiment_name, |b| {
                b.iter_batched(
                    || {
                        let (nodes, contacts, distances, distances_per_time, min_distances) = TVGUtilContactPlan::parse::<
                            NoManagement,
                            SegmentationManager,
                        >(ptvg_filepath, *modifier)
                            .unwrap();

                        fs::create_dir(format!("benches/results_weak_h/{}", &modifier.to_string()));
                        build_generic_router(router_type, nodes, contacts, distances, distances_per_time, min_distances, Some(spsn_opts.clone()))
                    },
                    |mut router| {
                        black_box(router.route(
                            black_box(source),
                            black_box(&bundle),
                            black_box(curr_time),
                            black_box(&excluded_nodes),
                            Some(format!("benches/results_weak_h/{}/{}.txt", &modifier.to_string(), router_name))
                        ));
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
}

criterion_group! {
    name=benches;
    config=Criterion::default().sample_size(10);
    targets=benchmark
}
criterion_main!(benches);