use std::*;
use std::fs::File;
use std::io::prelude::*;
extern crate rand;
use self::rand::Rng;

pub trait RandomGenerator {
    fn next_random(&mut self) -> f64;
}

pub struct RustRand {
    generator: rand::ThreadRng
}

impl RustRand {
    pub fn new() -> RustRand{
        RustRand {
            generator: rand::thread_rng()
        }
    }
}

impl RandomGenerator for RustRand {
    fn next_random(&mut self) -> f64 {
        self.generator.next_f64()
    }
}


pub struct RandReader {
    rand_list: Vec<f64>,
    index: usize
}

impl RandReader {
    pub fn new(file_name: &str) -> RandReader {
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