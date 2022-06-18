use std::alloc::System;
use std::time::Duration;
use std::{time::SystemTime, sync::Arc};

use cv::Image;
use cv::setup_cv_loop;

use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio::time::sleep;

extern crate tensorflow;

mod cv;

#[tokio::main]
async fn main() {
    println!("Hello, world!");


    let last_sighting: Arc<RwLock<Option<DogSighting>>> = Arc::new(RwLock::new(None));

    setup_cv_loop(last_sighting);

    // TODO web server stuff
}


pub struct DogSighting {
    timestamp: SystemTime,
    image: Image
}

