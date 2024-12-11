use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, default_value = "config/config.yml")]
    pub config_loc: String,
    #[arg(short, long, default_value = "INFO")]
    pub log_level: String,
    #[arg(short, long)]
    pub out_dir: Option<String>,
}
