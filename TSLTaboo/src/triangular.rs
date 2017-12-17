use std::*;
use std::fs::File;
use std::io::prelude::*;

pub struct TriangularMatrix<T> {
    data: Vec<T>,
    pub number_of_lines: usize
}

#[allow(dead_code)]
impl<T> TriangularMatrix<T> {
    pub fn from_file(file_name: &str) -> TriangularMatrix<usize> {
        let mut file = File::open(file_name).expect("Imposible Abrir el fichero de distancias");
        let file_content = &mut String::new();
        file.read_to_string(file_content).expect("Formato del fichero de distancias incorrecto");

        return TriangularMatrix {
            data: file_content
                .trim()
                .split_whitespace()
                .map(|e| e.parse::<usize>().expect("Elmento del fichero de distancias no es entero"))
                .collect(),
            number_of_lines: file_content.trim().lines().count() + 1
        };
    }
    pub fn filled_false(number_of_lines: usize) -> TriangularMatrix<bool> {
        let capacity = (number_of_lines * number_of_lines - number_of_lines) / 2;
        let mut data = Vec::with_capacity(capacity);
        for _ in 0..capacity { data.push(false); }
        TriangularMatrix { data, number_of_lines }
    }
    fn check_index(&self, line: usize, column: usize){
        if  column > line {panic!("Impossible to access {},{} element", line, column)}
        if  line > self.number_of_lines {panic!("Impossible to access {},{} element with {} columns"
                                          , line, column, self.number_of_lines)}
    }

    pub fn get(&self, line: usize, column: usize) -> &T {
        self.check_index(line, column);
        let line_jump = (line * line - line) / 2;     //N * (N -1)  / 2
        &self.data[line_jump + column]
    }

    pub fn set(&mut self, line: usize, column: usize, value: T) {
        self.check_index(line, column);
        let line_jump = (line * line - line) / 2;     //N * (N -1)  / 2
        self.data[line_jump + column] = value
    }

    pub fn enumerate_indexes(&self) -> TriangularMultiIndexEnumerate<T> {
        return TriangularMultiIndexEnumerate {
            index: (1, 0),
            number_of_lines: self.number_of_lines,
            matrix: &self.data
        };
    }

    pub fn enumerate_from(&self, column: usize, line: usize) -> TriangularMultiIndexEnumerate<T> {
        return TriangularMultiIndexEnumerate {
            index: (column, line),
            number_of_lines: self.number_of_lines,
            matrix: &self.data
        };
    }
}


pub struct TriangularMultiIndexEnumerate<'a, T: 'a> {
    index: (usize, usize),
    number_of_lines: usize,
    matrix: &'a Vec<T>,
}

impl<'a, T: 'a> Iterator for TriangularMultiIndexEnumerate<'a, T> {
    type Item = (usize, usize, &'a T);
    fn next(&mut self) -> Option<(usize, usize, &'a T)> {
        let line = self.index.0;
        let column = self.index.1;

        let next_column = self.index.1 + 1;
        self.index = (line + next_column / line, next_column % line);

        if line >= self.number_of_lines {
            return None;
        } else {
            let line_jump = (line * line - line) / 2;
            let value = &self.matrix[column + line_jump];
            return Some((line, column, value));
        }
    }
}
