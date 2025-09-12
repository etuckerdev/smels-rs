pub mod common;

use crate::ErrorInfo;

pub trait Rule: Send + Sync {
    fn apply(&self, errors: &[ErrorInfo]) -> Vec<Cause>;
}

#[derive(Debug)]
pub struct Cause {
    pub description: String,
    pub confidence: f32,
    pub fixes: Vec<String>,
}
