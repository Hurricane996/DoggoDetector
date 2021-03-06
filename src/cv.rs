#![allow(non_snake_case)]

use std::{time::{Duration, SystemTime}, sync::Arc, thread};

use tokio::{sync::RwLock};
use opencv::{prelude::*, imgcodecs::imencode, core::Vector};
use opencv::{core, highgui, videoio, imgproc};
use crate::{DogSighting, double_buffer::DoubleBuffer};
use lazy_static::lazy_static;
use opencv::highgui::{poll_key, WINDOW_AUTOSIZE, WindowFlags};

const CV_LOOP_SLEEP_TIME: Duration = Duration::from_millis(30);

lazy_static! {
    static ref CV_MOTION_THRESHOLD: core::Scalar = core::Scalar::from(100 * 100);
}

pub fn setup_cv_loop(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let last_sighting = last_sighting.clone();
    thread::spawn(move || {
        let mut cv_subsystem = CVSubsystem::new(core::Rect { x: 0, y: 0, width: 640, height: 480 });
        println!("CV Subsystem initialization complete");

        loop {
            println!("Running dogcheck");
            match cv_subsystem.get_dog() {
                Ok(Some(image)) => {
                    let sighting = DogSighting {
                        image,
                        timestamp: SystemTime::now()
                    };

                    last_sighting.blocking_write().replace(sighting);
                    println!("Dog found")
                },
                Ok(None) => {println!("No dog found")},
                Err(e) => {
                    eprintln!("Failed to check image for dog, got error {e}")
                }
            }

 
            thread::sleep(CV_LOOP_SLEEP_TIME);
        }
    });
}

 

pub type Image = Vec<u8>;

pub struct CVSubsystem {
    // OpenCV highgui uses names instead of objects or handles
    windowName: String,
    camera: videoio::VideoCapture,
    lastFrame: Mat,
    regionOfInterest: core::Rect,
}

impl CVSubsystem {
    pub fn new(roi: core::Rect) -> Self {
        // initialize OpenCV
        let name: &str = "Dog Monitor";
        highgui::named_window("Dog Monitor", WINDOW_AUTOSIZE)
            .unwrap_or_else(|err| panic!("Unable to open window! Error {err}"));
        poll_key().expect("Failed to poll key");
        println!("Window open successful");

        let cam: videoio::VideoCapture = videoio::VideoCapture::new(0, videoio::CAP_V4L2)
           .unwrap_or_else(|err| panic!("Error while opening video capture! Error {err}"));
         match cam.is_opened() {
             Ok(false) => panic!("Failed to open camera!"),
             Err(e) => panic!("Error checking camera open status! Error {e}"),
            _ => { println!("Open successful")}
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
            regionOfInterest: roi,
        }
    }
    
    pub fn get_dog(&mut self) -> Result<Option<Vec<u8>>, opencv::Error> {
        poll_key()?;
        
        // grab a new frame from the camera
        let mut newFrame: DoubleBuffer<Mat> = DoubleBuffer::default();
        self.camera.read(newFrame.back())?;
        newFrame.swap();

        // keep a copy of the original image around, this is what we send to the end user if we find the frank
        let new_frame_original = newFrame.clone_front();
        // and also what we send to the display
        highgui::imshow(self.windowName.as_str(), &new_frame_original)?;
        poll_key();
        
        // run it through the vision pipeline
        // crop to the region of interest

        // convert to grayscale to reduce visual noise
        let (src, dst) = newFrame.buffers();
        imgproc::cvt_color(src, dst, imgproc::COLOR_BGR2GRAY, 0)?;
        newFrame.swap();

        // apply a bit of blur to further reduce visual noise. this frame will later be stored so it can be compared
        // to the next frame.
        let (src, dst) = newFrame.buffers();
        imgproc::blur(src, dst, core::Size::new(4, 4), core::Point::new(-1, -1), core::BORDER_DEFAULT)?;
        newFrame.swap();
        highgui::imshow("grayscale", &newFrame.clone_front())?;
        poll_key();
        // if we don't have a previous frame to compare to, just store the new frame and say there is no dog
        // this is a pretty inelegant way of handling initialization, but oh well
        if self.lastFrame.empty() {
            self.lastFrame = newFrame.to_front();
        } else {
            let mut difference: DoubleBuffer<Mat> = DoubleBuffer::default();
            // compute the absolute difference between the new frame and the old frame.
            core::absdiff(newFrame.front(), &self.lastFrame, difference.back())?;
            difference.swap();
            highgui::imshow("difference", &difference.clone_front())?;
            poll_key()?;
            
            // the opencv docs say this can be done in-place on an image
            let (src, dst) = difference.buffers();
            imgproc::threshold(src, dst, 50 as f64, 255 as f64, imgproc::THRESH_BINARY)?;
            difference.swap();
            highgui::imshow("threshold", &difference.clone_front())?;
            poll_key()?;
            
            let kernel: Mat = imgproc::get_structuring_element(imgproc::MORPH_ELLIPSE, core::Size::new(5, 5), core::Point::new(-1, -1))?;
            
            let (src, dst) = difference.buffers();
            imgproc::erode(src, dst, &kernel, core::Point::new(-1, -1), 3, core::BORDER_DEFAULT, core::Scalar::from(0))?;
            difference.swap();
            highgui::imshow("erode", &difference.clone_front())?;
            poll_key()?;

            let (src, dst) = difference.buffers();
            imgproc::dilate(src, dst, &kernel, core::Point::new(-1, -1), 6, core::BORDER_DEFAULT, core::Scalar::from(0))?;
            difference.swap();
            highgui::imshow("dilate", &difference.clone_front())?;
            poll_key()?;
            
            if core::sum_elems(difference.front())? > *CV_MOTION_THRESHOLD {
                // motion detected
                // todo machine learning magic
                self.lastFrame = newFrame.to_front();

                
                let mut buf = Vector::new();
                imencode(".png", &new_frame_original, &mut buf, &Vector::new())?;
                return Ok(Some(buf.to_vec()));
            }
        }
        return Ok(None);
    }
}