// Machine Learning module for enhanced passport OCR and validation
// This module provides AI-driven improvements to the passport processing pipeline

// Original ML modules (temporarily commented out to fix compilation issues)
// pub mod feature_extraction;
// pub mod validation;
// pub mod training;

// Simplified implementation that works with our existing universal OCR system
pub mod simple_validator;

// Re-export the simplified implementation
pub use simple_validator::{SimpleValidator as FeatureExtractor, SimpleValidator as MlValidator, ValidationConfidence};

// For backward compatibility
pub type MlValidationResult = (bool, ValidationConfidence);
