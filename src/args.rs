#[derive(clap::Parser, Debug, Clone)]
pub enum Mode {
    Build,
    #[cfg(feature = "watch")]
    Watch,
}
