extern crate criterion;
use criterion::{criterion_group, Criterion};
use maze::Definition;
use maze::Maze;
use maze::MazeError;
use maze::Solution;

fn solve() -> Result<Solution, MazeError> {
    #[rustfmt::skip]
    let grid: Vec<Vec<char>> = vec![
        vec![' ', 'S', 'W', ' ', 'W', 'W', 'W'],
        vec![' ', 'W', ' ', 'W', 'F', ' ', 'W'],
        vec![' ', ' ', ' ', ' ', 'W', ' ', ' '],
        vec!['W', 'W', ' ', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', 'W', ' ', 'W', ' '],
        vec![' ', 'W', 'W', ' ', 'W', 'W', ' '],
        vec![' ', ' ', 'W', ' ', ' ', ' ', ' '],
        vec![' ', 'W', 'W', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', ' ', 'W', ' ', ' '],
        vec![' ', 'W', 'W', 'W', ' ', 'W', ' '],
        vec![' ', ' ', ' ', 'W', ' ', 'W', ' '],
        vec![' ', 'W', 'W', ' ', ' ', ' ', ' '],
        vec![' ', ' ', ' ', 'W', ' ', 'W', 'W'],
        vec![' ', 'W', 'W', 'W', ' ', 'W', 'W'],
        vec![' ', ' ', ' ', 'W', ' ', 'W', 'W'],
        vec!['W', 'W', ' ', 'W', ' ', 'W', 'W'],
        vec![' ', ' ', ' ', ' ', ' ', ' ', 'W'],
    ];
    let m = Maze::new(Definition::from_vec(grid));
    m.solve()
}

fn maze(c: &mut Criterion) {
    let mut group = c.benchmark_group("maze_group");
    group.sample_size(200);
    group.bench_function("maze.solve()", |b| {
        b.iter(|| {
            if let Err(e) = solve() {
                panic!("{}", e);
            }
        })
    });
}

criterion_group!(benches, maze);
