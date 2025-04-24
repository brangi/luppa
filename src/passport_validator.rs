use std::path::Path;
use crate::models::*;
use crate::processing::*;
use crate::validation::*;
use crate::utils::PassportError;

pub struct PassportValidator {
    #[allow(dead_code)]
    country_rules: CountryRules,
}

impl PassportValidator {
    pub fn new() -> Self {
        PassportValidator {
            country_rules: CountryRules::new(),
        }
    }
    
    // Main validation function that orchestrates the entire process
    pub fn validate(&self, image_path: &Path) -> Result<ValidationResult, PassportError> {
        // Step 1: Process the image
        let processed_image = ImageProcessor::process_image(image_path)?;
        
        // Step 2: Extract MRZ data
        let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;
        
        // Step 3: Detect security features
        let security_features = SecurityProcessor::detect_security_features(&processed_image)?;
        
        // Step 4: Extract visual data
        let visual_data = OcrProcessor::extract_visual_data(&processed_image)?;
        
        // Step 5: Extract biometric data
        let biometric_data = BiometricProcessor::extract_biometric_data(&processed_image)?;
        
        // Step 6: Run all validation checks
        let validation_result = self.validate_all(
            mrz_data,
            security_features,
            visual_data,
            biometric_data
        )?;
        
        Ok(validation_result)
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
        
        // Step 6: Validate expiry
        let expiry_validation = ExpiryValidator::validate(&visual_data)?;
        
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
