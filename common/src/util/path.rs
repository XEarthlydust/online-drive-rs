use crate::module::error::AppError;
use std::path::Path;

#[derive(Debug)]
pub struct FilePathInfo {
    pub parent: String,
    pub file_name: String,
    pub extension: Option<String>,
}

impl FilePathInfo {
    pub fn from_str(path_str: &str) -> Result<Self, AppError> {
        let path = Path::new(path_str);

        let parent = path
            .parent()
            .and_then(|p| Some(p.to_string_lossy().to_string()))
            .ok_or(AppError::PathOrNameError)?;

        let file_name = path
            .file_name()
            .and_then(|f| Some(f.to_string_lossy().to_string()))
            .ok_or(AppError::PathOrNameError)?;

        let extension = path.extension().map(|e| e.to_string_lossy().to_string());

        Ok(FilePathInfo {
            parent,
            file_name,
            extension,
        })
    }
}
