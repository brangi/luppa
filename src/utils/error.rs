use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PassportError {
    ImageProcessingError(String),
    MrzExtractionError(String),
    MrzParsingError(String),
    SecurityFeatureDetectionError(String),
    FormatError(String),
    BiometricExtractionError(String),
    ValidationError(String),
    IoError(String),
    DatabaseError(String),
    CountryRuleNotFound(String),
    InvalidDate(String),
}

impl fmt::Display for PassportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PassportError::ImageProcessingError(msg) => {
                write!(f, "Image processing error: {}", msg)
            }
            PassportError::MrzExtractionError(msg) => write!(f, "MRZ extraction error: {}", msg),
            PassportError::MrzParsingError(msg) => write!(f, "MRZ parsing error: {}", msg),
            PassportError::SecurityFeatureDetectionError(msg) => {
                write!(f, "Security feature detection error: {}", msg)
            }
            PassportError::FormatError(msg) => write!(f, "Format error: {}", msg),
            PassportError::BiometricExtractionError(msg) => {
                write!(f, "Biometric extraction error: {}", msg)
            }
            PassportError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PassportError::IoError(msg) => write!(f, "IO error: {}", msg),
            PassportError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            PassportError::CountryRuleNotFound(msg) => write!(f, "Country rule not found: {}", msg),
            PassportError::InvalidDate(msg) => write!(f, "Invalid date: {}", msg),
        }
    }
}

impl Error for PassportError {}
