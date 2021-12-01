use mappy::{Map, Tile};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 80;

fn make_map(start: &(usize, usize), end: &(usize, usize)) -> Map {
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT, '.', 1);
    let mut rng = rand::thread_rng();

    // Add random walls
    let n_walls = 200;
    for _ in 0..n_walls {
        let target = (
            rng.gen_range(0, MAP_WIDTH as usize - 1),
            rng.gen_range(0, MAP_HEIGHT as usize - 1)
        );
        if &target != start && &target != end {
            map.set(&target, Tile::new('#', 1));
        }
    }

    map
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let (start, end) = ((1, MAP_HEIGHT - 1), (MAP_WIDTH - 3, MAP_HEIGHT - 1));
    let available = |tile: &Tile| tile.id == '.';

    c.bench_function("a_star_test_map", |b| {
        b.iter(|| {
            let map = make_map(&start, &end);
            if let Some(path) = map.shortest_path(&start, &end, &available) {
                black_box(path);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);