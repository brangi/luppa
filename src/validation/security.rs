use crate::models::{
    DocumentFormat, SecurityFeatures, SecurityFeaturesValidationResult, ValidationIssue,
    ValidationIssueType,
};
use crate::processing::SecurityProcessor;
use crate::utils::PassportError;

pub struct SecurityValidator;

impl SecurityValidator {
    pub fn validate(
        security_features: &SecurityFeatures,
    ) -> Result<SecurityFeaturesValidationResult, PassportError> {
        // Default to TD3 (passport) format for backward compatibility
        Self::validate_with_format(security_features, &Some(DocumentFormat::TD3))
    }

    pub fn validate_with_format(
        security_features: &SecurityFeatures,
        document_format: &Option<DocumentFormat>,
    ) -> Result<SecurityFeaturesValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Use the SecurityProcessor to validate security features
        let security_valid =
            SecurityProcessor::validate_security_features(security_features, document_format);

        // Check basic security features
        let hologram_valid = security_features.hologram_present;
        if !hologram_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Hologram not detected".to_string(),
            });
        }

        // Check microprinting
        let microprinting_valid = security_features.microprinting_present;
        if !microprinting_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Microprinting not detected".to_string(),
            });
        }

        // Check UV features
        let uv_features_valid = security_features.uv_features_present;
        if !uv_features_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "UV features not detected".to_string(),
            });
        }

        // Check IR features
        let ir_features_valid = security_features.ir_features_present;
        if !ir_features_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "IR features not detected".to_string(),
            });
        }

        // Check watermark
        let watermark_valid = security_features.watermark_present;
        if !watermark_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Watermark not detected".to_string(),
            });
        }

        // Check security thread
        let security_thread_valid = security_features.security_thread_present;
        if !security_thread_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Security thread not detected".to_string(),
            });
        }

        // Check chip
        let chip_valid = security_features.chip_present;
        if !chip_valid && matches!(document_format, &Some(DocumentFormat::TD3)) {
            // Modern passports should have chips
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Chip not detected in eMRTD".to_string(),
            });
        }

        // Check additional security features
        let optical_variable_device_valid = security_features.optical_variable_device;
        let tactile_features_valid = security_features.tactile_features;
        let perforations_valid = security_features.perforations;
        let anti_scan_pattern_valid = security_features.anti_scan_pattern;
        let security_fibers_valid = security_features.security_fibers;
        let deliberate_errors_valid = security_features.deliberate_errors;

        // Check security levels
        let level_1_features_valid = !security_features.level_1_features.is_empty();
        let level_2_features_valid = !security_features.level_2_features.is_empty();
        let level_3_features_valid = !security_features.level_3_features.is_empty();

        if !level_1_features_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "No Level 1 security features detected".to_string(),
            });
        }

        if !level_2_features_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "No Level 2 security features detected".to_string(),
            });
        }

        // Overall validity
        let is_valid = security_valid && issues.is_empty();

        Ok(SecurityFeaturesValidationResult {
            is_valid,
            hologram_valid,
            microprinting_valid,
            uv_features_valid,
            ir_features_valid,
            watermark_valid,
            security_thread_valid,
            chip_valid,
            optical_variable_device_valid,
            tactile_features_valid,
            perforations_valid,
            anti_scan_pattern_valid,
            security_fibers_valid,
            deliberate_errors_valid,
            level_1_features_valid,
            level_2_features_valid,
            level_3_features_valid,
            issues,
        })
    }
}
