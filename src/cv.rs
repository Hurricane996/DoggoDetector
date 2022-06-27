use std::{time::{Duration, SystemTime}, sync::Arc, thread, ffi::c_void};

use tokio::{sync::RwLock, time::sleep, task};
use opencv::{prelude::*, imgcodecs::imencode, core::{Vector, _InputArray}};
use opencv::{core, highgui, imgcodecs, videoio, imgproc};
use opencv::core::ToInputArray;
use crate::DogSighting;
use lazy_static::lazy_static;


const CV_LOOP_SLEEP_TIME: Duration = Duration::from_secs(2);

lazy_static! {
    static ref CV_MOTION_THRESHOLD: core::Scalar = core::Scalar::from(100 * 100);
}

pub fn setup_cv_loop(last_sighting: Arc<RwLock<Option<DogSighting>>>) {
    let mut cv_subsystem = CVSubsystem::new(core::Rect { x: 0, y: 0, width: 640, height: 480 });

    let last_sighting = last_sighting.clone();
    thread::spawn(move || {
        loop {
            match cv_subsystem.get_dog() {
                Ok(Some(image)) => {
                    let sighting = DogSighting {
                        image,
                        timestamp: SystemTime::now()
                    };

                    last_sighting.blocking_write().replace(sighting);
                },
                Ok(None) => {},
                Err(e) => {
                    eprintln!("Failed to check image for dog, got error {e}")
                }
            }

 
            sleep(CV_LOOP_SLEEP_TIME);
        }
    });
}


trait DuplicatePointer {
    unsafe fn duplicate_pointer(&mut self) -> DuplicatedPointer;
}

impl DuplicatePointer for Mat {
    unsafe fn duplicate_pointer(&mut self) -> DuplicatedPointer {
        DuplicatedPointer{
            ptr: self.as_raw_mut()
        }
    }
}

struct DuplicatedPointer {
    ptr: *mut c_void
}

impl ToInputArray for DuplicatedPointer {
    fn input_array(&self) -> opencv::Result<core::_InputArray> {
        unsafe {
            Ok(_InputArray::from_raw(self.ptr))
        }
    }
}


pub type Image = Vec<u8>;

#[allow(non_snake_case)]
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
            regionOfInterest: roi,
        }
    }
    
    pub fn get_dog(&mut self) -> Result<Option<Vec<u8>>, opencv::Error> {
        // grab a new frame from the camera
        let mut newFrame: Mat = Mat::default();
        self.camera.read(&mut newFrame)?;
        
        // run it through the vision pipeline
        // crop to the region of interest
        
        // explicitly converting to an input array clones the pointer because bad library design
        // but im not complaining because it makes solving the problem easier

        unsafe {
            // convert to grayscale to reduce visual noise
            imgproc::cvt_color(&newFrame.duplicate_pointer(), &mut newFrame, imgproc::COLOR_BGR2GRAY, 0)?;
            // apply a bit of blur to further reduce visual noise. this frame will later be stored so it can be compared
            // to the next frame.
            imgproc::blur(&newFrame.duplicate_pointer(), &mut newFrame, core::Size::new(4, 4), core::Point::new(-1, -1), core::BORDER_DEFAULT)?;
        }
        // if we don't have a previous frame to compare to, so just store the new frame and say there is no dog
        // this is a pretty inelegant way of handling initialization, but oh well
        if self.lastFrame.empty() {
            self.lastFrame = newFrame;
        } else {
            let mut difference: Mat = Mat::default();
            // compute the absolute difference between the new frame and the old frame.
            core::absdiff(&newFrame, &self.lastFrame, &mut difference)?;
            let kernel: Mat = imgproc::get_structuring_element(imgproc::MORPH_RECT, core::Size::new(5, 5), core::Point::new(-1, -1))?;
            unsafe {
                imgproc::erode(&difference.duplicate_pointer(), &mut difference, &kernel, core::Point::new(-1, -1), 3, core::BORDER_DEFAULT, core::Scalar::from(0))?;
                imgproc::dilate(&difference.duplicate_pointer(), &mut difference, &kernel, core::Point::new(-1, -1), 3, core::BORDER_DEFAULT, core::Scalar::from(0))?;
            }
            if core::sum_elems(&difference)? > *CV_MOTION_THRESHOLD {
                // motion detected
                // todo machine learning magic
                self.lastFrame = newFrame;

                
                let mut buf = Vector::new();
                imencode(".png", &self.lastFrame, &mut buf, &Vector::new())?;
                return Ok(Some(buf.to_vec()));
            }
        }
        return Ok(None);
    }
}