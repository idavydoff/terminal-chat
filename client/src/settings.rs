use clap::Parser;

#[derive(Parser)]
pub struct Args {
  #[arg(short, long, help = "Server address")]
  pub address: String,

  #[arg(short, long, help = "Server secret key")]
  pub key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
  pub server_address: String,
  pub server_key: Option<String>,
}

impl Settings {
  pub fn new() -> Settings {
    let args = Args::parse();
    
    Settings { 
      server_address: args.address,
      server_key: args.key
    }
  }
}
