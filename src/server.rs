use clap::Parser;
use std::net::{IpAddr, /* Ipv4Addr, */ SocketAddr};

// Import the server logic from our library crate
use flight_sim::run_server;

/// Flight Simulator Server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// IP address to bind to
    #[clap(short, long, value_parser, default_value = "0.0.0.0")]
    ip: IpAddr,

    /// Port to bind to
    #[clap(short, long, value_parser, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Construct the socket address
    let addr = SocketAddr::new(args.ip, args.port);

    // Run the server using the function from the library
    run_server(addr).await;
}
