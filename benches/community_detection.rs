//! PR #9 performance gate — community detection must stay sub-second on
//! realistic user-scale ontology graphs.
//!
//! Roadmap acceptance (docs/ARCHITECTURE.md §6E-7): 100 objects + 200
//! links must run in < 1s. Label propagation is O((V+E)·iterations) so
//! we expect sub-millisecond on this size; the bench exists as a
//! regression fence against future algorithm swaps (e.g. a Leiden
//! rewrite) that could silently blow past the budget.
//!
//! Run: `cargo bench --bench community_detection`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use zeroclaw::ontology::community::{detect_communities, GraphEdge, GraphNode, GraphView};

/// Deterministic pseudo-random graph generator. We avoid pulling in
/// `rand` as a dev-dep just for this — a splitmix64 step is enough to
/// produce a non-trivial-but-reproducible edge distribution.
fn make_graph(node_count: u32, edge_count: u32, seed: u64) -> GraphView {
    let mut state = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut next = || {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };

    let nodes: Vec<GraphNode> = (1..=i64::from(node_count))
        .map(|id| GraphNode {
            object_id: id,
            title: None,
        })
        .collect();

    let mut edges = Vec::with_capacity(edge_count as usize);
    let mut produced = 0u32;
    // Cap attempts to keep termination obvious even if the requested
    // edge count would demand too many unique pairs.
    let max_attempts = edge_count.saturating_mul(8).max(edge_count + 16);
    let mut attempts = 0u32;
    while produced < edge_count && attempts < max_attempts {
        attempts += 1;
        let a = (next() % u64::from(node_count)) as i64 + 1;
        let b = (next() % u64::from(node_count)) as i64 + 1;
        if a == b {
            continue;
        }
        edges.push(GraphEdge {
            from_object_id: a,
            to_object_id: b,
            weight: 1,
        });
        produced += 1;
    }

    GraphView { nodes, edges }
}

fn bench_detect_100_nodes_200_edges(c: &mut Criterion) {
    let graph = make_graph(100, 200, 0xA11CE);
    let mut group = c.benchmark_group("community_detection");
    group.throughput(Throughput::Elements(u64::from(100u32)));
    group.bench_function("lpa_100n_200e", |b| {
        b.iter(|| {
            let assignment = detect_communities(black_box(&graph));
            black_box(assignment);
        });
    });
    group.finish();
}

fn bench_detect_1000_nodes_3000_edges(c: &mut Criterion) {
    let graph = make_graph(1000, 3000, 0xBEEF);
    let mut group = c.benchmark_group("community_detection");
    group.throughput(Throughput::Elements(u64::from(1000u32)));
    group.bench_function("lpa_1000n_3000e", |b| {
        b.iter(|| {
            let assignment = detect_communities(black_box(&graph));
            black_box(assignment);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_detect_100_nodes_200_edges,
    bench_detect_1000_nodes_3000_edges,
);
criterion_main!(benches);
