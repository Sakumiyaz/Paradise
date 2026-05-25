fn main() {
    eden_core::edenctl_cli::main_entry(std::env::args().skip(1).collect());
}
