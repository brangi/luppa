use crate::models::{
    DocumentFormat, ExpiryValidationResult, FormatValidationResult, MrzData,
    MrzValidationResult, ValidationIssue, ValidationIssueType, ValidationResult, VisualData
};
use crate::utils::PassportError;

/// MRTD Verifier for validating Machine Readable Travel Documents
/// according to ICAO Doc 9303 standards
pub struct MRTDVerifier;

impl MRTDVerifier {
    /// Create a new MRTD verifier
    pub fn new() -> Self {
        MRTDVerifier
    }

    /// Verify an MRTD document according to ICAO Doc 9303 standards
    pub fn verify(
        &self,
        _image_data: &[u8],
        mrz_data: &MrzData,
        visual_data: &VisualData,
    ) -> Result<ValidationResult, PassportError> {
        // Collect validation issues
        let mut issues = Vec::new();

        // 1. Validate MRZ data
        let mrz_validation = self.validate_mrz(mrz_data, visual_data)?;
        issues.extend(mrz_validation.issues.clone());

        // 2. Validate document format
        let format_validation = self.validate_format(mrz_data)?;
        issues.extend(format_validation.issues.clone());

        // 3. Validate expiry
        let expiry_validation = self.validate_expiry(mrz_data)?;
        issues.extend(expiry_validation.issues.clone());

        // Determine overall validity
        let is_valid = mrz_validation.is_valid
            && format_validation.is_valid
            && expiry_validation.is_valid;

        Ok(ValidationResult {
            is_valid,
            mrz_validation,
            format_validation,
            expiry_validation,
            issues,
        })
    }

    /// Validate MRZ data according to ICAO Doc 9303 standards
    fn validate_mrz(
        &self,
        mrz_data: &MrzData,
        visual_data: &VisualData,
    ) -> Result<MrzValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check document number
        let document_number_check_valid = mrz_data.document_number == visual_data.document_number;
        if !document_number_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Document number mismatch between MRZ and visual data".to_string(),
            });
        }

        // Check date of birth
        let date_of_birth_check_valid = mrz_data.date_of_birth == visual_data.date_of_birth;
        if !date_of_birth_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Date of birth mismatch between MRZ and visual data".to_string(),
            });
        }

        // Check date of expiry
        let date_of_expiry_check_valid = mrz_data.date_of_expiry == visual_data.date_of_expiry;
        if !date_of_expiry_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Date of expiry mismatch between MRZ and visual data".to_string(),
            });
        }

        // Check personal number if present
        let personal_number_check_valid = match (&mrz_data.personal_number, &visual_data.personal_number) {
            (Some(mrz_pn), Some(vis_pn)) => mrz_pn == vis_pn,
            (None, None) => true,
            _ => false,
        };
        if !personal_number_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Personal number mismatch between MRZ and visual data".to_string(),
            });
        }

        // Check composite fields
        let composite_check_valid = document_number_check_valid
            && date_of_birth_check_valid
            && date_of_expiry_check_valid
            && personal_number_check_valid;

        Ok(MrzValidationResult {
            is_valid: composite_check_valid,
            document_number_check_valid,
            date_of_birth_check_valid,
            date_of_expiry_check_valid,
            personal_number_check_valid,
            composite_check_valid,
            issues,
        })
    }

    /// Validate document format according to ICAO Doc 9303 standards
    fn validate_format(&self, mrz_data: &MrzData) -> Result<FormatValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check if document format is supported
        let format_supported = mrz_data.document_format.is_some();
        if !format_supported {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: "Unknown document format".to_string(),
            });
            return Ok(FormatValidationResult {
                is_valid: false,
                correct_format: false,
                issues,
            });
        }

        // Get valid document types for the format
        let valid_doc_types = match mrz_data.document_format.as_ref().unwrap() {
            DocumentFormat::TD1 => vec!["I", "ID"],
            DocumentFormat::TD2 => vec!["I", "ID"],
            DocumentFormat::TD3 => vec!["P", "PA"],
            DocumentFormat::MRVA => vec!["V", "VA"],
            DocumentFormat::MRVB => vec!["V", "VB"],
        };

        if !valid_doc_types.contains(&mrz_data.document_type.as_str()) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: format!(
                    "Invalid document type '{}' for format {:?}",
                    mrz_data.document_type, mrz_data.document_format
                ),
            });
        }

        // Check if all validations passed
        let is_valid = issues.is_empty();
        let correct_format = is_valid;

        Ok(FormatValidationResult {
            is_valid,
            correct_format,
            issues,
        })
    }

    /// Validate document expiry according to ICAO Doc 9303 standards
    fn validate_expiry(&self, mrz_data: &MrzData) -> Result<ExpiryValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Parse expiry date
        let expiry_date = chrono::NaiveDate::parse_from_str(&mrz_data.date_of_expiry, "%y%m%d")
            .map_err(|_| PassportError::InvalidDate("Invalid expiry date format".to_string()))?;

        // Check if document is expired
        let today = chrono::Local::now().naive_local().date();
        let not_expired = expiry_date >= today;

        if !not_expired {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Expiry,
                message: format!("Document expired on {}", expiry_date),
            });
        }

        Ok(ExpiryValidationResult {
            is_valid: not_expired,
            not_expired,
            issues,
        })
    }
}
