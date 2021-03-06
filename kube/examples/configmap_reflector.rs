#[macro_use] extern crate log;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{Api, ListParams, Meta},
    runtime::Reflector,
    Client,
};

fn spawn_periodic_reader(rf: Reflector<ConfigMap>) {
    tokio::spawn(async move {
        loop {
            // Periodically read our state
            tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
            let cms: Vec<_> = rf.state().await.unwrap().iter().map(Meta::name).collect();
            info!("Current configmaps: {:?}", cms);
        }
    });
}

/// Example way to read secrets
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let client = Client::try_default().await?;
    let namespace = std::env::var("NAMESPACE").unwrap_or("default".into());

    let cms: Api<ConfigMap> = Api::namespaced(client, &namespace);
    let lp = ListParams::default().timeout(10); // short watch timeout in this example
    let rf = Reflector::new(cms).params(lp);

    spawn_periodic_reader(rf.clone()); // read from a clone in a task

    rf.run().await?; // run reflector and listen for signals
    Ok(())
}
