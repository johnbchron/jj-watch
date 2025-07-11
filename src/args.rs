use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
  #[arg(long, default_value_t = false)]
  pub no_snapshot: bool,
}
