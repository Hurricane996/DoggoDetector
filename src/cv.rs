use std::{time::{Duration, SystemTime}, sync::Arc};

use tokio::{sync::RwLock, time::sleep};
use opencv::prelude::*;
use opencv::{highgui, imgcodecs};
use crate::DogSighting;


const CV_LOOP_SLEEP_TIME: Duration = Duration::from_secs(2);

pub fn setup_cv_loop(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let cv_subsystem = CVSubsystem::new();


    let last_sighting = last_sighting.clone();
    tokio::spawn(async move {
        loop {
            if let Some(image) = cv_subsystem.get_dog() {
                let sighting = DogSighting {
                    image,
                    timestamp: SystemTime::now()
                };

                last_sighting.write().await.replace(sighting);
            }
            sleep(CV_LOOP_SLEEP_TIME).await;
        }
    });
}


pub struct Image {}

pub struct CVSubsystem {

}

impl CVSubsystem {
    pub fn new() -> Self {
        // initialize OpenCV
        //highgui::named_window("Dog Monitor", 0)
        //    .unwrap_or_else(|err| panic!("OpenCV Initialization Failed! Error {err}"));
        // interim segfault testing
        // if this doesn't segfault, the rest of OpenCV should work
        match opencv::core::have_opencl() {
            Ok(true) => print!("OpenCL is supported."),
            Ok(false) => print!("OpenCL is not supported."),
            Err(e) => print!("OpenCV died i guess. Error: {e}")
        }
        Self {}
    }
    
    pub fn get_dog(&self ) -> Option<Image> {
        Some(Image {})
    }
}