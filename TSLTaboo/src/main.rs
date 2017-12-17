use std::*;

extern crate ordered_float;
extern crate linked_hash_set;

use linked_hash_set::LinkedHashSet;
pub use ordered_float::*;

mod triangular;

use triangular::TriangularMatrix;

mod random_generator;

use random_generator::RandomGenerator;
use random_generator::RandReader;
use random_generator::RustRand;


struct PathFinder<'a> {
    rand_gen: &'a mut RandomGenerator,
    cost_map: TriangularMatrix<usize>,
    current_solution: Vec<usize>,
    solution_size: usize,
    best_cost: f64,
    best_solution: Vec<usize>,
    best_solution_iteration: usize,
    taboo_list: LinkedHashSet<(usize, usize)>,
    reboot_parameter: usize,
    non_improvement_iterations: usize,
    total_iterations: usize,
    number_of_reboots: usize,
    taboo_list_size: usize
}


impl<'a> string::ToString for PathFinder<'a> {
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

        if self.non_improvement_iterations > self.reboot_parameter {
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

impl<'a> PathFinder<'a> {
    fn generate_rand_solution(&mut self) -> Vec<usize> {
        let mut rand_solution: Vec<usize> = Vec::with_capacity(self.solution_size);
        for _ in 0..self.solution_size {
            let rand_num = self.rand_gen.next_random();
            let rand_multiplier = self.solution_size as f64;
            let mut rand_position = (rand_num * rand_multiplier).floor() as usize + 1;
            while rand_solution.contains(&rand_position) {
                rand_position = cmp::max((rand_position + 1) % (self.solution_size + 1), 1);
            }
            rand_solution.push(rand_position);
        }
        rand_solution
    }

    fn new(cost_map: &str, rand_gen: &'a mut RandomGenerator,
           taboo_list_max_size: usize, reboot_parameter: usize)
           -> PathFinder<'a> {
        let cost_map = TriangularMatrix::<usize>::from_file(cost_map);
        let solution_size = cost_map.number_of_lines - 1;
        let taboo_list = LinkedHashSet::new();

        let mut next_path_finder = PathFinder {
            rand_gen: rand_gen,
            cost_map: cost_map,
            current_solution: Vec::new(),
            best_solution: Vec::new(),
            best_solution_iteration: 0,
            solution_size: solution_size,
            best_cost: 0.0,
            taboo_list: taboo_list,
            reboot_parameter: reboot_parameter,
            non_improvement_iterations: 0,
            total_iterations: 0,
            number_of_reboots: 1,
            taboo_list_size: taboo_list_max_size,
        };
        next_path_finder.current_solution = next_path_finder.generate_rand_solution();
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
        solution[i] = self.current_solution[j];
        solution[j] = self.current_solution[i];
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

    fn reboot_if_necessary(&mut self) {
        if self.non_improvement_iterations > self.reboot_parameter {
            self.current_solution = self.best_solution.clone();
            self.non_improvement_iterations = 0;
            self.taboo_list.clear();
            self.number_of_reboots += 1;
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

    fn update_taboo_list(&mut self, swap: (usize, usize)){
        if self.taboo_list.len() == self.taboo_list_size {
            self.taboo_list.pop_front();
        }
        self.taboo_list.insert(swap);
    }

    fn next_solution(&mut self) {
        self.reboot_if_necessary();
        let best_neighbour = self.generate_neighbours().iter()
            .filter(|e| !self.taboo_list.contains(*e))
            .map(|&(i, j)| (i, j, self.calculate_cost(&self.swap_solution(i, j))))
            .min_by_key(|&(_, _, cost)| OrderedFloat(cost))
            .unwrap();

        let best_neighbour_cost = best_neighbour.2;
        let best_swap = (best_neighbour.0, best_neighbour.1);
        self.current_solution = self.swap_solution(best_neighbour.0, best_neighbour.1);

        self.total_iterations += 1;
        self.update_taboo_list(best_swap);
        self.save_current_if_it_is_the_best(best_neighbour_cost);
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let number_of_iterations: usize = 10000;
    let taboo_list_max_elemnts: usize = 100;
    let reboot_on_it: usize = 99;

    let mut random_gen: Box<RandomGenerator> = match arguments.len() {
        2 => Box::new(RustRand::new()),
        3 => Box::new(RandReader::new(&arguments[2])),
        _ => {
            eprintln!("UTILIZA ./a.out <distancias.txt> [aleatorios.txt]");
            std::process::exit(0);
        }
    };

    let mut solver = PathFinder::new(&arguments[1], &mut *random_gen,
                                     taboo_list_max_elemnts, reboot_on_it);


    println!("{}", solver.to_string());
    for _ in 0..number_of_iterations {
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
