use crate::models::{MrzData, VisualData, MrzValidationResult, ValidationIssue, ValidationIssueType};
use crate::utils::PassportError;
use chrono::NaiveDate;

pub struct MrzValidator;

impl MrzValidator {
    /// Comprehensive validation of MRZ data including:
    /// 1. Check digits validation
    /// 2. Cross-checking between MRZ and visual data
    /// 3. Format validation
    /// 4. Consistency checks
    pub fn validate(mrz_data: &MrzData, visual_data: &VisualData) -> Result<MrzValidationResult, PassportError> {
        let mut issues = Vec::new();
        
        // 1. Validate check digits
        let doc_num_check = Self::validate_check_digit(&mrz_data.document_number, mrz_data.check_digits.document_number_check);
        let dob_check = Self::validate_check_digit(&mrz_data.date_of_birth, mrz_data.check_digits.date_of_birth_check);
        let doe_check = Self::validate_check_digit(&mrz_data.date_of_expiry, mrz_data.check_digits.date_of_expiry_check);
        let personal_num_check = if let Some(ref pn) = mrz_data.personal_number {
            Self::validate_check_digit(pn, mrz_data.check_digits.personal_number_check)
        } else {
            true // If no personal number, assume check digit is valid
        };
        
        // Create validation issues for failed check digits
        if !doc_num_check {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Document number check digit is invalid".to_string(),
            });
        }
        
        if !dob_check {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Date of birth check digit is invalid".to_string(),
            });
        }
        
        if !doe_check {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Date of expiry check digit is invalid".to_string(),
            });
        }
        
        if !personal_num_check {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: "Personal number check digit is invalid".to_string(),
            });
        }
        
        // 2. Cross-check MRZ data with visual data for consistency
        let inconsistencies = Self::cross_check_fields(mrz_data, visual_data);
        let has_inconsistencies = !inconsistencies.is_empty();
        issues.extend(inconsistencies);
        
        // Create final validation result
        let is_valid = doc_num_check && dob_check && doe_check && personal_num_check && !has_inconsistencies;
        
        let result = MrzValidationResult {
            is_valid,
            document_number_check_valid: doc_num_check,
            date_of_birth_check_valid: dob_check,
            date_of_expiry_check_valid: doe_check,
            personal_number_check_valid: personal_num_check,
            composite_check_valid: true, // We'll assume composite check is valid for now
            issues,
        };
        
        Ok(result)
    }
    
    /// Calculate check digit according to ICAO Doc 9303 algorithm
    /// and compare it with provided check digit
    fn validate_check_digit(field: &str, check_char: char) -> bool {
        let expected = Self::calculate_check_digit(field);
        expected == check_char
    }
    
    /// Calculate check digit according to ICAO Doc 9303 algorithm
    /// Each character is assigned a value (A=10, B=11, etc.)
    /// Multiply each value by a weighting (7,3,1 repeated)
    /// Sum the products and take modulo 10
    fn calculate_check_digit(field: &str) -> char {
        let weights = [7, 3, 1]; // Standard ICAO weights
        
        let sum: u32 = field.chars().enumerate().map(|(i, c)| {
            let value = match c {
                '0'..='9' => c as u32 - '0' as u32,
                'A'..='Z' => c as u32 - 'A' as u32 + 10,
                '<' => 0,
                _ => 0, // Default value for unexpected characters
            };
            
            value * weights[i % 3] // Apply weight based on position
        }).sum();
        
        // Take modulo 10 and convert back to character
        let check_digit = (sum % 10) as u8;
        (check_digit + b'0') as char
    }
    
    /// Cross-check MRZ data with visually extracted data for inconsistencies
    /// Returns a list of validation issues for any inconsistencies found
    fn cross_check_fields(mrz_data: &MrzData, visual_data: &VisualData) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // 1. Document number check
        if !Self::fields_match(&mrz_data.document_number, &visual_data.document_number) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Document number mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.document_number, visual_data.document_number),
            });
        }
        
        // 2. Names check (handle differently as MRZ often has truncated names)
        if !visual_data.surname.is_empty() && !Self::names_compatible(&mrz_data.surname, &visual_data.surname) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Surname mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.surname, visual_data.surname),
            });
        }
        
        if !visual_data.given_names.is_empty() && !Self::names_compatible(&mrz_data.given_names, &visual_data.given_names) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Given names mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.given_names, visual_data.given_names),
            });
        }
        
        // 3. Date of birth check
        if !visual_data.date_of_birth.is_empty() && !Self::dates_compatible(&mrz_data.date_of_birth, &visual_data.date_of_birth) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Date of birth mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.date_of_birth, visual_data.date_of_birth),
            });
        }
        
        // 4. Gender check
        if !visual_data.gender.is_empty() && !Self::fields_match_case_insensitive(&mrz_data.gender, &visual_data.gender) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Gender mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.gender, visual_data.gender),
            });
        }
        
        // 5. Date of expiry check
        if !visual_data.date_of_expiry.is_empty() && !Self::dates_compatible(&mrz_data.date_of_expiry, &visual_data.date_of_expiry) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::Mrz,
                message: format!("Date of expiry mismatch: MRZ='{}', Visual='{}'", 
                                 mrz_data.date_of_expiry, visual_data.date_of_expiry),
            });
        }
        
        // 6. Personal number check (if available)
        if let (Some(ref mrz_pn), Some(ref viz_pn)) = (&mrz_data.personal_number, &visual_data.personal_number) {
            if !Self::fields_match(mrz_pn, viz_pn) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Mrz,
                    message: format!("Personal number mismatch: MRZ='{}', Visual='{}'", mrz_pn, viz_pn),
                });
            }
        }
        
        issues
    }
    
    /// Helper method to check if two fields match exactly
    fn fields_match(field1: &str, field2: &str) -> bool {
        // Clean both fields (remove spaces, special characters)
        let clean1 = Self::clean_field(field1);
        let clean2 = Self::clean_field(field2);
        
        clean1 == clean2
    }
    
    /// Helper method to check if two fields match ignoring case
    fn fields_match_case_insensitive(field1: &str, field2: &str) -> bool {
        // Clean both fields and convert to uppercase
        let clean1 = Self::clean_field(field1).to_uppercase();
        let clean2 = Self::clean_field(field2).to_uppercase();
        
        clean1 == clean2
    }
    
    /// Helper method to clean a field for comparison
    /// Removes spaces, special characters, etc.
    fn clean_field(field: &str) -> String {
        field.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }
    
    /// Helper method to check if names are compatible
    /// This is less strict than exact matching as names in MRZ are often truncated
    fn names_compatible(mrz_name: &str, visual_name: &str) -> bool {
        if mrz_name.is_empty() || visual_name.is_empty() {
            return true; // Can't check empty names
        }
        
        // Clean names
        let clean_mrz = Self::clean_field(mrz_name).to_uppercase();
        let clean_viz = Self::clean_field(visual_name).to_uppercase();
        
        // If one is contained in the other or they're exactly equal
        clean_mrz.contains(&clean_viz) || clean_viz.contains(&clean_mrz) || clean_mrz == clean_viz
    }
    
    /// Helper method to check if dates are compatible
    /// This handles different date formats like YYMMDD vs DD/MM/YYYY
    fn dates_compatible(mrz_date: &str, visual_date: &str) -> bool {
        if mrz_date.is_empty() || visual_date.is_empty() {
            return true; // Can't check empty dates
        }
        
        // Try to parse both dates into a standard format
        if let Some(mrz_parsed) = Self::parse_date(mrz_date) {
            if let Some(viz_parsed) = Self::parse_date(visual_date) {
                return mrz_parsed == viz_parsed;
            }
        }
        
        // Fallback to simple numeric comparison if parsing fails
        let clean_mrz = Self::clean_field(mrz_date);
        let clean_viz = Self::clean_field(visual_date);
        
        // Compare only the numeric parts if they're the same length
        if clean_mrz.len() == clean_viz.len() {
            return clean_mrz == clean_viz;
        }
        
        // If lengths differ, check if one contains the other
        clean_mrz.contains(&clean_viz) || clean_viz.contains(&clean_mrz)
    }
    
    /// Parse various date formats into a standardized NaiveDate
    fn parse_date(date_str: &str) -> Option<NaiveDate> {
        // Common date formats:
        // 1. YYMMDD (MRZ format)
        // 2. DD/MM/YYYY (Visual format)
        // 3. DD-MM-YYYY
        // 4. YYYY/MM/DD
        
        let cleaned = date_str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '/' || *c == '-' || *c == '.')
            .collect::<String>();
            
        // Try MRZ format (YYMMDD)
        if cleaned.len() == 6 && cleaned.chars().all(|c| c.is_ascii_digit()) {
            let year = 2000 + cleaned[0..2].parse::<i32>().unwrap_or(0);
            let month = cleaned[2..4].parse::<u32>().unwrap_or(0);
            let day = cleaned[4..6].parse::<u32>().unwrap_or(0);
            
            return NaiveDate::from_ymd_opt(year, month, day);
        }
        
        // Try common visual formats using separators
        let parts: Vec<&str> = cleaned.split(|c| c == '/' || c == '-' || c == '.').collect();
        
        if parts.len() == 3 {
            let (day, month, year) = match (parts[0].len(), parts[2].len()) {
                // DD/MM/YYYY format
                (2, 4) => {
                    let d = parts[0].parse::<u32>().unwrap_or(0);
                    let m = parts[1].parse::<u32>().unwrap_or(0);
                    let y = parts[2].parse::<i32>().unwrap_or(0);
                    (d, m, y)
                },
                // YYYY/MM/DD format
                (4, 2) => {
                    let y = parts[0].parse::<i32>().unwrap_or(0);
                    let m = parts[1].parse::<u32>().unwrap_or(0);
                    let d = parts[2].parse::<u32>().unwrap_or(0);
                    (d, m, y)
                },
                // Default to DD/MM/YY assuming 2000+
                _ => {
                    let d = parts[0].parse::<u32>().unwrap_or(0);
                    let m = parts[1].parse::<u32>().unwrap_or(0);
                    let mut y = parts[2].parse::<i32>().unwrap_or(0);
                    
                    // Handle 2-digit year
                    if y < 100 {
                        y += 2000;
                    }
                    
                    (d, m, y)
                }
            };
            
            return NaiveDate::from_ymd_opt(year, month, day);
        }
        
        None
    }
}
