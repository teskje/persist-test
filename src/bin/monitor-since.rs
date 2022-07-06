use std::env;
use std::time::Duration;

use mz_storage::client::sources::SourceData;
use mz_ore::metrics::MetricsRegistry;
use mz_persist_client::cache::PersistClientCache;
use mz_persist_client::PersistLocation;
use mz_repr::{Diff, Timestamp};
use tokio::time;

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let consensus_uri = args.next().expect("missing consensus URI");
    let blob_uri = args.next().expect("missing blob URI");
    let shard_id = args
        .next()
        .expect("missing shard ID")
        .parse()
        .expect("invalid shard ID");

    let location = PersistLocation {
        blob_uri,
        consensus_uri,
    };

    let mut persist_clients = PersistClientCache::new(&MetricsRegistry::new());
    let client = persist_clients
        .open(location)
        .await
        .expect("could not open persist client");

    loop {
        let probe = client
            .open_reader::<SourceData, (), Timestamp, Diff>(shard_id)
            .await
            .expect("could not open persist shard");
        let since = probe.since().elements()[0];
        println!("{since}");
        probe.expire().await;

        time::sleep(Duration::from_secs(1)).await;
    }
}
