extern crate criterion;
use criterion::{criterion_group, Criterion};
use data_model::{Maze, MazeDefinition};
use maze::{Error as MazeError, MazeSolver, MazeSolution};

fn solve() -> Result<MazeSolution, MazeError> {
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
    let maze = Maze::new(MazeDefinition::from_vec(grid));
    maze.solve()
}

fn maze(c: &mut Criterion) {
    let mut group = c.benchmark_group("maze_group");
    group.sample_size(100);
    group.bench_function("maze.solve()", |b| {
        b.iter(|| {
            if let Err(error) = solve() {
                panic!("{}", error);
            }
        })
    });
}

criterion_group!(benches, maze);
