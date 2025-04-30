use crate::models::{FormatValidationResult, ValidationIssue, ValidationIssueType, VisualData};
use crate::utils::PassportError;

pub struct FormatValidator;

impl FormatValidator {
    pub fn validate(visual_data: &VisualData) -> Result<FormatValidationResult, PassportError> {
        let mut issues = Vec::new();
        let mut correct_format = true;

        // Check that required fields are present
        if visual_data.document_number.is_empty() {
            correct_format = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Document number is missing".to_string(),
            });
        }

        if visual_data.surname.is_empty() {
            correct_format = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Surname is missing".to_string(),
            });
        }

        if visual_data.given_names.is_empty() {
            correct_format = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Given names are missing".to_string(),
            });
        }

        if visual_data.date_of_birth.is_empty() {
            correct_format = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Date of birth is missing".to_string(),
            });
        }

        if visual_data.date_of_expiry.is_empty() {
            correct_format = false;
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Date of expiry is missing".to_string(),
            });
        }

        // Additional format checks could be performed here
        // - Document number format validation
        // - Date format validation
        // - Name format validation

        Ok(FormatValidationResult {
            is_valid: correct_format,
            correct_format,
            issues,
        })
    }
}
