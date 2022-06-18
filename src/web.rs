use std::{sync::Arc, net::SocketAddr};

use axum::{Router, routing::get, body::Body};
use tokio::sync::RwLock;

use crate::DogSighting;

pub async fn setup_web_server(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let server = Router::<Body>::new()
        .route("/sighting_time", get(|| async move {

            // the client will call sighting_time on poll to determine whether the dog has been sighted after the last notification.
            // if dog has been spotted, then the client will call the unfinished method that sends the actual image

            let timestamp = last_sighting.read().await.as_ref().map(|inner| inner.timestamp);

            serde_json::to_string(&timestamp).unwrap()
        }));
        
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(server.into_make_service())
            .await
            .unwrap();
}