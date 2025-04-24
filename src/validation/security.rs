use crate::models::{SecurityFeatures, SecurityFeaturesValidationResult, ValidationIssue, ValidationIssueType};
use crate::utils::PassportError;

pub struct SecurityValidator;

impl SecurityValidator {
    pub fn validate(features: &SecurityFeatures) -> Result<SecurityFeaturesValidationResult, PassportError> {
        let mut issues = Vec::new();
        let mut is_valid = true;
        
        // Validate each security feature
        let hologram_valid = features.hologram_present;
        if !hologram_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Hologram not detected".to_string(),
            });
        }
        
        let microprinting_valid = features.microprinting_present;
        if !microprinting_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Microprinting not detected".to_string(),
            });
        }
        
        let uv_features_valid = features.uv_features_present;
        if !uv_features_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "UV features not detected".to_string(),
            });
        }
        
        let ir_features_valid = features.ir_features_present;
        if !ir_features_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "IR features not detected".to_string(),
            });
        }
        
        let watermark_valid = features.watermark_present;
        if !watermark_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Watermark not detected".to_string(),
            });
        }
        
        let security_thread_valid = features.security_thread_present;
        if !security_thread_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Security thread not detected".to_string(),
            });
        }
        
        let chip_valid = features.chip_present;
        if !chip_valid {
            is_valid = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Chip not detected".to_string(),
            });
        }
        
        Ok(SecurityFeaturesValidationResult {
            is_valid,
            hologram_valid,
            microprinting_valid,
            uv_features_valid,
            ir_features_valid,
            watermark_valid,
            security_thread_valid,
            chip_valid,
            issues,
        })
    }
}
