use clap::Parser;

fn main() {
    jet_bonds_cli::run(jet_bonds_cli::Opts::parse()).unwrap()
}
