fn main() -> anyhow::Result<()> {
    good_parts::run(std::env::args_os())
}
