use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("No input files were provided")]
    EmptyInput,
    #[error("Output path must end with .pdf")]
    InvalidOutputPath,
    #[error("Input file does not exist: {0}")]
    MissingInputFile(String),
    #[error("Unsupported file kind for path: {0}")]
    UnsupportedInput(String),
    #[error("No supported input files were selected")]
    NoSupportedInputs,
    #[error("Failed to read file: {0}")]
    Io(String),
    #[error("Failed to decode image: {0}")]
    Image(String),
    #[error("Failed to process PDF: {0}")]
    Pdf(String),
    #[error("Failed to save merged PDF: {0}")]
    Write(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let payload = match self {
            AppError::EmptyInput => ErrorPayload {
                code: "empty_input",
                message: self.to_string(),
            },
            AppError::InvalidOutputPath => ErrorPayload {
                code: "invalid_output_path",
                message: self.to_string(),
            },
            AppError::MissingInputFile(_) => ErrorPayload {
                code: "missing_input_file",
                message: self.to_string(),
            },
            AppError::UnsupportedInput(_) => ErrorPayload {
                code: "unsupported_input",
                message: self.to_string(),
            },
            AppError::NoSupportedInputs => ErrorPayload {
                code: "no_supported_inputs",
                message: self.to_string(),
            },
            AppError::Io(_) => ErrorPayload {
                code: "io_error",
                message: self.to_string(),
            },
            AppError::Image(_) => ErrorPayload {
                code: "image_error",
                message: self.to_string(),
            },
            AppError::Pdf(_) => ErrorPayload {
                code: "pdf_error",
                message: self.to_string(),
            },
            AppError::Write(_) => ErrorPayload {
                code: "write_error",
                message: self.to_string(),
            },
        };

        payload.serialize(serializer)
    }
}
