use gumdrop::Options;

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "Launch specific installation by its ID", meta = "ID")]
    pub install: Option<String>,
}

impl Args {
    pub fn parse() -> Self {
        Args::parse_args_default_or_exit()
    }
}

pub fn get_launch_install() -> Option<String> {
    Args::parse().install
}