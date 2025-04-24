use crate::models::{BiometricData, BiometricValidationResult, ValidationIssue, ValidationIssueType};
use crate::utils::PassportError;

pub struct BiometricValidator;

impl BiometricValidator {
    pub fn validate(data: &BiometricData) -> Result<BiometricValidationResult, PassportError> {
        let mut issues = Vec::new();
        let mut is_valid = true;
        
        // Check if face image is present and valid
        let face_matches = data.face_image.is_some();
        if !face_matches {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Face image not available for comparison".to_string(),
            });
            // For demo, we don't fail validation if face image is missing
        }
        
        // Check chip authenticity
        let chip_authentic = if let Some(chip) = &data.chip_data {
            chip.is_readable && chip.authentication_success
        } else {
            false
        };
        
        if !chip_authentic {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Chip authentication failed".to_string(),
            });
        }
        
        Ok(BiometricValidationResult {
            is_valid,
            face_matches,
            chip_authentic,
            issues,
        })
    }
}
