#![deny(clippy::pedantic)]

mod cli;
mod xxtea;

mod crypt;
mod haxe;
mod savetool;

fn main() {
    cli::run();
}
