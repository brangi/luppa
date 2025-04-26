//! Passport Processing Module
//! 
//! This module provides the core processing functionality for passport OCR and validation:
//! - `image_processor`: Consolidated image preprocessing for OCR optimization
//! - `enhanced_ocr`: Universal, multilingual passport field extraction with MRZ parsing
//! - `security_features`: Combined security and biometric verification
//! - `batch_visual_verification`: Batch processing capabilities

pub mod image_processor;
pub mod security_features;
pub mod enhanced_ocr;
pub mod batch_visual_verification;

pub use image_processor::ImageProcessor;
pub use enhanced_ocr::{OcrProcessor, EnhancedOcrProcessor};
pub use security_features::SecurityProcessor;
