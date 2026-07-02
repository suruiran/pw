#[derive(Debug, clap::Parser)]
#[command(version)]
pub(crate) struct Entry {
    pub cmd: String,

    #[arg(short = 'C', long)]
    pub config: Option<Vec<String>>,

    #[arg(short = 'U', long)]
    pub using: Option<String>,

    #[arg(long)]
    pub dryrun: Option<bool>,

    #[cfg(debug_assertions)]
    pub log: Option<String>,
}

pub(crate) struct Theme {}
