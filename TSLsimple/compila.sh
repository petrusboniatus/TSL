#!/bin/bash

if [ ! -d ~/.cargo ]; then
	echo "Se va a descargar el compilador de Rust"
	echo "en tus carpetas $(tput bold)~/.cargo y ~/.rustup$(tput sgr0)"
	echo "para eliminarlo debes ejecutar el fichero $(tput bold)deleteCompiler.sh $(tput sgr0)" 
	echo "" 	
	curl https://sh.rustup.rs -sSf > scriptRust.sh
	sh scriptRust.sh -y --no-modify-path > /dev/null 
	rm scriptRust.sh
fi

source $HOME/.cargo/env
cargo build
cp target/debug/practica01 ./a.out
