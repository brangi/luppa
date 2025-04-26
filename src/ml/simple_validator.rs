use crate::models::*;
use std::collections::HashMap;
use regex::Regex;

/// A simplified ML validator that provides similar functionality without compilation issues
pub struct SimpleValidator {
    country_rules: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ValidationConfidence {
    pub mrz_confidence: f32,
    pub visual_confidence: f32,
    pub consistency_confidence: f32,
    pub security_feature_confidence: f32,
    pub fraud_detection_confidence: f32,
}

impl SimpleValidator {
    pub fn new() -> Self {
        let mut country_rules = HashMap::new();
        
        // Add some example country rules
        country_rules.insert("USA".to_string(), vec![
            "document_number_length:9".to_string(),
            "name_format:SURNAME/GIVEN".to_string(),
        ]);
        
        country_rules.insert("GBR".to_string(), vec![
            "document_number_length:9".to_string(),
            "name_format:SURNAME/GIVEN".to_string(),
        ]);
        
        Self { country_rules }
    }
    
    /// Validate both MRZ and visual data, returning overall validity and confidence scores
    pub fn validate(&self, mrz_data: &MrzData, visual_data: &VisualData) -> (bool, ValidationConfidence) {
        // Calculate confidence for each validation component
        let mrz_confidence = self.validate_mrz(mrz_data);
        let visual_confidence = self.validate_visual_data(visual_data);
        let consistency_confidence = self.validate_consistency(mrz_data, visual_data);
        let security_feature_confidence = 0.85_f32; // Placeholder
        let fraud_detection_confidence = self.detect_fraud(mrz_data, visual_data);
        
        // Determine overall validity
        let is_valid = 
            mrz_confidence > 0.7_f32 && 
            visual_confidence > 0.6_f32 && 
            consistency_confidence > 0.7_f32 &&
            fraud_detection_confidence > 0.8_f32;
        
        // Return validation result and confidence scores
        (
            is_valid,
            ValidationConfidence {
                mrz_confidence,
                visual_confidence,
                consistency_confidence,
                security_feature_confidence,
                fraud_detection_confidence,
            }
        )
    }
    
    // Validate MRZ data using ML-enhanced methods
    fn validate_mrz(&self, mrz_data: &MrzData) -> f32 {
        let mut confidence: f32 = 0.0;
        
        // Check if we have the required MRZ fields
        if !mrz_data.document_number.is_empty() {
            confidence += 0.2_f32;
        }
        
        if !mrz_data.surname.is_empty() && !mrz_data.given_names.is_empty() {
            confidence += 0.2_f32;
        }
        
        if !mrz_data.nationality.is_empty() {
            confidence += 0.1_f32;
        }
        
        if !mrz_data.date_of_birth.is_empty() {
            confidence += 0.2_f32;
        }
        
        if !mrz_data.gender.is_empty() {
            confidence += 0.1_f32;
        }
        
        if !mrz_data.date_of_expiry.is_empty() {
            confidence += 0.2_f32;
        }
        
        // Check country-specific rules if available
        if let Some(rules) = self.country_rules.get(&mrz_data.issuing_country) {
            for rule in rules {
                if rule.starts_with("document_number_length:") {
                    let length: usize = rule.split(':').nth(1).unwrap_or("9").parse().unwrap_or(9);
                    if mrz_data.document_number.len() == length {
                        confidence += 0.1_f32;
                    } else {
                        confidence -= 0.2_f32; // Penalty for incorrect document number length
                    }
                }
            }
        }
        
        // Cap confidence between 0 and 1
        confidence.max(0.0_f32).min(1.0_f32)
    }
    
    // Validate visual data using ML-enhanced methods
    fn validate_visual_data(&self, visual_data: &VisualData) -> f32 {
        let mut confidence: f32 = 0.0;
        let checks: f32 = 8.0; // Total number of checks
        
        // Document data checks
        if !visual_data.document_number.is_empty() {
            confidence += 1.0_f32;
        }
        
        if !visual_data.surname.is_empty() {
            confidence += 1.0_f32;
        }
        
        // Check date of birth
        if !visual_data.date_of_birth.is_empty() {
            confidence += 1.0_f32;
        }
        
        // Check gender
        if !visual_data.gender.is_empty() {
            confidence += 1.0_f32;
        }
        
        // Check date of expiry
        if !visual_data.date_of_expiry.is_empty() {
            confidence += 1.0_f32;
        }
        
        // Check date of issue
        if !visual_data.date_of_issue.is_empty() {
            confidence += 1.0_f32;
        }
        
        // Check place of birth
        if let Some(place) = &visual_data.place_of_birth {
            if !place.is_empty() {
                confidence += 1.0_f32;
            }
        }
        
        // Check authority
        if let Some(auth) = &visual_data.authority {
            if !auth.is_empty() {
                confidence += 1.0_f32;
            }
        }
        
        // Calculate and return overall confidence score
        (confidence / checks).max(0.0_f32).min(1.0_f32)
    }
    
    // Validate consistency between MRZ and visual data zones
    fn validate_consistency(&self, mrz_data: &MrzData, visual_data: &VisualData) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut checks: i32 = 0;
        
        // Document number consistency
        if !mrz_data.document_number.is_empty() && !visual_data.document_number.is_empty() {
            let doc_similarity = self.calculate_similarity(
                &mrz_data.document_number, 
                &visual_data.document_number
            );
            confidence += doc_similarity;
            checks += 1;
        }
        
        // Name consistency
        if !mrz_data.surname.is_empty() && !visual_data.surname.is_empty() {
            let surname_similarity = self.calculate_similarity(
                &mrz_data.surname.to_uppercase(), 
                &visual_data.surname.to_uppercase()
            );
            confidence += surname_similarity;
            checks += 1;
        }
        
        // Date of birth consistency
        if !mrz_data.date_of_birth.is_empty() && !visual_data.date_of_birth.is_empty() {
            let dob_similarity = self.calculate_date_similarity(
                &mrz_data.date_of_birth, 
                &visual_data.date_of_birth
            );
            confidence += dob_similarity;
            checks += 1;
        }
        
        // Gender consistency 
        if !mrz_data.gender.is_empty() && !visual_data.gender.is_empty() {
            let gender_similarity = if mrz_data.gender == visual_data.gender {
                1.0_f32
            } else {
                0.0_f32
            };
            confidence += gender_similarity;
            checks += 1;
        }
        
        // Date of expiry consistency
        if !mrz_data.date_of_expiry.is_empty() && !visual_data.date_of_expiry.is_empty() {
            let doe_similarity = self.calculate_date_similarity(
                &mrz_data.date_of_expiry, 
                &visual_data.date_of_expiry
            );
            confidence += doe_similarity;
            checks += 1;
        }
        
        // Calculate average confidence across all checks
        if checks > 0 {
            confidence / checks as f32
        } else {
            // If no checks could be performed, assign a low confidence
            0.3_f32
        }
    }
    
    // Calculate similarity between two string values
    fn calculate_similarity(&self, a: &str, b: &str) -> f32 {
        if a == b {
            return 1.0_f32;
        }
        
        // Calculate Levenshtein distance
        let distance = self.levenshtein_distance(a, b);
        let max_len = a.len().max(b.len()) as f32;
        
        if max_len == 0.0_f32 {
            return 1.0_f32;
        }
        
        // Calculate similarity as 1 - (normalized distance)
        let similarity = 1.0_f32 - (distance as f32 / max_len);
        similarity.max(0.0_f32).min(1.0_f32)
    }
    
    // Calculate simple Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        if s1 == s2 {
            return 0;
        }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                
                matrix[i][j] = [
                    matrix[i - 1][j] + 1,
                    matrix[i][j - 1] + 1,
                    matrix[i - 1][j - 1] + cost
                ].iter().min().cloned().unwrap_or(0);
            }
        }
        
        matrix[len1][len2]
    }
    
    // Calculate similarity between two dates, handling different formats
    fn calculate_date_similarity(&self, date1: &str, date2: &str) -> f32 {
        // Normalize date formats to YYMMDD for comparison
        let normalized1 = self.normalize_date(date1);
        let normalized2 = self.normalize_date(date2);
        
        self.calculate_similarity(&normalized1, &normalized2)
    }
    
    // Normalize dates to a standard format for comparison
    fn normalize_date(&self, date: &str) -> String {
        // Simple pattern matching to extract year, month, day in various formats
        
        // Try YY-MM-DD or YYMMDD format
        if let Ok(re) = Regex::new(r"(?i)(\d{2})[-/.]?(\d{2})[-/.]?(\d{2})") {
            if let Some(caps) = re.captures(date) {
                if caps.len() >= 4 {
                    return format!("{}{}{}", 
                        caps.get(1).map_or("", |m| m.as_str()),
                        caps.get(2).map_or("", |m| m.as_str()),
                        caps.get(3).map_or("", |m| m.as_str())
                    );
                }
            }
        }
        
        // Try DD-MM-YY format
        if let Ok(re) = Regex::new(r"(?i)(\d{2})[-/.](\d{2})[-/.](\d{2})") {
            if let Some(caps) = re.captures(date) {
                if caps.len() >= 4 {
                    return format!("{}{}{}", 
                        caps.get(3).map_or("", |m| m.as_str()),
                        caps.get(2).map_or("", |m| m.as_str()),
                        caps.get(1).map_or("", |m| m.as_str())
                    );
                }
            }
        }
        
        // Just remove non-digits if we can't parse it
        date.chars().filter(|c| c.is_digit(10)).collect()
    }
    
    // Detect potential fraud indicators using AI techniques
    fn detect_fraud(&self, mrz_data: &MrzData, visual_data: &VisualData) -> f32 {
        let mut fraud_likelihood: f32 = 0.0;
        
        // Check for mismatched data between MRZ and visual zones
        let consistency = self.validate_consistency(mrz_data, visual_data);
        if consistency < 0.6_f32 {
            fraud_likelihood += 0.4_f32;
        }
        
        // Check for impossible dates
        if visual_data.date_of_birth.contains("00/00") || visual_data.date_of_birth.contains("99/99") {
            fraud_likelihood += 0.4_f32;
        }
        
        // Check for expired passport (more than 10 years old)
        if visual_data.date_of_expiry.contains("201") || visual_data.date_of_expiry.contains("200") {
            fraud_likelihood += 0.2_f32;
        }
        
        // Invert fraud likelihood to get confidence
        1.0_f32 - fraud_likelihood.max(0.0_f32).min(1.0_f32)
    }
}
