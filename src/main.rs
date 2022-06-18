use std::{time::SystemTime, sync::Arc};

use mock_cv::Image;
use mock_cv::CVSubsystem;

use tokio::sync::Mutex;
use tokio::sync::RwLock;


mod mock_cv;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let cv_subsystem = CVSubsystem::new();

    let last_dog_sighting: Arc<RwLock<Option<DogSighting>>> = Arc::new(RwLock::new(None));

    {
        let last_dog_sighting = last_dog_sighting.clone();
        tokio::spawn(async move {
            loop {
                if let Some(sighting) = cv_subsystem.get_dog() {
                    s = last_dog_sighting.write().await;
                }
            }
        });
    }
}

pub struct DogSighting {
    timestamp: SystemTime,
    image: Image
}

