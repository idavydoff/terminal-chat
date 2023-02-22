use clap::Parser;

#[derive(Parser)]
pub struct Args {
  #[arg(short, long, help = "Port that the server will serve")]
  pub port: u16,

  #[arg(short, long, help = "Maximum amount of chat users")]
  pub max_users: Option<u16>,

  #[arg(short, long, help = "The key that users need to know to participate the chat")]
  pub key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Settings {
  pub port: u16,
  pub max_users: u16,
  pub key: Option<String>,
}

impl Settings {
  pub fn new() -> Settings {
    let args = Args::parse();
    
    Settings { 
      port: args.port, 
      max_users: args.max_users.unwrap_or(10), 
      key: args.key
    }
  }
}
