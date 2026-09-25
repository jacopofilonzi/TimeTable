use std::sync::Arc;

use super::{University, unicam::Unicam};
use crate::errors::{AppError, NotFound};

/// All supported universities.
pub struct Registry {
    universities: Vec<Arc<dyn University>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            universities: vec![Arc::new(Unicam::new())],
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn University>> {
        self.universities.iter()
    }

    /// Case-insensitive lookup by id.
    pub fn get(&self, id: &str) -> Result<&Arc<dyn University>, AppError> {
        self.universities
            .iter()
            .find(|u| u.info().id.eq_ignore_ascii_case(id))
            .ok_or_else(|| NotFound::new(format!("University '{id}' is not supported")).into())
    }
}
