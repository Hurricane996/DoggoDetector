
mod cv;
mod web;

use std::{time::SystemTime, sync::Arc};

use cv::Image;
use cv::setup_cv_loop;

use tokio::sync::RwLock;

use crate::web::setup_web_server;

extern crate tensorflow;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Hello, world!");

    let last_sighting: Arc<RwLock<Option<DogSighting>>> = Arc::new(RwLock::new(None));

    setup_cv_loop(last_sighting.clone());

    setup_web_server(last_sighting).await;
}


pub struct DogSighting {
    timestamp: SystemTime,
    image: Image
}

