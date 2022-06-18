pub struct Image {}

pub struct CVSubsystem {}

impl CVSubsystem {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_dog(&self ) -> Option<Image> {
        Some(Image {})
    }
}