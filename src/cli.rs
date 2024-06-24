use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
	pub file: PathBuf,
}
