#[derive(Debug, Clone, PartialEq)]
pub enum DocumentFormat {
    TD1,  // ID Card (85.6mm × 54.0mm)
    TD2,  // ID Card (105.0mm × 74.0mm)
    TD3,  // Passport (125.0mm × 88.0mm)
    MRVA, // Visa Format-A (80.0mm × 120.0mm)
    MRVB, // Visa Format-B (74.0mm × 105.0mm)
}

impl DocumentFormat {
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            DocumentFormat::TD1 => (85.6, 54.0),   // mm
            DocumentFormat::TD2 => (105.0, 74.0),  // mm
            DocumentFormat::TD3 => (125.0, 88.0),  // mm
            DocumentFormat::MRVA => (80.0, 120.0), // mm
            DocumentFormat::MRVB => (74.0, 105.0), // mm
        }
    }

    pub fn mrz_lines(&self) -> usize {
        match self {
            DocumentFormat::TD1 => 3,
            DocumentFormat::TD2 => 2,
            DocumentFormat::TD3 => 2,
            DocumentFormat::MRVA => 2,
            DocumentFormat::MRVB => 2,
        }
    }

    pub fn mrz_chars_per_line(&self) -> usize {
        match self {
            DocumentFormat::TD1 => 30,
            DocumentFormat::TD2 => 36,
            DocumentFormat::TD3 => 44,
            DocumentFormat::MRVA => 44,
            DocumentFormat::MRVB => 36,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MrzData {
    pub document_format: Option<DocumentFormat>,
    pub document_type: String,
    pub issuing_country: String,
    pub document_number: String,
    pub surname: String,
    pub given_names: String,
    pub nationality: String,
    pub date_of_birth: String,
    pub gender: String,
    pub date_of_expiry: String,
    pub personal_number: Option<String>,
    pub optional_data: Option<String>,
    pub check_digits: CheckDigits,
    pub raw_mrz_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CheckDigits {
    pub document_number_check: char,
    pub date_of_birth_check: char,
    pub date_of_expiry_check: char,
    pub personal_number_check: char,
    pub composite_check: char,
}

#[derive(Debug, Clone)]
pub struct SecurityFeatures {
    pub hologram_present: bool,
    pub microprinting_present: bool,
    pub uv_features_present: bool,
    pub ir_features_present: bool,
    pub watermark_present: bool,
    pub security_thread_present: bool,
    pub chip_present: bool,
    pub optical_variable_device: bool,
    pub tactile_features: bool,
    pub perforations: bool,
    pub anti_scan_pattern: bool,
    pub security_fibers: bool,
    pub deliberate_errors: bool,
    pub level_1_features: Vec<String>, // Visual features
    pub level_2_features: Vec<String>, // Features requiring simple equipment
    pub level_3_features: Vec<String>, // Forensic features
}

#[derive(Debug, Clone)]
pub struct BiometricData {
    pub face_image: Option<Vec<u8>>,
    pub fingerprint_data: Option<Vec<Vec<u8>>>,
    pub iris_data: Option<Vec<Vec<u8>>>,
    pub chip_data: Option<ChipData>,
    pub face_quality: f64,
    pub fingerprint_quality: f64,
    pub iris_quality: f64,
    pub has_chip_requirement: bool,
    pub has_fingerprint_requirement: bool,
    pub has_iris_requirement: bool,
}

#[derive(Debug, Clone)]
pub struct ChipData {
    pub is_readable: bool,
    pub data_groups_present: Vec<String>,
    pub authentication_success: bool,
    pub basic_access_control: bool,
    pub extended_access_control: bool,
    pub pace_authentication: bool,
    pub active_authentication: bool,
    pub chip_authentication: bool,
    pub terminal_authentication: bool,
}

#[derive(Debug, Clone)]
pub struct VisualData {
    pub document_format: Option<DocumentFormat>,
    pub document_type: String,
    pub issuing_country: String,
    pub document_number: String,
    pub name: String,
    pub surname: String,
    pub given_names: String,
    pub nationality: String,
    pub date_of_birth: String,
    pub gender: String,
    pub place_of_birth: Option<String>,
    pub date_of_issue: String,
    pub date_of_expiry: String,
    pub authority: Option<String>,
    pub personal_number: Option<String>,
    pub portrait: Option<Vec<u8>>,
    pub signature: Option<Vec<u8>>,
    pub secondary_portrait: Option<Vec<u8>>, // Ghost image
    pub additional_fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ValidationIssueType {
    Mrz,
    Security,
    Format,
    Biometric,
    Database,
    Expiry,
    Generic,
    Chip,
    PKI,
    Certificate,
    DocumentAuthenticity,
    PersonalIdentification,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub mrz_validation: MrzValidationResult,
    pub security_validation: SecurityFeaturesValidationResult,
    pub format_validation: FormatValidationResult,
    pub biometric_validation: BiometricValidationResult,
    pub database_validation: DatabaseValidationResult,
    pub expiry_validation: ExpiryValidationResult,
    pub pki_validation: Option<PKIValidationResult>,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct PKIValidationResult {
    pub is_valid: bool,
    pub certificate_chain_valid: bool,
    pub certificate_not_revoked: bool,
    pub document_signer_valid: bool,
    pub country_signing_cert_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct MrzValidationResult {
    pub is_valid: bool,
    pub document_number_check_valid: bool,
    pub date_of_birth_check_valid: bool,
    pub date_of_expiry_check_valid: bool,
    pub personal_number_check_valid: bool,
    pub composite_check_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct SecurityFeaturesValidationResult {
    pub is_valid: bool,
    pub hologram_valid: bool,
    pub microprinting_valid: bool,
    pub uv_features_valid: bool,
    pub ir_features_valid: bool,
    pub watermark_valid: bool,
    pub security_thread_valid: bool,
    pub chip_valid: bool,
    pub optical_variable_device_valid: bool,
    pub tactile_features_valid: bool,
    pub perforations_valid: bool,
    pub anti_scan_pattern_valid: bool,
    pub security_fibers_valid: bool,
    pub deliberate_errors_valid: bool,
    pub level_1_features_valid: bool,
    pub level_2_features_valid: bool,
    pub level_3_features_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct BiometricValidationResult {
    pub is_valid: bool,
    pub face_matches: bool,
    pub fingerprint_matches: bool,
    pub iris_matches: bool,
    pub chip_authentic: bool,
    pub basic_access_control_valid: bool,
    pub extended_access_control_valid: bool,
    pub pace_valid: bool,
    pub active_authentication_valid: bool,
    pub chip_authentication_valid: bool,
    pub terminal_authentication_valid: bool,
    pub secondary_biometric_valid: bool,
    pub meets_icao_standards: bool,
    pub false_acceptance_rate: f32,
    pub false_rejection_rate: f32,
    pub verification_time_ms: u64,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct FormatValidationResult {
    pub is_valid: bool,
    pub correct_format: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct DatabaseValidationResult {
    pub is_valid: bool,
    pub in_database: bool,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct ExpiryValidationResult {
    pub is_valid: bool,
    pub not_expired: bool,
    pub issues: Vec<ValidationIssue>,
}
