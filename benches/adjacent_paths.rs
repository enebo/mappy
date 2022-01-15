use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mappy::Map;
use mappy::map::generate_ascii_map;

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 80;

pub fn criterion_benchmark(c: &mut Criterion) {
    let map_string = "##############\n\
                          #..#......#..#\n\
                          #..###.#.....#\n\
                          #..###..#.#..#\n\
                          #..######.#..#\n\
                          #............#\n\
                          ##############";
    let map = generate_ascii_map("map", map_string).unwrap();

    c.bench_function("adjacent_paths", |b| {
        b.iter(|| {
            let value = map.adjacent_paths(&(0, 0), &|c| *c == '#', false);
            black_box(value);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);