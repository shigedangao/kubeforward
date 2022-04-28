use clap::Parser;
use simple_logger;

mod scenario;
mod error;
mod kube;
mod utils;

#[derive(Debug, Parser)]
#[clap(name = "kubeforward", author = "marc intha-amnouay")]
struct Args {
    #[clap(short, long)]
    namespace: Option<String>,

    #[clap(short, long)]
    context: bool
}

#[tokio::main]
async fn main() {
    // init the logger
    simple_logger::init_with_level(log::Level::Info)
        .expect("Expect to initialize the logger");

    let args = Args::parse();
    let context_scenario = match args.context {
        false => None,
        true => {
            let config = scenario::context::trigger_scenario()
                .expect("Expect to retrieve a context");
            Some(config)
        }
    };

    let ns = match args.namespace {
        Some(ns) => ns,
        None => scenario::namespace::trigger_scenario(&context_scenario).await
            .expect("Expect to retrieve namespace from the list of namespace")
    };

    let res = scenario::forward::trigger_scenario(
        context_scenario,
        ns
    ).await;

    if let Err(err) = res {
        log::error!("{}", err.to_string());
    }
}
