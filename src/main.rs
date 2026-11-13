fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(floorplan::adapters::cli::run(&args));
}
