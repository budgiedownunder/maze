use crate::Definition;
use crate::solution::Solution;
use crate::Solver;
use crate::solver::SolveError;

#[allow(dead_code)]
pub struct Maze {
    pub definition: Definition,
}

impl Maze {
    pub fn new(definition: Definition) -> Maze {
        Maze {
            definition
        }
    }
    pub fn solve(&self) -> Result<Solution, SolveError> {
        let s = Solver{ maze: &self };
        s.solve()
    }
}
