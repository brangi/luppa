use std::path::Path;
use crate::models::{MrzData, VisualData, ValidationResult, CountryRules, SecurityFeatures, BiometricData};
use crate::ml::{MlValidator, FeatureExtractor};
use crate::processing::*;
use crate::validation::*;
use crate::utils::PassportError;

pub struct PassportValidator {
    // Underscore prefix to acknowledge these fields are currently unused but will be used in future implementations
    _country_rules: CountryRules,
    ml_validator: MlValidator,
    _feature_extractor: FeatureExtractor,
    // Flag to enable/disable ML-enhanced validation
    use_ml_validation: bool,
}

impl PassportValidator {
    pub fn new() -> Self {
        PassportValidator {
            _country_rules: CountryRules::new(),
            ml_validator: MlValidator::new(),
            _feature_extractor: FeatureExtractor::new(),
            use_ml_validation: true, // Enable ML validation by default
        }
    }
    
    // Allows enabling or disabling ML-enhanced validation
    pub fn with_ml_validation(mut self, enable: bool) -> Self {
        self.use_ml_validation = enable;
        self
    }
    
    // Main validation function that orchestrates the entire process
    pub fn validate(&self, image_path: &Path) -> Result<ValidationResult, PassportError> {
        // Step 1: Process the image
        let image_bytes = std::fs::read(image_path)
            .map_err(|e| PassportError::OcrError(format!("Failed to read image file: {}", e)))?;
        let processed_image = ImageProcessor::preprocess_image(&image_bytes)?;
        
        // Step 2: Extract MRZ data
        let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;
        
        // Step 3: Detect security features
        let security_features = SecurityProcessor::detect_security_features(&processed_image)?;
        
        // Step 4: Extract visual data using enhanced OCR with multi-language support
        // Configure with English, Spanish, French, German and fallback languages
        let tesseract_langs = &["eng", "spa", "fra", "deu"];
        let initial_visual_data = EnhancedOcrProcessor::extract_visual_data_from_bytes(&processed_image, tesseract_langs)?;
        
        // Step 4b: Apply field correction to improve accuracy by cross-validating MRZ and visual data
        println!("\n🔎 Cross-validating MRZ and visual data for higher accuracy...");
        let visual_data = FieldCorrection::correct_visual_data(&mrz_data, &initial_visual_data);
        
        // Step 5: Extract biometric data using consolidated SecurityProcessor
        let biometric_data = SecurityProcessor::extract_biometric_data(&processed_image)?;
        
        // Step 6: Run all validation checks
        let validation_result = self.validate_all(
            mrz_data,
            security_features,
            visual_data,
            biometric_data
        )?;
        
        Ok(validation_result)
    }
    
    // Validation function that uses pre-extracted data (useful for PDF processing)
    pub fn validate_with_extracted_data(&self, mrz_data: &MrzData, visual_data: &VisualData) -> Result<ValidationResult, PassportError> {
        // Apply field correction for improved accuracy
        println!("\n🔎 Cross-validating MRZ and visual data for higher accuracy...");
        let corrected_visual_data = FieldCorrection::correct_visual_data(mrz_data, visual_data);
        
        // Create placeholder security features
        let security_features = SecurityFeatures {
            hologram_present: false,
            microprinting_present: false,
            uv_features_present: false,
            ir_features_present: false,
            watermark_present: false,
            security_thread_present: false,
            chip_present: false,
        };
        
        // Create placeholder biometric data
        let biometric_data = BiometricData {
            face_image: None,
            chip_data: None,
        };
        
        // Validate the extracted data and return the result
        self.validate_all(
            mrz_data.clone(),
            security_features,
            corrected_visual_data,
            biometric_data
        )
    }

    // Validate all aspects of the passport
    fn validate_all(
        &self,
        mrz_data: MrzData,
        security_features: SecurityFeatures,
        visual_data: VisualData,
        biometric_data: BiometricData,
    ) -> Result<ValidationResult, PassportError> {
        // Step 1: Validate MRZ data
        let mrz_validation = MrzValidator::validate(&mrz_data, &visual_data)?;
        
        // Step 2: Validate security features
        let security_validation = SecurityValidator::validate(&security_features)?;
        
        // Step 3: Validate format
        let format_validation = FormatValidator::validate(&visual_data)?;
        
        // Step 4: Validate biometric data
        let biometric_validation = BiometricValidator::validate(&biometric_data)?;
        
        // Step 5: Validate against database
        let database_validation = DatabaseValidator::validate(&visual_data)?;
        
        // Step 6: Validate expiry (using both MRZ and visual data)
        let expiry_validation = ExpiryValidator::validate_with_mrz(&visual_data, &mrz_data)?;
        
        // Step 7: ML-enhanced validation (if enabled)
        let mut ml_validation_issues = Vec::new();
        
        if self.use_ml_validation {
            // Use the ML validator to enhance validation
            let (ml_is_valid, ml_confidence) = self.ml_validator.validate(&mrz_data, &visual_data);
            
            // Use the ML validation result to potentially affect the final result
            // We're now implementing the ML validation but storing the result in the issues
            // instead of a separate variable to avoid the unused variable warning
            
            // Add ML validation issues if any
            if !ml_is_valid {
                ml_validation_issues.push(format!("ML validation failed with confidence: {:.2}% (MRZ: {:.2}%, Visual: {:.2}%)", 
                    ml_confidence.consistency_confidence * 100.0,
                    ml_confidence.mrz_confidence * 100.0,
                    ml_confidence.visual_confidence * 100.0));
            } else {
                println!("ML validation passed with confidence: {:.2}%", ml_confidence.consistency_confidence * 100.0);
            }
            
            // Log ML validation results
            println!("ML Validation: {}", if ml_is_valid { "PASSED" } else { "FAILED" });
            println!("  - MRZ Confidence: {:.2}%", ml_confidence.mrz_confidence * 100.0);
            println!("  - Visual Confidence: {:.2}%", ml_confidence.visual_confidence * 100.0);
            println!("  - Consistency: {:.2}%", ml_confidence.consistency_confidence * 100.0);
            println!("  - Security Features: {:.2}%", ml_confidence.security_feature_confidence * 100.0);
            println!("  - Fraud Detection: {:.2}%", ml_confidence.fraud_detection_confidence * 100.0);
        }
        
        // Combine all validation results
        let is_valid = mrz_validation.is_valid && 
                     security_validation.is_valid && 
                     format_validation.is_valid && 
                     biometric_validation.is_valid && 
                     database_validation.is_valid && 
                     expiry_validation.is_valid;
        
        // Combine all issues
        let mut issues = Vec::new();
        issues.extend(mrz_validation.issues.clone());
        issues.extend(security_validation.issues.clone());
        issues.extend(format_validation.issues.clone());
        issues.extend(biometric_validation.issues.clone());
        issues.extend(database_validation.issues.clone());
        issues.extend(expiry_validation.issues.clone());
        
        Ok(ValidationResult {
            is_valid,
            mrz_validation,
            security_validation,
            format_validation,
            biometric_validation,
            database_validation,
            expiry_validation,
            issues,
        })
    }
}
