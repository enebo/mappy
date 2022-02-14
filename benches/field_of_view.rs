use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use mappy::{calculate_field_of_view, Map, Spot};

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 80;

fn make_map(start: &(usize, usize), end: &(usize, usize)) -> Map<char, char> {
    let mut map = Map::new("map", MAP_WIDTH, MAP_HEIGHT, &|_| '.');
    let mut rng = rand::thread_rng();

    // Add random walls
    let n_walls = 200;
    for _ in 0..n_walls {
        let target = (
            rng.gen_range(0, MAP_WIDTH as usize - 1),
            rng.gen_range(0, MAP_HEIGHT as usize - 1)
        );
        if &target != start && &target != end {
            map.set(&target, Spot::new('#', None));
        }
    }

    map
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let (start, end) = ((1, MAP_HEIGHT - 1), (MAP_WIDTH - 3, MAP_HEIGHT - 1));
    let visible = |tile: &Spot<char, char>| tile.solid == '.';

    c.bench_function("field_of_view", |b| {
        b.iter(|| {
            let map = make_map(&start, &end);
            let mut light_map = map.create_overlay();

            calculate_field_of_view(&map, &start, 20, &mut light_map, &visible);
            black_box(light_map);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);