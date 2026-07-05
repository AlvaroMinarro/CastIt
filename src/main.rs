mod app;
mod config;
mod domain;
mod infra;

fn main() -> Result<(), iced_layershell::Error> {
    app::run()
}
