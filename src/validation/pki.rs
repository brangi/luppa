use crate::models::{
    BiometricData, MrzData, PKIValidationResult, ValidationIssue, ValidationIssueType,
};
use crate::utils::PassportError;

pub struct PkiValidator;

impl PkiValidator {
    pub fn validate(
        mrz_data: &MrzData,
        biometric_data: &BiometricData,
    ) -> Result<PKIValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check if chip data is available for PKI validation
        let has_chip_data = biometric_data.chip_data.is_some();
        if !has_chip_data && biometric_data.has_chip_requirement {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Chip data required for PKI validation is missing".to_string(),
            });
        }

        // Get document signing certificate validation status
        let (document_signing_cert_valid, country_signing_cert_valid, certificate_chain_valid) =
            if has_chip_data {
                let result = crate::processing::PKIProcessor::validate_certificate_chain(mrz_data);
                match result {
                    Ok(r) => (r.document_signer_valid, r.country_signing_cert_valid, true), // Use true as placeholder for certificate_chain_valid
                    Err(_) => (false, false, false),
                }
            } else {
                (false, false, false)
            };

        if !document_signing_cert_valid && has_chip_data {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Document signing certificate validation failed".to_string(),
            });
        }

        if !country_signing_cert_valid && has_chip_data {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Country signing certificate validation failed".to_string(),
            });
        }

        // Check revocation status
        let (not_revoked, revocation_check_performed) = if has_chip_data {
            let result = crate::processing::PKIProcessor::check_revocation_status(mrz_data);
            (result.not_revoked, result.check_performed)
        } else {
            (false, false)
        };

        if !not_revoked && revocation_check_performed {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Certificate has been revoked".to_string(),
            });
        }

        // Check document signature
        let document_signature_valid = if has_chip_data {
            crate::processing::PKIProcessor::validate_document_signature(mrz_data, biometric_data)
        } else {
            false
        };

        if !document_signature_valid && has_chip_data {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Document signature validation failed".to_string(),
            });
        }

        // Check security object (SOD)
        let security_object_valid = if has_chip_data {
            crate::processing::PKIProcessor::validate_security_object(biometric_data)
        } else {
            false
        };

        if !security_object_valid && has_chip_data {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Security Object (SOD) validation failed".to_string(),
            });
        }

        // Check passive authentication
        let _passive_authentication_valid = document_signature_valid && security_object_valid;

        // Check ICAO PKD status
        let (in_icao_pkd, pkd_check_performed) = if has_chip_data {
            let result = crate::processing::PKIProcessor::check_icao_pkd(mrz_data);
            (result.in_pkd, result.check_performed)
        } else {
            (false, false)
        };

        if !in_icao_pkd && pkd_check_performed {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::PKI,
                message: "Certificate not found in ICAO PKD".to_string(),
            });
        }

        // Overall validity
        let is_valid = !biometric_data.has_chip_requirement
            || (document_signing_cert_valid
                && country_signing_cert_valid
                && certificate_chain_valid
                && not_revoked
                && document_signature_valid
                && security_object_valid
                && (in_icao_pkd || !pkd_check_performed));

        Ok(PKIValidationResult {
            is_valid,
            document_signer_valid: document_signing_cert_valid,
            country_signing_cert_valid,
            certificate_chain_valid,
            certificate_not_revoked: not_revoked,
            issues,
        })
    }
}
