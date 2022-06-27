use std::{sync::Arc, net::SocketAddr};

use axum::{Router, routing::get, body::{Body, self}, http::Response};
use tokio::sync::RwLock;

use crate::DogSighting;

use lazy_static::lazy_static;

lazy_static! {
    static ref ERROR_BYTES: Vec<u8> = "404".bytes().collect();
}

pub async fn setup_web_server(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let server = Router::<Body>::new();
    let server = {
        let last_sighting = last_sighting.clone();
        server.route("/sighting_time", get(|| async move {

            // the client will call sighting_time on poll to determine whether the dog has been sighted after the last notification.
            // if dog has been spotted, then the client will call the unfinished method that sends the actual image

            let timestamp = last_sighting.read().await.as_ref().map(|inner| inner.timestamp);

            serde_json::to_string(&timestamp).unwrap()
        }))
    };

    let server = server.route("/last_image.png", get(|| async move {
        let image = last_sighting.read().await;

        let data = match image.as_ref() {
            Some(image) => {
                Response::builder()
                // todo is there a way to avoid this clone
                    .body(body::Full::from(image.image.clone()))
            },
            None => {
                Response::builder()
                    .status(404)
                    .body(body::Full::from("No dog sighting"))
            },
        };

        data.unwrap()
    }));
        
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(server.into_make_service())
            .await
            .unwrap();
}