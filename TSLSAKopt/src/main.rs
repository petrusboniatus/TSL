use std::*;

extern crate ordered_float;

pub use ordered_float::*;

mod triangular;

use triangular::TriangularMatrix;

mod random_generator;

use random_generator::RandomGenerator;
use random_generator::RandReader;
use random_generator::RustRand;


struct PathFinder<'a> {
    mu: f64,
    phi: f64,
    solution_size: usize,
    rand_gen: &'a mut RandomGenerator,
    cost_map: TriangularMatrix<usize>,

    current_solution: Vec<usize>,
    current_solution_cost: f64,

    tested_solution: Vec<usize>,
    tested_solution_insertion: (usize, usize),
    tested_solution_cost: f64,

    best_cost: f64,
    best_solution: Vec<usize>,
    best_solution_iteration: usize,
    last_was_accepted: bool,

    total_iterations: usize,

    initial_temperature: f64,
    current_temperature: f64,
    accepted_candidates: usize,
    tested_candidates: usize,
    delta: f64,
    cooldowns_counter: usize,
}


impl<'a> string::ToString for PathFinder<'a> {
    fn to_string(&self) -> String {
        let mut acepted_message = "";
        if self.last_was_accepted {
            acepted_message = "\tSOLUCION CANDIDATA ACEPTADA\n";
        }

        let mut result = String::new();

        if self.cooldowns_counter > 0 && self.tested_candidates == 1 {
            result = format!("\
            ============================\n\
            ENFRIAMIENTO: {:.6}\n\
            ============================\n\
            TEMPERATURA: {:.6}\n\n",
                             self.cooldowns_counter,
                             self.current_temperature
            );
        }

        if self.total_iterations == 0 {
            result += &format!("\
            SOLUCION INICIAL:\n\
            \tRECORRIDO: {}\n\
            \tFUNCION OBJETIVO (km): {}\n\
            \tTEMPERATURA INICIAL: {:.6}\n",
                               self.current_solution.iter()
                                   .fold(String::new(), |acc, e| {
                                       acc + &e.to_string() + " "
                                   }),
                               self.calculate_cost(&self.current_solution),
                               self.current_temperature
            );
        } else {
            result += &format!("\
            ITERACION: {}\n\
            \tINDICE CIUDAD: {}\n\
            \tCIUDAD: {}\n\
            \tINDICE INSERCION: {}\n\
            \tRECORRIDO: {}\n\
            \tFUNCION OBJETIVO (km): {}\n\
            \tDELTA: {:.0}\n\
            \tTEMPERATURA: {:.6}\n\
            \tVALOR DE LA EXPONENCIAL: {:.6}\n\
            {}\
            \tCANDIDATAS PROBADAS: {}, ACEPTADAS: {}\n\n\
            ",
                               self.total_iterations,
                               self.tested_solution_insertion.0,
                               self.tested_solution[self.tested_solution_insertion.1],
                               self.tested_solution_insertion.1,
                               self.tested_solution.iter()
                                   .fold(String::new(), |acc, e| {
                                       acc + &e.to_string() + " "
                                   }),
                               self.tested_solution_cost,
                               self.delta,
                               self.current_temperature,
                               f64::exp(-self.delta / self.current_temperature),
                               acepted_message,
                               self.tested_candidates, self.accepted_candidates
            );
        }


        result
    }
}

impl<'a> PathFinder<'a> {
    fn new(cost_map: &str, rand_gen: &'a mut RandomGenerator, phi: f64, mu: f64) -> PathFinder<'a> {
        let cost_map = TriangularMatrix::<usize>::from_file(cost_map);
        let solution_size = cost_map.number_of_lines - 1;

        let mut next_pf = PathFinder {
            phi: phi,
            mu: mu,
            rand_gen: rand_gen,
            cost_map: cost_map,
            current_solution: Vec::new(),
            current_solution_cost: 0.0,
            best_solution: Vec::new(),
            best_solution_iteration: 0,
            solution_size: solution_size,
            last_was_accepted: true,
            tested_solution_insertion: (0, 0),
            tested_solution: Vec::new(),
            tested_solution_cost: 0.0,
            best_cost: 0.0,
            total_iterations: 0,
            accepted_candidates: 0,
            tested_candidates: 0,
            current_temperature: 0.0,
            cooldowns_counter: 0,
            delta: 0.0,
            initial_temperature: 0.0,
        };
        next_pf.current_solution = next_pf.generate_rand_solution();
        next_pf.best_cost = next_pf.calculate_cost(&next_pf.current_solution);
        next_pf.current_solution_cost = next_pf.best_cost;
        next_pf.current_temperature = (-next_pf.mu / f64::ln(next_pf.phi)) * next_pf.best_cost;
        next_pf.initial_temperature = next_pf.current_temperature;
        next_pf
    }


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

    fn generate_neighbours(&mut self) -> Vec<(usize, usize)> {
        let mut neighbours = Vec::new();
        let rand_float = self.rand_gen.next_random();
        let node_from: usize = (rand_float * (self.solution_size as f64)).floor() as usize;

        for i in 0..self.solution_size {
            if i != node_from {
                neighbours.push((node_from, i));
            }
        }
        neighbours
    }

    fn insert_on_current(&self, insertion: (usize, usize)) -> Vec<usize> {
        let i = cmp::max(insertion.0, insertion.1);
        let j = cmp::min(insertion.0, insertion.1);
        let mut solution = self.current_solution.clone();
        for swap in j..(i + 1) {
            solution[swap] = self.current_solution[i - swap + j];
        }

        solution
    }

    fn save_tested_if_proceed(&mut self) {
        let probability_of_acceptation;
        self.delta = self.tested_solution_cost - self.current_solution_cost;

        if self.tested_solution_cost < self.current_solution_cost {
            probability_of_acceptation = 1.0
        } else {
            probability_of_acceptation = f64::exp(-self.delta / self.current_temperature);
        }

        self.last_was_accepted = self.rand_gen.next_random() < probability_of_acceptation;

        if self.last_was_accepted {
            self.current_solution = self.tested_solution.clone();
            self.current_solution_cost = self.tested_solution_cost;
            self.accepted_candidates += 1;
        }
        self.tested_candidates += 1;

        if self.current_solution_cost < self.best_cost {
            self.best_cost = self.current_solution_cost;
            self.best_solution = self.current_solution.clone();
            self.best_solution_iteration = self.total_iterations;
        }
    }

    fn cooldown_if_proceed(&mut self) {
        if self.tested_candidates >= 120 || self.accepted_candidates >= 40 {
            self.accepted_candidates = 0;
            self.tested_candidates = 0;
            self.cooldowns_counter += 1;
            self.current_temperature = self.initial_temperature / (1.0 + self.cooldowns_counter as f64);
        }
    }

    fn next_solution(&mut self) {
        self.cooldown_if_proceed();
        let best_neighbour = self.generate_neighbours().iter()
            .map(|&(i, j)|
                (i, j, self.calculate_cost(&self.insert_on_current((i, j))))
            )
            .min_by_key(|&(_, _, cost)| OrderedFloat(cost))
            .unwrap();

        self.tested_solution_cost = best_neighbour.2;
        self.tested_solution_insertion = (best_neighbour.0, best_neighbour.1);
        self.tested_solution = self.insert_on_current(self.tested_solution_insertion);

        self.total_iterations += 1;
        self.save_tested_if_proceed();
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let number_of_iterations: usize = 10000;
    let phi = 0.7;
    let mu = 0.01;

    let mut random_gen: Box<RandomGenerator> = match arguments.len() {
        2 => Box::new(RustRand::new()),
        3 => Box::new(RandReader::new(&arguments[2])),
        _ => {
            eprintln!("UTILIZA ./a.out <distancias.txt> [aleatorios.txt]");
            std::process::exit(0);
        }
    };

    let mut solver = PathFinder::new(&arguments[1], &mut *random_gen,
                                     phi, mu);


    println!("{}", solver.to_string());
    for _ in 0..number_of_iterations {
        solver.next_solution();
        print!("{}", solver.to_string());
    }

    let result: String = format!("\
        \nMEJOR SOLUCION: \n\
        \tRECORRIDO: {}\n\
        \tFUNCION OBJETIVO (km): {}\n\
        \tITERACION: {}\n\
        \tmu = {:#?}, phi = {:#?}\n",
                                 solver.best_solution.iter()
                                     .fold(String::new(), |acc, e| {
                                         acc + &e.to_string() + " "
                                     }),
                                 solver.best_cost,
                                 solver.best_solution_iteration,
                                 solver.mu, solver.phi
    );

    print!("{}", result);

}
