#[macro_use]
extern crate serde_derive;
extern crate log;
extern crate simple_logger;

use futures::StreamExt;
use kube::{
    api::{Object, Void, RawApi, Informer, WatchEvent},
    client::APIClient,
    config,
};
use log::{info, warn};

// This is our new Book struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Book {
    pub title: String,
    pub authors: Option<Vec<String>>,
}

// This is a convenience alias that describes the object we get from Kubernetes
type KubeBook = Object<Book, Void>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize simple_logger.
    //simple_logger::init_with_level(log::Level::Warn).unwrap();
    simple_logger::init().unwrap();

    // Load the kubeconfig file.
    let kubeconfig = config::load_kube_config().await?;

    // Create a new client
    let client = APIClient::new(kubeconfig);

    // Set a namespace. We're just hard-coding for now.
    let namespace = "default";

    // Describe the CRD we're working with.
    // This is basically the fields from our CRD definition.
    let resource = RawApi::customResource("books")
        .group("example.technosophos.com")
        .within(&namespace);

    // Create our informer and start listening.
    let informer = Informer::raw(client, resource).init().await?;
    loop {
        let mut events = informer.poll().await?.boxed();

        // Now we just do something each time a new book event is triggered.
        while let Some(event) = events.next().await {
            let event = event?;
            handle_events(event)?;
        }
    }
}

fn handle_events(event: WatchEvent<KubeBook>) -> anyhow::Result<()> {
    // This will receive events each time something happens
    match event {
        WatchEvent::Added(book) => {
            info!("Added a book {} with title '{}'", book.metadata.name, book.spec.title);
        },
        WatchEvent::Deleted(book) => {
            info!("Deleted a book {}", book.metadata.name);
        }
        _ => {
            warn!("another event");
        }
    }
    Ok(())
}
