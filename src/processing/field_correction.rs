use crate::models::{MrzData, VisualData};
use std::collections::HashMap;

/// FieldCorrection provides automatic correction for visually extracted fields
/// by cross-validating with MRZ data which typically has higher reliability
/// due to its error-checking mechanisms.
pub struct FieldCorrection;

impl FieldCorrection {
    /// Create a corrected version of VisualData by reconciling differences
    /// with the more reliable MRZ data
    pub fn correct_visual_data(mrz_data: &MrzData, visual_data: &VisualData) -> VisualData {
        // Start with the original visual data
        let mut corrected = visual_data.clone();
        
        // Keep track of which fields were corrected for logging/debugging
        let mut corrections = HashMap::new();
        
        // Document number correction (prioritize MRZ as it has error checking)
        if !visual_data.document_number.is_empty() && 
           !Self::fields_match(&mrz_data.document_number, &visual_data.document_number) {
            corrections.insert("document_number", 
                format!("Visual: {} -> MRZ: {}", visual_data.document_number, mrz_data.document_number));
            corrected.document_number = mrz_data.document_number.clone();
        } else if visual_data.document_number.is_empty() && !mrz_data.document_number.is_empty() {
            // If visual field is empty but MRZ has data, use MRZ data
            corrected.document_number = mrz_data.document_number.clone();
            corrections.insert("document_number", 
                format!("Empty -> MRZ: {}", mrz_data.document_number));
        }
        
        // Names correction (carefully, as MRZ often has truncated names)
        if !visual_data.surname.is_empty() && 
           !Self::names_compatible(&mrz_data.surname, &visual_data.surname) {
            // If names don't match but MRZ name is very short (likely truncated),
            // prefer the visual data which likely has the full name
            if mrz_data.surname.len() < visual_data.surname.len() && mrz_data.surname.len() < 20 {
                // Keep visual data but log the discrepancy
                corrections.insert("surname_discrepancy", 
                    format!("Visual: {} vs MRZ: {} (kept visual due to possible MRZ truncation)", 
                           visual_data.surname, mrz_data.surname));
            } else {
                // Otherwise prefer MRZ data
                corrected.surname = mrz_data.surname.clone();
                corrections.insert("surname", 
                    format!("Visual: {} -> MRZ: {}", visual_data.surname, mrz_data.surname));
            }
        } else if visual_data.surname.is_empty() && !mrz_data.surname.is_empty() {
            corrected.surname = mrz_data.surname.clone();
            corrections.insert("surname", 
                format!("Empty -> MRZ: {}", mrz_data.surname));
        }
        
        // Similar approach for given names
        if !visual_data.given_names.is_empty() && 
           !Self::names_compatible(&mrz_data.given_names, &visual_data.given_names) {
            if mrz_data.given_names.len() < visual_data.given_names.len() && mrz_data.given_names.len() < 20 {
                corrections.insert("given_names_discrepancy", 
                    format!("Visual: {} vs MRZ: {} (kept visual due to possible MRZ truncation)", 
                           visual_data.given_names, mrz_data.given_names));
            } else {
                corrected.given_names = mrz_data.given_names.clone();
                corrections.insert("given_names", 
                    format!("Visual: {} -> MRZ: {}", visual_data.given_names, mrz_data.given_names));
            }
        } else if visual_data.given_names.is_empty() && !mrz_data.given_names.is_empty() {
            corrected.given_names = mrz_data.given_names.clone();
            corrections.insert("given_names", 
                format!("Empty -> MRZ: {}", mrz_data.given_names));
        }
        
        // Update full name if components were corrected
        if corrections.contains_key("surname") || corrections.contains_key("given_names") {
            corrected.name = format!("{} {}", corrected.given_names.trim(), corrected.surname.trim()).trim().to_string();
            corrections.insert("name", 
                format!("Updated to: {}", corrected.name));
        }
        
        // Nationality correction
        if !visual_data.nationality.is_empty() && 
           !Self::fields_match_case_insensitive(&mrz_data.nationality, &visual_data.nationality) {
            corrected.nationality = mrz_data.nationality.clone();
            corrections.insert("nationality", 
                format!("Visual: {} -> MRZ: {}", visual_data.nationality, mrz_data.nationality));
        } else if visual_data.nationality.is_empty() && !mrz_data.nationality.is_empty() {
            corrected.nationality = mrz_data.nationality.clone();
            corrections.insert("nationality", 
                format!("Empty -> MRZ: {}", mrz_data.nationality));
        }
        
        // Date of birth correction
        if !visual_data.date_of_birth.is_empty() && 
           !Self::dates_compatible(&mrz_data.date_of_birth, &visual_data.date_of_birth) {
            // MRZ dates are more reliable due to check digits
            corrected.date_of_birth = Self::format_mrz_date(&mrz_data.date_of_birth);
            corrections.insert("date_of_birth", 
                format!("Visual: {} -> MRZ: {}", visual_data.date_of_birth, corrected.date_of_birth));
        } else if visual_data.date_of_birth.is_empty() && !mrz_data.date_of_birth.is_empty() {
            corrected.date_of_birth = Self::format_mrz_date(&mrz_data.date_of_birth);
            corrections.insert("date_of_birth", 
                format!("Empty -> MRZ: {}", corrected.date_of_birth));
        }
        
        // Date of expiry correction
        if !visual_data.date_of_expiry.is_empty() && 
           !Self::dates_compatible(&mrz_data.date_of_expiry, &visual_data.date_of_expiry) {
            corrected.date_of_expiry = Self::format_mrz_date(&mrz_data.date_of_expiry);
            corrections.insert("date_of_expiry", 
                format!("Visual: {} -> MRZ: {}", visual_data.date_of_expiry, corrected.date_of_expiry));
        } else if visual_data.date_of_expiry.is_empty() && !mrz_data.date_of_expiry.is_empty() {
            corrected.date_of_expiry = Self::format_mrz_date(&mrz_data.date_of_expiry);
            corrections.insert("date_of_expiry", 
                format!("Empty -> MRZ: {}", corrected.date_of_expiry));
        }
        
        // Gender correction
        if !visual_data.gender.is_empty() && 
           !Self::fields_match_case_insensitive(&mrz_data.gender, &visual_data.gender) {
            corrected.gender = mrz_data.gender.clone();
            corrections.insert("gender", 
                format!("Visual: {} -> MRZ: {}", visual_data.gender, mrz_data.gender));
        } else if visual_data.gender.is_empty() && !mrz_data.gender.is_empty() {
            corrected.gender = mrz_data.gender.clone();
            corrections.insert("gender", 
                format!("Empty -> MRZ: {}", mrz_data.gender));
        }
        
        // Personal number correction if available in both
        if let (Some(ref mrz_pn), Some(ref viz_pn)) = (&mrz_data.personal_number, &visual_data.personal_number) {
            if !Self::fields_match(mrz_pn, viz_pn) {
                corrected.personal_number = mrz_data.personal_number.clone();
                corrections.insert("personal_number", 
                    format!("Visual: {} -> MRZ: {}", viz_pn, mrz_pn));
            }
        } else if visual_data.personal_number.is_none() && mrz_data.personal_number.is_some() {
            corrected.personal_number = mrz_data.personal_number.clone();
            corrections.insert("personal_number", 
                format!("Empty -> MRZ: {:?}", mrz_data.personal_number));
        }
        
        // Place of birth (if available in MRZ but not in visual)
        if visual_data.place_of_birth.is_none() && mrz_data.place_of_birth.is_some() {
            corrected.place_of_birth = mrz_data.place_of_birth.clone();
            corrections.insert("place_of_birth", 
                format!("Empty -> MRZ: {:?}", mrz_data.place_of_birth));
        }
        
        // Log corrections for debugging/analytics
        if !corrections.is_empty() {
            println!("📝 Field corrections applied:");
            for (field, correction) in &corrections {
                println!("  • {}: {}", field, correction);
            }
        }
        
        corrected
    }
    
    /// Helper method to check if two fields match exactly
    fn fields_match(field1: &str, field2: &str) -> bool {
        // Clean both fields (remove spaces, special characters)
        let clean1 = field1.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
            
        let clean2 = field2.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        
        clean1 == clean2
    }
    
    /// Helper method to check if two fields match ignoring case
    fn fields_match_case_insensitive(field1: &str, field2: &str) -> bool {
        // Clean both fields and convert to uppercase
        let clean1 = field1.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase();
            
        let clean2 = field2.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase();
        
        clean1 == clean2
    }
    
    /// Helper method to check if names are compatible
    /// This is less strict than exact matching as names in MRZ are often truncated
    fn names_compatible(mrz_name: &str, visual_name: &str) -> bool {
        if mrz_name.is_empty() || visual_name.is_empty() {
            return true; // Can't check empty names
        }
        
        // Clean names
        let clean_mrz = mrz_name.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase();
            
        let clean_viz = visual_name.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase();
        
        // If one is contained in the other or they're exactly equal
        clean_mrz.contains(&clean_viz) || clean_viz.contains(&clean_mrz) || clean_mrz == clean_viz
    }
    
    /// Helper method to check if dates are compatible
    /// This handles different date formats like YYMMDD vs DD/MM/YYYY
    fn dates_compatible(mrz_date: &str, visual_date: &str) -> bool {
        if mrz_date.is_empty() || visual_date.is_empty() {
            return true; // Can't check empty dates
        }
        
        // Parse MRZ date (standard format YYMMDD)
        if mrz_date.len() == 6 && mrz_date.chars().all(|c| c.is_ascii_digit()) {
            let cleaned_visual = visual_date.chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>();
                
            // If visual date has only digits (likely YYMMDD or DDMMYY)
            if cleaned_visual.chars().all(|c| c.is_ascii_digit()) {
                if cleaned_visual.len() == 6 {
                    // Could be YYMMDD (like MRZ) or DDMMYY
                    // Try both and see if either matches
                    return mrz_date == cleaned_visual || 
                           mrz_date[4..6] == cleaned_visual[0..2] && 
                           mrz_date[2..4] == cleaned_visual[2..4] && 
                           mrz_date[0..2] == cleaned_visual[4..6];
                } else if cleaned_visual.len() == 8 {
                    // Likely DDMMYYYY or YYYYMMDD
                    // Extract YY from YYYY for comparison
                    let yy = if cleaned_visual.starts_with("19") || cleaned_visual.starts_with("20") {
                        &cleaned_visual[2..4] // YYYYMMDD format
                    } else {
                        &cleaned_visual[6..8] // DDMMYYYY format
                    };
                    
                    let mm_dd_matches = 
                        (mrz_date[2..4] == cleaned_visual[2..4] && mrz_date[4..6] == cleaned_visual[0..2]) || // DDMMYYYY
                        (mrz_date[2..4] == cleaned_visual[4..6] && mrz_date[4..6] == cleaned_visual[6..8]);  // YYYYMMDD
                        
                    return &mrz_date[0..2] == yy && mm_dd_matches;
                }
            }
            
            // If visual date has separators, try to extract components
            let parts: Vec<&str> = visual_date.split(|c| c == '/' || c == '-' || c == '.').collect();
            
            if parts.len() == 3 {
                let day_part = parts[0].parse::<u32>().ok();
                let month_part = parts[1].parse::<u32>().ok();
                let year_part = parts[2].parse::<i32>().ok();
                
                if let (Some(d), Some(m), Some(y)) = (day_part, month_part, year_part) {
                    let mrz_day = mrz_date[4..6].parse::<u32>().unwrap_or(0);
                    let mrz_month = mrz_date[2..4].parse::<u32>().unwrap_or(0);
                    let mrz_year = if mrz_date[0..2].parse::<u32>().unwrap_or(0) >= 50 {
                        1900 + mrz_date[0..2].parse::<i32>().unwrap_or(0)
                    } else {
                        2000 + mrz_date[0..2].parse::<i32>().unwrap_or(0)
                    };
                    
                    // Compare components
                    let year_matches = y % 100 == mrz_year % 100 || y == mrz_year;
                    return d == mrz_day && m == mrz_month && year_matches;
                }
            }
        }
        
        // Default fallback comparison
        let clean_mrz = mrz_date.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();
            
        let clean_viz = visual_date.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();
            
        clean_mrz == clean_viz || clean_mrz.contains(&clean_viz) || clean_viz.contains(&clean_mrz)
    }
    
    /// Format MRZ date (YYMMDD) into a more human-readable format (DD/MM/YYYY)
    fn format_mrz_date(mrz_date: &str) -> String {
        if mrz_date.len() == 6 && mrz_date.chars().all(|c| c.is_ascii_digit()) {
            let day = &mrz_date[4..6];
            let month = &mrz_date[2..4];
            let year_prefix = if mrz_date[0..2].parse::<u32>().unwrap_or(0) >= 50 {
                "19"
            } else {
                "20"
            };
            let year = format!("{}{}", year_prefix, &mrz_date[0..2]);
            
            format!("{}/{}/{}", day, month, year)
        } else {
            mrz_date.to_string()
        }
    }
}
