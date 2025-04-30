use crate::models::{
    BiometricData, BiometricValidationResult, DatabaseValidationResult, DocumentFormat,
    ExpiryValidationResult, FormatValidationResult, MrzData, MrzValidationResult,
    PKIValidationResult, SecurityFeatures, SecurityFeaturesValidationResult, ValidationIssue,
    ValidationIssueType, ValidationResult,
};
use crate::processing::biometric::BiometricType;
use crate::processing::pki::RevocationStatus;
use crate::processing::{BiometricProcessor, PKIProcessor, SecurityProcessor};
use crate::utils::PassportError;
use std::collections::HashMap;

/// MRTD Verifier for validating Machine Readable Travel Documents
/// according to ICAO Doc 9303 standards
pub struct MRTDVerifier {
    trusted_certificates: HashMap<String, Vec<u8>>,
    master_lists: HashMap<String, Vec<u8>>,
}

impl MRTDVerifier {
    /// Create a new MRTD verifier
    pub fn new() -> Self {
        MRTDVerifier {
            trusted_certificates: HashMap::new(),
            master_lists: HashMap::new(),
        }
    }

    /// Add a trusted certificate to the verifier
    pub fn add_trusted_certificate(&mut self, name: &str, certificate_data: Vec<u8>) {
        self.trusted_certificates
            .insert(name.to_string(), certificate_data);
    }

    /// Add a master list to the verifier
    pub fn add_master_list(&mut self, country_code: &str, master_list_data: Vec<u8>) {
        self.master_lists
            .insert(country_code.to_string(), master_list_data);
    }

    /// Verify an MRTD document (simplified interface for PassportValidator)
    pub fn verify(
        &self,
        image_data: &[u8],
        mrz_data: &MrzData,
        security_features: &SecurityFeatures,
        biometric_data: &BiometricData,
    ) -> Result<ValidationResult, PassportError> {
        // Use document_format from MRZ data if available, otherwise default to TD3
        let _document_format = if let Some(format) = &mrz_data.document_format {
            format
        } else {
            &DocumentFormat::TD3
        };

        // Collect validation issues
        let mut issues = Vec::new();

        // 1. Validate MRZ data
        let mrz_validation = self.validate_mrz(mrz_data)?;
        issues.extend(mrz_validation.issues.clone());

        // 2. Validate security features
        let security_validation =
            self.validate_security_features(security_features, &mrz_data.document_format)?;
        issues.extend(security_validation.issues.clone());

        // 3. Validate format
        let format_validation = self.validate_format(mrz_data)?;
        issues.extend(format_validation.issues.clone());

        // 4. Validate biometrics
        let biometric_validation =
            self.validate_biometrics(biometric_data, &mrz_data.document_format, None)?;
        issues.extend(biometric_validation.issues.clone());

        // 5. Validate against database
        let database_validation = self.validate_against_database(mrz_data)?;
        issues.extend(database_validation.issues.clone());

        // 6. Validate expiry
        let expiry_validation = self.validate_expiry(mrz_data)?;
        issues.extend(expiry_validation.issues.clone());

        // 7. Validate PKI
        let pki_validation = self.validate_pki(image_data, biometric_data, mrz_data)?;
        issues.extend(pki_validation.issues.clone());

        // Overall validity
        let is_valid = mrz_validation.is_valid
            && security_validation.is_valid
            && format_validation.is_valid
            && biometric_validation.is_valid
            && database_validation.is_valid
            && expiry_validation.is_valid
            && pki_validation.is_valid;

        Ok(ValidationResult {
            is_valid,
            mrz_validation,
            security_validation,
            format_validation,
            biometric_validation,
            database_validation,
            expiry_validation,
            pki_validation: Some(pki_validation),
            issues,
        })
    }

    /// Verify an MRTD document according to ICAO Doc 9303 standards
    pub fn verify_document(
        &self,
        mrz_data: &MrzData,
        security_features: &SecurityFeatures,
        biometric_data: &BiometricData,
        document_data: &[u8],
        live_biometric_capture: Option<&[u8]>,
    ) -> Result<ValidationResult, PassportError> {
        // Collect validation issues
        let mut issues = Vec::new();

        // 1. Validate MRZ data
        let mrz_validation = self.validate_mrz(mrz_data)?;
        issues.extend(mrz_validation.issues.clone());

        // 2. Validate security features
        let security_validation =
            self.validate_security_features(security_features, &mrz_data.document_format)?;
        issues.extend(security_validation.issues.clone());

        // 3. Validate document format
        let format_validation = self.validate_format(mrz_data)?;
        issues.extend(format_validation.issues.clone());

        // 4. Validate biometric data
        let biometric_validation = self.validate_biometrics(
            biometric_data,
            &mrz_data.document_format,
            live_biometric_capture,
        )?;
        issues.extend(biometric_validation.issues.clone());

        // 5. Validate against database
        let database_validation = self.validate_against_database(mrz_data)?;
        issues.extend(database_validation.issues.clone());

        // 6. Validate expiry
        let expiry_validation = self.validate_expiry(mrz_data)?;
        issues.extend(expiry_validation.issues.clone());

        // 7. Validate PKI (for electronic documents)
        let pki_validation = if biometric_data.chip_data.is_some() {
            let validation = self.validate_pki(document_data, biometric_data, mrz_data)?;
            issues.extend(validation.issues.clone());
            Some(validation)
        } else {
            None
        };

        // Determine overall validity
        let is_valid = mrz_validation.is_valid
            && security_validation.is_valid
            && format_validation.is_valid
            && biometric_validation.is_valid
            && database_validation.is_valid
            && expiry_validation.is_valid
            && pki_validation.as_ref().map_or(true, |v| v.is_valid);

        Ok(ValidationResult {
            is_valid,
            mrz_validation,
            security_validation,
            format_validation,
            biometric_validation,
            database_validation,
            expiry_validation,
            pki_validation,
            issues,
        })
    }

    /// Validate MRZ data according to ICAO Doc 9303 standards
    fn validate_mrz(&self, mrz_data: &MrzData) -> Result<MrzValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check document type is valid for the format
        let valid_doc_types = match &mrz_data.document_format {
            Some(DocumentFormat::TD1) | Some(DocumentFormat::TD2) => vec!["ID", "A"],
            Some(DocumentFormat::TD3) => vec!["P"],
            Some(DocumentFormat::MRVA) | Some(DocumentFormat::MRVB) => vec!["V"],
            None => vec!["P", "ID", "A", "V"], // Allow all document types if format is unknown
        };

        if !valid_doc_types.contains(&mrz_data.document_type.as_str()) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!(
                    "Invalid document type '{}' for format {:?}",
                    mrz_data.document_type, mrz_data.document_format
                ),
            });
        }

        // Validate check digits
        let document_number_check_valid = self.validate_check_digit(
            &mrz_data.document_number,
            mrz_data.check_digits.document_number_check,
        );

        if !document_number_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Invalid document number check digit".to_string(),
            });
        }

        // Extract date of birth as YYMMDD for check digit validation
        let dob_parts: Vec<&str> = mrz_data.date_of_birth.split_whitespace().collect();
        let dob_for_check = if dob_parts.len() == 3 {
            let year = dob_parts[2].chars().skip(2).collect::<String>();
            let month = dob_parts[1];
            let day = dob_parts[0];
            format!("{}{}{}", year, month, day)
        } else {
            "000000".to_string()
        };

        let date_of_birth_check_valid =
            self.validate_check_digit(&dob_for_check, mrz_data.check_digits.date_of_birth_check);

        if !date_of_birth_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Invalid date of birth check digit".to_string(),
            });
        }

        // Extract expiry date as YYMMDD for check digit validation
        let exp_parts: Vec<&str> = mrz_data.date_of_expiry.split_whitespace().collect();
        let exp_for_check = if exp_parts.len() == 3 {
            let year = exp_parts[2].chars().skip(2).collect::<String>();
            let month = exp_parts[1];
            let day = exp_parts[0];
            format!("{}{}{}", year, month, day)
        } else {
            "000000".to_string()
        };

        let date_of_expiry_check_valid =
            self.validate_check_digit(&exp_for_check, mrz_data.check_digits.date_of_expiry_check);

        if !date_of_expiry_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Invalid date of expiry check digit".to_string(),
            });
        }

        // Validate personal number check digit if present
        let personal_number_check_valid = if let Some(personal_number) = &mrz_data.personal_number {
            self.validate_check_digit(personal_number, mrz_data.check_digits.personal_number_check)
        } else {
            true
        };

        if !personal_number_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Invalid personal number check digit".to_string(),
            });
        }

        // Validate composite check digit
        // In a real implementation, this would calculate the composite check
        // over the appropriate fields based on the document format
        let composite_check_valid = true; // Placeholder

        if !composite_check_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Invalid composite check digit".to_string(),
            });
        }

        // Check if all validations passed
        let is_valid = document_number_check_valid
            && date_of_birth_check_valid
            && date_of_expiry_check_valid
            && personal_number_check_valid
            && composite_check_valid;

        Ok(MrzValidationResult {
            is_valid,
            document_number_check_valid,
            date_of_birth_check_valid,
            date_of_expiry_check_valid,
            personal_number_check_valid,
            composite_check_valid,
            issues,
        })
    }

    /// Validate a check digit according to ICAO Doc 9303 standards
    fn validate_check_digit(&self, _data: &str, _check_digit: char) -> bool {
        // ICAO Doc 9303 check digit calculation
        // Each character is assigned a value: A-Z (10-35), 0-9 (0-9), < (0)
        // Multiply each value by a weight based on position (7, 3, 1 repeating)
        // Sum the results and take modulo 10

        // Placeholder implementation - in a real system this would implement
        // the actual ICAO check digit algorithm

        // For demonstration purposes, we'll just return true
        // In a real implementation, this would calculate the check digit
        // and compare it with the provided check digit
        true
    }

    /// Validate security features according to ICAO Doc 9303 standards
    fn validate_security_features(
        &self,
        security_features: &SecurityFeatures,
        document_format: &Option<DocumentFormat>,
    ) -> Result<SecurityFeaturesValidationResult, PassportError> {
        // Use the SecurityProcessor to validate security features
        let security_valid =
            SecurityProcessor::validate_security_features(security_features, document_format);

        let mut issues = Vec::new();

        // Check basic security features
        if !security_features.hologram_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Hologram not detected".to_string(),
            });
        }

        if !security_features.microprinting_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Microprinting not detected".to_string(),
            });
        }

        if !security_features.uv_features_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "UV features not detected".to_string(),
            });
        }

        if !security_features.ir_features_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "IR features not detected".to_string(),
            });
        }

        if !security_features.watermark_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Watermark not detected".to_string(),
            });
        }

        if !security_features.security_thread_present {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Security,
                message: "Security thread not detected".to_string(),
            });
        }

        // Check electronic security features for eMRTDs
        let chip_valid = if security_features.chip_present {
            true // Placeholder - would check chip authentication in real implementation
        } else {
            if matches!(document_format, Some(DocumentFormat::TD3)) {
                // Modern passports should have chips
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Security,
                    message: "Chip not detected in eMRTD".to_string(),
                });
                false
            } else {
                // Other document types may not require chips
                true
            }
        };

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

        Ok(SecurityFeaturesValidationResult {
            is_valid: security_valid && issues.is_empty(),
            hologram_valid: security_features.hologram_present,
            microprinting_valid: security_features.microprinting_present,
            uv_features_valid: security_features.uv_features_present,
            ir_features_valid: security_features.ir_features_present,
            watermark_valid: security_features.watermark_present,
            security_thread_valid: security_features.security_thread_present,
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

    /// Validate document format according to ICAO Doc 9303 standards
    fn validate_format(&self, mrz_data: &MrzData) -> Result<FormatValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check if MRZ data matches the expected format
        let expected_lines = mrz_data
            .document_format
            .as_ref()
            .map_or(3, |format| format.mrz_lines());
        let actual_lines = mrz_data.raw_mrz_lines.len();

        if expected_lines != actual_lines {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Format,
                message: format!(
                    "Expected {} MRZ lines for format {:?}, found {}",
                    expected_lines, mrz_data.document_format, actual_lines
                ),
            });
        }

        // Check if MRZ line lengths match the expected format
        let expected_chars = mrz_data
            .document_format
            .as_ref()
            .map_or(44, |format| format.mrz_chars_per_line());
        for (i, line) in mrz_data.raw_mrz_lines.iter().enumerate() {
            if line.len() != expected_chars {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Format,
                    message: format!(
                        "Expected {} characters in MRZ line {}, found {}",
                        expected_chars,
                        i + 1,
                        line.len()
                    ),
                });
            }
        }

        // Check if document type is valid for the format
        let valid_doc_types = match &mrz_data.document_format {
            Some(DocumentFormat::TD1) | Some(DocumentFormat::TD2) => vec!["ID", "A"],
            Some(DocumentFormat::TD3) => vec!["P"],
            Some(DocumentFormat::MRVA) | Some(DocumentFormat::MRVB) => vec!["V"],
            None => vec!["P", "ID", "A", "V"], // Accept any valid type if format is unknown
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

    /// Validate biometric data according to ICAO Doc 9303 standards
    fn validate_biometrics(
        &self,
        biometric_data: &BiometricData,
        document_format: &Option<DocumentFormat>,
        live_capture: Option<&[u8]>,
    ) -> Result<BiometricValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check biometric data quality
        let quality_result =
            BiometricProcessor::validate_biometric_quality(biometric_data, document_format);

        if !quality_result.meets_icao_standards {
            for issue in &quality_result.issues {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Biometric,
                    message: issue.clone(),
                });
            }
        }

        // Check face biometric match if live capture is provided
        let face_matches = if let Some(live_data) = live_capture {
            if biometric_data.face_image.is_some() {
                let verification_result = BiometricProcessor::verify_biometrics(
                    biometric_data,
                    live_data,
                    BiometricType::Face,
                );

                if !verification_result.success {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::Biometric,
                        message: format!(
                            "Face verification failed with confidence score: {:.2}",
                            verification_result.confidence_score
                        ),
                    });
                }

                verification_result.success
            } else {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Biometric,
                    message: "No face image available for verification".to_string(),
                });
                false
            }
        } else {
            // No live capture provided, so we can't verify
            true
        };

        // Check chip authentication for eMRTDs
        let chip_authentic = if let Some(chip_data) = &biometric_data.chip_data {
            chip_data.authentication_success
        } else {
            if matches!(document_format, Some(DocumentFormat::TD3)) {
                // Modern passports should have chips
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Biometric,
                    message: "No chip data available for eMRTD".to_string(),
                });
                false
            } else {
                // Other document types may not require chips
                true
            }
        };

        // Check additional eMRTD authentication methods
        let (
            basic_access_control_valid,
            extended_access_control_valid,
            pace_valid,
            active_authentication_valid,
            chip_authentication_valid,
            terminal_authentication_valid,
        ) = if let Some(chip_data) = &biometric_data.chip_data {
            (
                chip_data.basic_access_control,
                chip_data.extended_access_control,
                chip_data.pace_authentication,
                chip_data.active_authentication,
                chip_data.chip_authentication,
                chip_data.terminal_authentication,
            )
        } else {
            (true, true, true, true, true, true) // Not applicable if no chip
        };

        // Check if all validations passed
        let is_valid = face_matches
            && chip_authentic
            && basic_access_control_valid
            && (extended_access_control_valid
                || !matches!(document_format, Some(DocumentFormat::TD3)))
            && (pace_valid || !matches!(document_format, Some(DocumentFormat::TD3)))
            && (active_authentication_valid
                || !matches!(document_format, Some(DocumentFormat::TD3)));

        // Use ICAO performance requirements
        let false_acceptance_rate = 0.001; // 0.1%
        let false_rejection_rate = 0.05; // 5%
        let verification_time_ms = 5000; // 5 seconds

        Ok(BiometricValidationResult {
            is_valid,
            face_matches,
            fingerprint_matches: true, // Placeholder
            iris_matches: true,        // Placeholder
            chip_authentic,
            basic_access_control_valid,
            extended_access_control_valid,
            pace_valid,
            active_authentication_valid,
            chip_authentication_valid,
            terminal_authentication_valid,
            secondary_biometric_valid: true, // Placeholder
            meets_icao_standards: true,      // Placeholder
            false_acceptance_rate,
            false_rejection_rate,
            verification_time_ms,
            issues,
        })
    }

    /// Validate document against database
    fn validate_against_database(
        &self,
        _mrz_data: &MrzData,
    ) -> Result<DatabaseValidationResult, PassportError> {
        // Placeholder implementation
        // In a real implementation, this would:
        // 1. Connect to a database of valid documents
        // 2. Check if the document exists and is valid
        // 3. Check against watch lists

        let mut issues = Vec::new();

        // Simulate database check
        let in_database = true;

        if !in_database {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Database,
                message: "Document not found in database".to_string(),
            });
        }

        // Check if document is on a watch list
        let on_watch_list = false; // Placeholder

        if on_watch_list {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Database,
                message: "Document is on a watch list".to_string(),
            });
        }

        // Check if all validations passed
        let is_valid = in_database && !on_watch_list;

        Ok(DatabaseValidationResult {
            is_valid,
            in_database,
            issues,
        })
    }

    /// Validate document expiry
    fn validate_expiry(&self, mrz_data: &MrzData) -> Result<ExpiryValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Parse expiry date
        let exp_parts: Vec<&str> = mrz_data.date_of_expiry.split_whitespace().collect();
        if exp_parts.len() != 3 {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Expiry,
                message: "Invalid expiry date format".to_string(),
            });

            return Ok(ExpiryValidationResult {
                is_valid: false,
                not_expired: false,
                issues,
            });
        }

        // Extract day, month, year
        let day = exp_parts[0].parse::<u32>().unwrap_or(0);
        let month = exp_parts[1].parse::<u32>().unwrap_or(0);
        let year = exp_parts[2].parse::<u32>().unwrap_or(0);

        // Check if date is valid
        if day < 1 || day > 31 || month < 1 || month > 12 || year < 2000 {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Expiry,
                message: "Invalid expiry date components".to_string(),
            });

            return Ok(ExpiryValidationResult {
                is_valid: false,
                not_expired: false,
                issues,
            });
        }

        // Get current date (simplified for demonstration)
        let current_year = 2025;
        let current_month = 4;
        let current_day = 26;

        // Check if document is expired
        let not_expired = (year > current_year)
            || (year == current_year && month > current_month)
            || (year == current_year && month == current_month && day >= current_day);

        if !not_expired {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Expiry,
                message: "Document is expired".to_string(),
            });
        }

        // Check if all validations passed
        let is_valid = not_expired;

        Ok(ExpiryValidationResult {
            is_valid,
            not_expired,
            issues,
        })
    }

    /// Validate PKI for electronic documents
    fn validate_pki(
        &self,
        _document_data: &[u8],
        biometric_data: &BiometricData,
        mrz_data: &MrzData,
    ) -> Result<PKIValidationResult, PassportError> {
        let mut issues = Vec::new();

        // Check if chip data is available
        let _chip_data = match &biometric_data.chip_data {
            Some(data) => data,
            None => {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::PKI,
                    message: "No chip data available for PKI validation".to_string(),
                });

                return Ok(PKIValidationResult {
                    is_valid: false,
                    certificate_chain_valid: false,
                    certificate_not_revoked: false,
                    document_signer_valid: false,
                    country_signing_cert_valid: false,
                    issues,
                });
            }
        };

        // Validate certificate chain
        let cert_validation = PKIProcessor::validate_certificate_chain(mrz_data)?;

        if !cert_validation.is_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Certificate,
                message: "Certificate chain validation failed".to_string(),
            });
        }

        // Check certificate revocation status
        let certificate_not_revoked = match cert_validation.revocation_status {
            RevocationStatus::NotRevoked => true,
            RevocationStatus::Revoked => {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Certificate,
                    message: "Certificate has been revoked".to_string(),
                });
                false
            }
            RevocationStatus::Unknown => {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Certificate,
                    message: "Certificate revocation status unknown".to_string(),
                });
                false
            }
        };

        // Check document signer certificate
        let document_signer_valid = cert_validation.document_signer_valid;
        if !document_signer_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Certificate,
                message: "Document signer certificate is invalid".to_string(),
            });
        }

        // Check country signing certificate
        let country_signing_cert_valid = cert_validation.country_signing_cert_valid;
        if !country_signing_cert_valid {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Certificate,
                message: "Country signing certificate is invalid".to_string(),
            });
        }

        // Check if all validations passed
        let is_valid = cert_validation.is_valid
            && certificate_not_revoked
            && document_signer_valid
            && country_signing_cert_valid;

        Ok(PKIValidationResult {
            is_valid,
            certificate_chain_valid: cert_validation.is_valid,
            certificate_not_revoked,
            document_signer_valid,
            country_signing_cert_valid,
            issues,
        })
    }
}
