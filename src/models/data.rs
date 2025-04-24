#[derive(Debug, Clone)]
pub struct MrzData {
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
    pub check_digits: CheckDigits,
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
}

#[derive(Debug, Clone)]
pub struct BiometricData {
    pub face_image: Option<Vec<u8>>,
    pub chip_data: Option<ChipData>,
}

#[derive(Debug, Clone)]
pub struct ChipData {
    pub is_readable: bool,
    pub data_groups_present: Vec<String>,
    pub authentication_success: bool,
}

#[derive(Debug, Clone)]
pub struct VisualData {
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
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub struct BiometricValidationResult {
    pub is_valid: bool,
    pub face_matches: bool,
    pub chip_authentic: bool,
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
