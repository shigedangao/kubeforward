use clap::Parser;

mod scenario;
mod error;
mod kube;

#[derive(Debug, Parser)]
#[clap(name = "kubexec", author = "marc intha-amnouay")]
struct Args {
    #[clap(short, long, default_value = "default")]
    namespace: String,

    #[clap(short, long)]
    context: bool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let context_scenario_res = match args.context {
        false => None,
        true => {
            let config = scenario::context::trigger_scenario()
                .expect("Expect to retrieve a context");
            Some(config)
        }
    };

    let res = scenario::exec::trigger_scenario(
        context_scenario_res, 
        args.namespace
    ).await;

    if let Err(err) = res {
        println!("{err:?}");
    }
}
