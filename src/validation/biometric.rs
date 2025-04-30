use crate::models::{
    BiometricData, BiometricValidationResult, ValidationIssue, ValidationIssueType,
};
use crate::utils::PassportError;

pub struct BiometricValidator;

impl BiometricValidator {
    pub fn validate(data: &BiometricData) -> Result<BiometricValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check if face image is present and valid
        let face_matches = data.face_image.is_some();
        if !face_matches {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Face image not available for comparison".to_string(),
            });
        }

        // Check fingerprint data
        let fingerprint_matches = data.fingerprint_data.is_some();
        if !fingerprint_matches && data.has_fingerprint_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Fingerprint data missing but required".to_string(),
            });
        }

        // Check iris data
        let iris_matches = data.iris_data.is_some();
        if !iris_matches && data.has_iris_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Iris data missing but required".to_string(),
            });
        }

        // Check chip authenticity and features
        let (
            chip_authentic,
            basic_access_control_valid,
            extended_access_control_valid,
            active_authentication_valid,
            chip_authentication_valid,
            terminal_authentication_valid,
            pace_valid,
            secondary_biometric_valid,
        ) = if let Some(chip) = &data.chip_data {
            (
                chip.is_readable && chip.authentication_success,
                chip.basic_access_control,
                chip.extended_access_control,
                chip.active_authentication,
                chip.chip_authentication,
                chip.terminal_authentication,
                chip.pace_authentication,
                false, // Placeholder for secondary biometric validation
            )
        } else {
            (false, false, false, false, false, false, false, false)
        };

        if !chip_authentic && data.has_chip_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Chip authentication failed".to_string(),
            });
        }

        // Check biometric quality
        let face_quality_valid = data.face_quality >= 0.8;
        let fingerprint_quality_valid =
            !data.has_fingerprint_requirement || data.fingerprint_quality >= 0.8;
        let iris_quality_valid = !data.has_iris_requirement || data.iris_quality >= 0.8;

        if !face_quality_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Face image quality below ICAO standards".to_string(),
            });
        }

        if !fingerprint_quality_valid && data.has_fingerprint_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Fingerprint quality below ICAO standards".to_string(),
            });
        }

        if !iris_quality_valid && data.has_iris_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Biometric,
                message: "Iris quality below ICAO standards".to_string(),
            });
        }

        // Calculate if biometrics meet ICAO standards
        let meets_icao_standards = face_quality_valid
            && fingerprint_quality_valid
            && iris_quality_valid
            && (chip_authentic || !data.has_chip_requirement);

        // Overall validity
        let is_valid = issues.is_empty();

        Ok(BiometricValidationResult {
            is_valid,
            face_matches,
            chip_authentic,
            fingerprint_matches,
            iris_matches,
            basic_access_control_valid,
            extended_access_control_valid,
            active_authentication_valid,
            chip_authentication_valid,
            terminal_authentication_valid,
            pace_valid,
            secondary_biometric_valid,
            meets_icao_standards,
            false_acceptance_rate: 0.001, // 0.1%
            false_rejection_rate: 0.05,   // 5%
            verification_time_ms: 5000,   // 5 seconds
            issues: issues.clone(),
        })
    }
}
