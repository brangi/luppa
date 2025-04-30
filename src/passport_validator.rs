use crate::models::*;
use crate::processing::*;
use crate::utils::PassportError;
use crate::validation::*;
use crate::verification::MRTDVerifier;
use std::path::Path;

pub struct PassportValidator;

impl PassportValidator {
    pub fn new() -> Self {
        PassportValidator
    }

    // Main validation function that orchestrates the entire process
    pub fn validate(&self, image_path: &Path) -> Result<ValidationResult, PassportError> {
        // Step 1: Process the image
        let processed_image = ImageProcessor::process_image(image_path)?;

        // Step 2: Extract MRZ data
        let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;

        // Step 3: Extract visual data
        let visual_data = OcrProcessor::extract_visual_data(&processed_image)?;

        // Step 4: Run all validation checks
        let validation_result = self.validate_all(
            mrz_data.clone(),
            visual_data.clone(),
        )?;

        // Step 5: Run MRTD verification if available (for ICAO Doc 9303 compliance)
        let mrtd_verifier = MRTDVerifier::new();
        let _mrtd_verification = mrtd_verifier.verify(
            &processed_image,
            &mrz_data,
            &visual_data,
        );

        Ok(validation_result)
    }

    // Validate all aspects of the passport
    fn validate_all(
        &self,
        mrz_data: MrzData,
        visual_data: VisualData,
    ) -> Result<ValidationResult, PassportError> {
        // Step 1: Validate MRZ data
        let mrz_validation = MrzValidator::validate(&mrz_data, &visual_data)?;

        // Step 2: Validate format
        let format_validation = FormatValidator::validate(&visual_data)?;

        // Step 3: Validate expiry
        let expiry_validation = ExpiryValidator::validate(&visual_data)?;

        // Combine all validation results
        let is_valid = mrz_validation.is_valid
            && format_validation.is_valid
            && expiry_validation.is_valid;

        // Combine all issues
        let mut issues = Vec::new();
        issues.extend(mrz_validation.issues.clone());
        issues.extend(format_validation.issues.clone());
        issues.extend(expiry_validation.issues.clone());

        Ok(ValidationResult {
            is_valid,
            mrz_validation,
            format_validation,
            expiry_validation,
            issues,
        })
    }
}
