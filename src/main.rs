fn main() {
    std::process::exit(forge::app::run(
        std::env::args_os().collect(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ));
}
