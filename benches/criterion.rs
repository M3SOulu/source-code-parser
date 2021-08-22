use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use statistical::*;

mod test_constants;
use test_constants::*;

extern crate source_code_parser;
use source_code_parser::{
    ressa::{run_ressa_parse, Executor, NodePattern, ParserContext},
    *,
};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn ast_benchmark<'a, T, F>(c: &mut Criterion, name: &str, dir: &'a Directory, f: F)
where
    T: 'a,
    F: Fn(&'a Directory) -> T,
{
    let epoch = jemalloc_ctl::epoch::mib().unwrap();
    let allocated = jemalloc_ctl::stats::allocated::mib().unwrap();

    let mut mem = vec![];
    c.bench_function(name, |b| {
        b.iter(|| {
            epoch.advance().unwrap();
            let before = allocated.read().unwrap();
            let _ctx = black_box(f(dir));
            epoch.advance().unwrap();
            mem.push(abs_diff(allocated.read().unwrap(), before) as f64);
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

fn ressa_benchmark(c: &mut Criterion, name: &str, ressa_json: &str, dir: &str) {
    let dir = serde_json::from_str::<Directory>(dir).unwrap();
    let ctx = parse_project_context(&dir).unwrap();
    let ressa = serde_json::from_str::<Vec<NodePattern>>(ressa_json).unwrap();
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ctx = black_box(run_ressa_parse(&mut ctx.modules.clone(), ressa.clone()));
        })
    });
}

fn ressa_benchmark_endpoint_simple(c: &mut Criterion) {
    ressa_benchmark(
        c,
        "ressa_endpoint_deathstarbench_simple",
        ressa_json_endpoint_simple_dsb,
        &*directory_json_dsb(),
    )
}

fn ressa_benchmark_endpoint(c: &mut Criterion) {
    ressa_benchmark(
        c,
        "ressa_endpoint_deathstarbench_call_graph",
        ressa_json_endpoint_dsb,
        &*directory_json_dsb(),
    )
}

fn ressa_benchmark_entity(c: &mut Criterion) {
    ressa_benchmark(
        c,
        "ressa_entity_deathstarbench",
        ressa_json_entity_dsb,
        &*directory_json_dsb(),
    )
}

fn ressa_benchmark_endpoint_tt(c: &mut Criterion) {
    ressa_benchmark(
        c,
        "ressa_endpoint_trainticket",
        ressa_json_endpoint_tt,
        &*directory_json_tt(),
    )
}

fn ressa_benchmark_entity_tt(c: &mut Criterion) {
    ressa_benchmark(
        c,
        "ressa_entity_trainticket",
        ressa_json_entity_tt,
        &*directory_json_tt(),
    )
}

fn laast_benchmark_dsb(c: &mut Criterion) {
    let dir = serde_json::from_str::<Directory>(directory_json_dsb().as_str()).unwrap();
    ast_benchmark(c, "laast_deathstarbench", &dir, parse_project_context)
}

fn laast_benchmark_tt(c: &mut Criterion) {
    let dir = serde_json::from_str::<Directory>(directory_json_tt().as_str()).unwrap();
    ast_benchmark(c, "laast_trainticket", &dir, parse_project_context)
}

fn treesitter_ast_benchmark_dsb(c: &mut Criterion) {
    let dir = serde_json::from_str::<Directory>(directory_json_dsb().as_str()).unwrap();
    ast_benchmark(c, "treesitter_ast_deathstarbench", &dir, parse_directory_ts)
}

fn treesitter_ast_benchmark_tt(c: &mut Criterion) {
    let dir = serde_json::from_str::<Directory>(directory_json_tt().as_str()).unwrap();
    ast_benchmark(c, "treesitter_ast_trainticket", &dir, parse_directory_ts)
}

criterion_group!(
    benches,
    laast_benchmark_dsb,
    treesitter_ast_benchmark_dsb,
    laast_benchmark_tt,
    treesitter_ast_benchmark_tt,
    ressa_benchmark_endpoint_simple,
    ressa_benchmark_endpoint,
    ressa_benchmark_entity,
    ressa_benchmark_endpoint_tt,
    ressa_benchmark_entity_tt
);
criterion_main!(benches);
