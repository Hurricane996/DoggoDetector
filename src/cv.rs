use std::{time::{Duration, SystemTime}, sync::Arc};

use tokio::{sync::RwLock, time::sleep};
use opencv::prelude::*;
use opencv::{core, highgui, imgcodecs, videoio, imgproc};
use opencv::core::ToInputArray;
use crate::DogSighting;


const CV_LOOP_SLEEP_TIME: Duration = Duration::from_secs(2);
const CV_MOTION_THRESHOLD: core::Scalar = core::Scalar::from(100 * 100);

pub fn setup_cv_loop(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let cv_subsystem = CVSubsystem::new(core::Rect { x: 0, y: 0, width: 640, height: 480 });

    let last_sighting = last_sighting.clone();
    /*tokio::spawn(async move {
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
    });*/
}

pub type Image = Mat;

//#[allow(non_snake_case)]
pub struct CVSubsystem {
    // OpenCV highgui uses names instead of objects or handles
    windowName: String,
    camera: videoio::VideoCapture,
    lastFrame: Mat,
    regionOfInterest: core::Rect
}

impl CVSubsystem {
    pub fn new(roi: core::Rect) -> Self {
        // initialize OpenCV
        let name: &str = "Dog Monitor";
        highgui::named_window("Dog Monitor", 0)
            .unwrap_or_else(|err| panic!("Unable to open window! Error {err}"));
        let cam: videoio::VideoCapture = videoio::VideoCapture::new(0, videoio::CAP_ANY)
            .unwrap_or_else(|err| panic!("Error while opening video capture! Error {err}"));
        match cam.is_opened() {
            Ok(false) => panic!("Failed to open camera!"),
            Err(e) => panic!("Error checking camera open status! Error {e}"),
            _ => { }
        }
        // interim segfault testing
        // if this doesn't segfault, the rest of OpenCV should work
        /*print!("Starting up opencv...");
        match opencv::core::have_opencl() {
            Ok(true) => print!("OpenCL is supported."),
            Ok(false) => print!("OpenCL is not supported."),
            Err(e) => print!("OpenCV died i guess. Error: {e}")
        }*/
        Self {
            windowName: String::from(name),
            camera: cam,
            lastFrame: Mat::default(),
            regionOfInterest: roi
        }
    }
    
    pub fn get_dog(&mut self) -> Result<Option<Mat>, opencv::Error> {
        // grab a new frame from the camera
        let mut newFrame: Mat = Mat::default();
        self.camera.read(&mut newFrame)?;
        
        // run it through the vision pipeline
        // crop to the region of interest
        
        // convert to grayscale to reduce visual noise
        imgproc::cvt_color(&newFrame, &mut newFrame, imgproc::COLOR_BGR2GRAY, 0)?;
        // apply a bit of blur to further reduce visual noise. this frame will later be stored so it can be compared
        // to the next frame.
        imgproc::blur(&newFrame, &mut newFrame, core::Size::new(4, 4), core::Point::new(-1, -1), core::BORDER_DEFAULT)?;
        
        // if we don't have a previous frame to compare to, so just store the new frame and say there is no dog
        // this is a pretty inelegant way of handling initialization, but oh well
        if self.lastFrame.empty() {
            self.lastFrame = newFrame;
        } else {
            let mut difference: Mat = Mat::default();
            // compute the absolute difference between the new frame and the old frame.
            core::absdiff(&newFrame, &self.lastFrame, &mut difference)?;
            let kernel: Mat = imgproc::get_structuring_element(imgproc::MORPH_RECT, core::Size::new(5, 5), core::Point::new(-1, -1))?;
            imgproc::erode(&difference, &mut difference, &kernel, core::Point::new(-1, -1), 3, core::BORDER_DEFAULT, core::Scalar::from(0))?;
            imgproc::dilate(&difference, &mut difference, &kernel, core::Point::new(-1, -1), 3, core::BORDER_DEFAULT, core::Scalar::from(0))?;
            
            if core::sum_elems(&difference)? > CV_MOTION_THRESHOLD {
                // motion detected
                // todo machine learning magic
                self.lastFrame = newFrame;
                return Ok(Some(self.lastFrame.clone()));
            }
        }
        return Ok(None);
    }
}