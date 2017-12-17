use std::*;
use std::fs::File;
use std::io::prelude::*;

extern crate rand;
use rand::Rng;
mod triangular;

use triangular::TriangularMatrix;

trait RandomGenerator {
    fn next_random(&mut self) -> f64;
}

struct RustRand {
    generator: rand::ThreadRng
}

impl RustRand {
    fn new() -> RustRand {
        RustRand{
            generator: rand::thread_rng()
        }
    }
}

impl RandomGenerator for RustRand {
    fn next_random(&mut self) -> f64 {
        self.generator.next_f64()
    }
}


struct RandReader {
    rand_list: Vec<f64>,
    index: usize
}

impl RandReader {
    fn new(file_name: &str) -> RandReader {
        let mut file = File::open(file_name).expect("Imposible Abrir el fichero de aleatorios");
        let file_content = &mut String::new();
        file.read_to_string(file_content).expect("Formato del fichero de aleatorios incorrecto");

        return RandReader {
            rand_list: file_content
                .split_whitespace()
                .map(str::parse::<f64>)
                .map(|e| e.expect("Formato de fichero de aleatorios no parseable a puntoflotante"))
                .collect(),
            index: 0
        };
    }
}

impl RandomGenerator for RandReader {
    fn next_random(&mut self) -> f64 {
        let next_float = self.rand_list[self.index];
        self.index = (self.index + 1) % self.rand_list.len();

        next_float
    }
}


struct PathFinder<'a> {
    rand_gen: &'a mut RandomGenerator,
    cost_map: TriangularMatrix<usize>,
    current_solution: Vec<usize>,
    solution_size: usize,
    current_cost: f64,
    visited_nodes: TriangularMatrix<bool>
}

impl<'a> PathFinder<'a> {
    fn new(cost_map: &str, rand_gen: &'a mut RandomGenerator) -> PathFinder<'a> {
        let cost_map = TriangularMatrix::<usize>::from_file(cost_map);
        let solution_size = cost_map.number_of_lines - 1;
        let mut first_solution: Vec<usize> = Vec::with_capacity(solution_size);
        let visited_nodes = TriangularMatrix::<bool>::filled_false(solution_size);

        for _ in 0..solution_size {
            let rand_num = rand_gen.next_random();
            let rand_multiplier = solution_size as f64;
            let mut rand_position = (rand_num * rand_multiplier).floor() as usize + 1;
            while first_solution.contains(&rand_position) {
                rand_position = cmp::max((rand_position + 1) % (solution_size + 1), 1);
            }
            first_solution.push(rand_position);
        }

        let mut next_path_finder = PathFinder {
            rand_gen: rand_gen,
            cost_map: cost_map,
            current_solution: first_solution,
            solution_size: solution_size,
            current_cost: 0.0,
            visited_nodes: visited_nodes,
        };
        next_path_finder.current_cost = next_path_finder.calculate_cost(&next_path_finder.current_solution);

        next_path_finder
    }

    fn calculate_cost(&self, solution: &Vec<usize>) -> f64 {
        let mut total_cost: usize = 0;

        total_cost += *self.cost_map.get(solution[0], 0);

        for i in 0..(self.solution_size - 1) {
            let node_to = cmp::max(solution[i], solution[i + 1]);
            let node_from = cmp::min(solution[i], solution[i + 1]);
            total_cost += *self.cost_map.get(node_to, node_from);
        }

        total_cost += *self.cost_map.get(solution[self.solution_size - 1], 0);

        total_cost as f64
    }

    fn swap_solution(&self, i: usize, j: usize) -> Vec<usize> {
        let mut solution = self.current_solution.clone();
        solution[i] = self.current_solution[j];
        solution[j] = self.current_solution[i];
        return solution;
    }


    fn next_neighbour(&mut self, number_of_neighbour: i32) -> Option<Vec<usize>> {
        let swap_i = (self.solution_size as f64 * self.rand_gen.next_random()).floor() as usize;
        let swap_j = (self.solution_size as f64 * self.rand_gen.next_random()).floor() as usize;
        let mut i = cmp::max(swap_i, swap_j);
        let mut j = cmp::min(swap_i, swap_j);

        if i == j {
            i = cmp::max((i + 1) % self.solution_size, 1);
            j = 0;
        }
        let result = self.visited_nodes
            .enumerate_from(i, j)
            .chain(self.visited_nodes.enumerate_indexes().take((i) * self.solution_size + j))
            .find(|&(_, _, visited)| *visited == false)
            .map(|(i, j, visited)| (i, j, *visited));
        match result {
            Some((i, j, _)) => {
                self.visited_nodes.set(i, j, true);
                let res = self.swap_solution(i, j);
                print!("\tVECINO V_{} -> Intercambio: ({}, {}); {:?}; ", number_of_neighbour, i, j, res);

                Some(res)
            }
            None => None
        }
    }

    fn next_solution(&mut self) -> Option<(&Vec<usize>, f64)> {
        let mut n_neighbour = 0;
        loop {
            let next_neighbour = match self.next_neighbour(n_neighbour) {
                Some(next) => next,
                None => break
            };
            n_neighbour += 1;
            let next_cost = self.calculate_cost(&next_neighbour);
            println!("{}km", next_cost);

            if next_cost < self.current_cost {
                self.current_cost = next_cost;
                self.current_solution = next_neighbour;
                self.visited_nodes = TriangularMatrix::<bool>::filled_false(self.solution_size);
                return Some((&self.current_solution, self.current_cost));
            }
        }
        return None;
    }
}


fn main() {

    let arguments: Vec<String> = env::args().collect();

    let mut random_gen: Box<RandomGenerator> = match arguments.len() {
        2 => Box::new(RustRand::new()),
        3 => Box::new(RandReader::new(&arguments[2])),
        _ => panic!("\n\n Invalid syntax: ./a.out <distancias.txt> [aleatorios.txt]\n\n")
    };

    let mut solver = PathFinder::new(&arguments[1], &mut *random_gen);

    let mut i = 0;
    println!("\nSOLUCION S_{} -> {:?}; {}km", i, solver.current_solution, solver.current_cost);
    loop {
        i += 1;
        match solver.next_solution() {
            Some((solution, cost)) => println!("\nSOLUCION S_{} -> {:?}; {}km", i, solution, cost),
            None => break
        }
    }
}


