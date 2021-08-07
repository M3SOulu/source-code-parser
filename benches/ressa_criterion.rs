use criterion::{black_box, criterion_group, criterion_main, Criterion};
use statistical::*;

mod test_constants;
use test_constants::*;

extern crate source_code_parser;
use source_code_parser::{
    msd::{run_msd_parse, NodePattern},
    *,
};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn laast_benchmark(c: &mut Criterion) {
    let epoch = jemalloc_ctl::epoch::mib().unwrap();
    let allocated = jemalloc_ctl::stats::allocated::mib().unwrap();

    let dir = serde_json::from_str::<Directory>(directory_json_dsb).unwrap();
    let mut mem = vec![];
    c.bench_function("LAAST", |b| {
        b.iter(|| {
            epoch.advance().unwrap();
            let before = allocated.read().unwrap();
            let _ctx = black_box(parse_project_context(&dir)).unwrap();
            epoch.advance().unwrap();
            mem.push((allocated.read().unwrap() - before) as f64);
        })
    });
    let mean = mean(&mem);
    println!(
        "{} +/- {} ({})",
        mean,
        standard_deviation(&mem, Some(mean)),
        median(&mem)
    );
}

fn ressa_benchmark(c: &mut Criterion, name: &str, msds_json: &str) {
    let dir = serde_json::from_str::<Directory>(directory_json_dsb).unwrap();
    let ctx = parse_project_context(&dir).unwrap();
    let msds = serde_json::from_str::<Vec<NodePattern>>(msds_json).unwrap();
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ctx = black_box(run_msd_parse(&mut ctx.modules.clone(), msds.clone()));
        })
    });
}

fn ressa_benchmark_endpoint_simple(c: &mut Criterion) {
    ressa_benchmark(c, "RESSA Endpoint Simple", msds_json_endpoint_simple_dsb)
}

fn ressa_benchmark_endpoint(c: &mut Criterion) {
    ressa_benchmark(c, "RESSA Endpint (Call Graph)", msds_json_endpoint_dsb)
}

fn ressa_benchmark_entity(c: &mut Criterion) {
    ressa_benchmark(c, "RESSA Entity", msds_json_entity_dsb)
}

fn ressa_benchmark_endpoint_tt(c: &mut Criterion) {
    ressa_benchmark(c, "RESSA Endpoint (TrainTicket)", msds_json_endpoint_tt)
}

fn ressa_benchmark_entity_tt(c: &mut Criterion) {
    ressa_benchmark(c, "RESSA Entity (TrainTicket)", msds_json_entity_tt)
}

criterion_group!(
    benches,
    ressa_benchmark_endpoint_simple_dsb,
    ressa_benchmark_endpoint_dsb,
    ressa_benchmark_entity_dsb,
    ressa_benchmark_endpoint_tt,
    ressa_benchmark_entity_tt
);
criterion_main!(benches);
