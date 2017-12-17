use std::*;

extern crate ordered_float;
extern crate linked_hash_set;
extern crate rayon;

use linked_hash_set::LinkedHashSet;
pub use ordered_float::*;
use rayon::prelude::*;
mod triangular;

use triangular::TriangularMatrix;
use triangular::FreqMatrix;

mod random_generator;

use random_generator::RandomGenerator;
use random_generator::RustRand;

const NUMBER_OF_ITERATIONS: usize = 10_000;
const TABOO_LIST_MAX_ELEMENTS: usize = 30;
const REBOOT_ON_IT: usize = 99;
const DIVISOR_CHANGE_ON_REBOOT: usize = 4;
const TRIES_ON_REBOOT: usize = 1000;
const REPETITION_CONST: f64 = 1.0;
const INTENSIFICATION_MOD: usize = 10;

struct PathFinder {
    cost_map: TriangularMatrix<usize>,
    current_solution: Vec<usize>,
    solution_size: usize,
    best_cost: f64,
    best_solution: Vec<usize>,
    best_solution_iteration: usize,
    taboo_list: LinkedHashSet<(usize, usize)>,
    non_improvement_iterations: usize,
    total_iterations: usize,
    number_of_reboots: usize,
    freq_mat: FreqMatrix,
}


impl string::ToString for PathFinder {
    fn to_string(&self) -> String {
        let mut result: String;
        if self.total_iterations == 0 {
            result = format!("\
            RECORRIDO INICIAL\n\
            \tRECORRIDO: {}\n\
            \tCOSTE (km): {}\n\
            ",
                             self.current_solution.iter()
                                 .fold(String::new(), |acc, e| {
                                     acc + &e.to_string() + " "
                                 }),
                             self.calculate_cost(&self.current_solution)
            );
        } else {
            result = format!("\
            ITERACION: {}\n\
            \tINTERCAMBIO: {:?}\n\
            \tRECORRIDO: {}\n\
            \tCOSTE (km): {}\n\
            \tITERACIONES SIN MEJORA: {}\n\
            \tLISTA TABU:\n{}\n\
            ",
             self.total_iterations,
             self.taboo_list.iter().last().unwrap(),
             self.current_solution
                 .iter()
                 .fold(String::new(), |acc, e| {
                     acc + &e.to_string() + " "
                 }),
             self.calculate_cost(&self.current_solution),
             self.non_improvement_iterations,
             self.taboo_list.iter()
                 .fold(String::new(), |acc, &(i, j)| {
                     acc + "\t" + &i.to_string() + " " + &j.to_string() + "\n"
                 })
            );
        }

        if self.non_improvement_iterations > REBOOT_ON_IT {
            result = result + &format!("\
                ***************\n\
                REINICIO: {}\n\
                ***************\n\n",
                self.number_of_reboots
            );
        }

        result
    }
}

impl PathFinder {
    fn generate_greedy_solution(&mut self) -> Vec<usize> {
        let mut first_solution: Vec<usize> = Vec::with_capacity(self.solution_size);
        let mut node_from: usize = 0;
        for _ in 0..self.solution_size {
            let (i, j, _) = self.cost_map.enumerate_indexes()
                .filter(|&(i, j, _)| i == node_from || j == node_from)
                .filter(|&(i, j, _)| !first_solution.contains(&i) || !first_solution.contains(&j))
                .filter(|&(i, j, _)| (i != 0 && j != 0) || first_solution.len() == 0
                    || first_solution.len() == (self.solution_size - 2))
                .min_by_key(|&(_, _, cost)| cost)
                .expect("Fail on greedy solution");

            let node_to;
            if i == node_from { node_to = j; } else { node_to = i; }

            first_solution.push(node_to);
            node_from = node_to;
        }
        first_solution
    }

    fn new(cost_map: &str) -> PathFinder {
        let cost_map = TriangularMatrix::<usize>::from_file(cost_map);
        let solution_size = cost_map.number_of_lines - 1;
        let taboo_list = LinkedHashSet::new();

        let mut next_path_finder = PathFinder {
            cost_map: cost_map,
            current_solution: Vec::new(),
            best_solution: Vec::new(),
            best_solution_iteration: 0,
            solution_size: solution_size,
            best_cost: 0.0,
            taboo_list: taboo_list,
            non_improvement_iterations: 0,
            total_iterations: 0,
            number_of_reboots: 1,
            freq_mat: FreqMatrix::new(solution_size + 1)
        };
        next_path_finder.current_solution = next_path_finder.generate_greedy_solution();
        next_path_finder.best_cost = next_path_finder.calculate_cost(&next_path_finder.current_solution);
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
        for swap in j..(i + 1) {
            solution[swap] = self.current_solution[i - swap + j];
        }
        return solution;
    }

    fn generate_neighbours(&self) -> Vec<(usize, usize)> {
        let mut neighbours = Vec::new();
        for i in 0..self.solution_size {
            for j in 0..i {
                neighbours.push((i, j));
            }
        }
        neighbours
    }

    fn reboot_diversification(&mut self){
        let mut g = RustRand::new();
        let mut best_vec = Vec::new();
        let mut best_cost = f64::MAX;

        let delta_cost = (*self.cost_map.get_max() - *self.cost_map.get_min()) as f64;

        for _ in 0..TRIES_ON_REBOOT {
            let mut new_vec = self.current_solution.clone();
            for _ in 0..(self.current_solution.len() / DIVISOR_CHANGE_ON_REBOOT) {
                let rand_num1 = g.next_random();
                let rand_num2 = g.next_random();
                let rand_multiplier = (self.solution_size / 4) as f64;
                let rand_position1 = (rand_num1 * rand_multiplier).floor() as usize + 1;
                let rand_position2 = (rand_num2 * rand_multiplier).floor() as usize + 1;
                new_vec.swap(rand_position1, rand_position2);
            }

            let freq_cost = self.freq_mat.get_solution_freq_cost(&new_vec);
            let new_cost = self.calculate_cost(&new_vec) + freq_cost * delta_cost
                * REPETITION_CONST;

            if new_cost < best_cost {
                best_cost = new_cost;
                best_vec = new_vec;
            }
        }
        self.current_solution = best_vec;
        self.non_improvement_iterations = 0;
        self.taboo_list.clear();
        self.number_of_reboots += 1;
    }

    fn reboot_intensification(&mut self){
        self.current_solution = self.best_solution.clone();
        self.non_improvement_iterations = 0;
        self.number_of_reboots += 1;
    }

    fn reboot_if_necessary(&mut self) {
        if self.non_improvement_iterations <= REBOOT_ON_IT {  return; }

        if self.number_of_reboots % INTENSIFICATION_MOD == 0 {
            self.reboot_intensification();
        } else {
            self.reboot_diversification();
        }

    }

    fn save_current_if_it_is_the_best(&mut self, current_solution_cost: f64) {
        if current_solution_cost < self.best_cost {
            self.best_cost = current_solution_cost;
            self.best_solution = self.current_solution.clone();
            self.non_improvement_iterations = 0;
            self.best_solution_iteration = self.total_iterations;
        } else {
            self.non_improvement_iterations += 1;
        }
    }

    fn update_taboo_list(&mut self, swap: (usize, usize)) {
        if self.taboo_list.len() == TABOO_LIST_MAX_ELEMENTS {
            self.taboo_list.pop_front();
        }
        self.taboo_list.insert(swap);
    }

    fn next_solution(&mut self) {
        self.reboot_if_necessary();
        let best_neighbour = self.generate_neighbours().par_iter()
            .filter(|e| !self.taboo_list.contains(*e))
            .map(|&(i, j)| (i, j, self.calculate_cost(&self.swap_solution(i, j))))
            .min_by_key(|&(_, _, cost)| OrderedFloat(cost))
            .unwrap();

        let best_neighbour_cost = best_neighbour.2;
        let best_swap = (best_neighbour.0, best_neighbour.1);
        self.current_solution = self.swap_solution(best_neighbour.0, best_neighbour.1);

        self.freq_mat.insert_solution(&self.current_solution);

        self.total_iterations += 1;
        self.update_taboo_list(best_swap);
        self.save_current_if_it_is_the_best(best_neighbour_cost);
    }
}


fn main() {

    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 2 {
        eprintln!("UTILIZA ./a.out <distancias.txt>");
        std::process::exit(0);
    }

    let mut solver = PathFinder::new(&arguments[1]);

    println!("{}", solver.to_string());
    for _ in 0..NUMBER_OF_ITERATIONS {
        solver.next_solution();
        print!("{}", solver.to_string());
    }

    let result: String = format!("\
            \nMEJOR SOLUCION: \n\
            \tRECORRIDO: {}\n\
            \tCOSTE (km): {}\n\
            \tITERACION: {}\n\
            ",
                                 solver.best_solution.iter()
                                     .fold(String::new(), |acc, e| {
                                         acc + &e.to_string() + " "
                                     }),
                                 solver.calculate_cost(&solver.best_solution),
                                 solver.best_solution_iteration

    );

    print!("{}", result);

}
