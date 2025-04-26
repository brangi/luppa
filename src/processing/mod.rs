//! Passport Processing Module
//! 
//! This module provides the core processing functionality for passport OCR and validation:
//! - `image_processor`: Consolidated image preprocessing for OCR optimization
//! - `enhanced_ocr`: Universal, multilingual passport field extraction with MRZ parsing
//! - `security_features`: Combined security and biometric verification
//! - `batch_visual_verification`: Batch processing capabilities
//! - `field_correction`: Core processing modules for passport extraction and validation

pub mod image_processor;
pub mod enhanced_ocr;
pub mod extractors;
pub mod cleaning;
pub mod image_ops;
pub mod mrz;
pub mod security_features;
pub mod batch_visual_verification;
pub mod field_correction;

pub use image_processor::ImageProcessor;
pub use enhanced_ocr::{OcrProcessor, EnhancedOcrProcessor};
pub use security_features::SecurityProcessor;
pub use field_correction::FieldCorrection;
