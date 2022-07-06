use std::env;

use mz_ore::metrics::MetricsRegistry;
use mz_persist_client::cache::PersistClientCache;
use mz_persist_client::read::ListenEvent;
use mz_persist_client::PersistLocation;
use mz_repr::{Diff, Timestamp};
use mz_storage::client::sources::SourceData;

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
    let mut reader = persist_clients
        .open(location)
        .await
        .expect("could not open persist client")
        .open_reader::<SourceData, (), Timestamp, Diff>(shard_id)
        .await
        .expect("could not open persist shard");

    let as_of = reader.since();

    let mut snapshot = reader
        .snapshot(as_of.clone())
        .await
        .expect("could not serve requested as_of");

    while let Some(updates) = snapshot.next().await {
        for ((key, _), ts, diff) in updates {
            let row = key.unwrap().0.unwrap();
            println!("{row}, {ts}, {diff}");
        }
    }

    let mut listen = reader
        .listen(as_of.clone())
        .await
        .expect("cannot serve requested as_of");

    loop {
        for event in listen.next().await {
            match event {
                ListenEvent::Progress(upper) => {
                    reader.downgrade_since(upper.clone()).await;
                }
                ListenEvent::Updates(updates) => {
                    for ((key, _), ts, diff) in updates {
                        let row = key.unwrap().0.unwrap();
                        println!("{row}, {ts}, {diff}");
                    }
                }
            }
        }
    }
}
