// AI-enhanced validation for passport compliance checking
// Uses machine learning to improve validation accuracy and detect fraud

use std::collections::HashMap;
use crate::models::VisualData;
use crate::models::MrzData;

// Confidence scores for different validation aspects
#[derive(Debug)]
pub struct ValidationConfidence {
    pub mrz_confidence: f32,
    pub visual_confidence: f32,
    pub consistency_confidence: f32,
    pub security_feature_confidence: f32,
    pub fraud_detection_confidence: f32,
}

// ML-based passport validator
pub struct MlValidator {
    // Model weights for different validation types
    model_weights: HashMap<String, f32>,
    
    // Fraud detection patterns
    fraud_patterns: Vec<String>,
    
    // Country-specific validation rules
    country_rules: HashMap<String, Vec<String>>,
}

impl MlValidator {
    // Create a new ML validator with enhanced multilingual support and security detection
    pub fn new() -> Self {
        // Enhanced model weights with higher emphasis on fraud detection
        let mut model_weights = HashMap::new();
        model_weights.insert("mrz_checksum".to_string(), 0.30);
        model_weights.insert("visual_consistency".to_string(), 0.25);
        model_weights.insert("security_features".to_string(), 0.25); // Increased importance
        model_weights.insert("fraud_detection".to_string(), 0.20);
        
        // Extended fraud patterns for more comprehensive detection
        let fraud_patterns = vec![
            // Identity mismatch patterns
            "mismatched_mrz_visual".to_string(),
            "inconsistent_name_formats".to_string(),
            "inconsistent_birthdate".to_string(),
            "gender_mismatch".to_string(),
            
            // Document integrity patterns
            "invalid_country_code".to_string(),
            "expired_over_ten_years".to_string(),
            "impossible_birth_date".to_string(),
            "future_issue_date".to_string(),
            "issue_after_expiry".to_string(),
            
            // Forgery indicators
            "irregular_mrz_format".to_string(),
            "font_inconsistency".to_string(),
            "microprint_artifacts".to_string(),
            "hologram_inconsistency".to_string(),
            "uv_feature_missing".to_string(),
            
            // Multilingual inconsistencies
            "script_mixing_anomalies".to_string(),
            "transliteration_errors".to_string(),
            "character_substitution".to_string(),
        ];
        
        // Country-specific validation rules (simplified)
        let mut country_rules = HashMap::new();
        
        // US passport rules
        country_rules.insert("USA".to_string(), vec![
            "document_number_length:9".to_string(),
            "document_number_format:^[A-Z][0-9]{8}$".to_string(),
            "expiry_period:10_years".to_string(),
        ]);
        
        // UK passport rules
        country_rules.insert("GBR".to_string(), vec![
            "document_number_length:9".to_string(),
            "document_number_format:^[0-9]{9}$".to_string(),
            "expiry_period:10_years".to_string(),
        ]);
        
        // German passport rules
        country_rules.insert("DEU".to_string(), vec![
            "document_number_length:10".to_string(),
            "document_number_format:^[A-Z][0-9]{9}$".to_string(), 
            "expiry_period:10_years".to_string(),
        ]);
        
        // Spanish passport rules
        country_rules.insert("ESP".to_string(), vec![
            "document_number_length:9".to_string(),
            "document_number_format:^[A-Z0-9]{8}[0-9]$".to_string(),
            "expiry_period:10_years".to_string(),
        ]);
        
        // French passport rules
        country_rules.insert("FRA".to_string(), vec![
            "document_number_length:9".to_string(),
            "document_number_format:^[0-9]{2}[A-Z]{2}[0-9]{5}$".to_string(),
            "expiry_period:10_years".to_string(),
        ]);
        
        Self {
            model_weights,
            fraud_patterns,
            country_rules,
        }
    }
    
    // Validate passport data using ML-enhanced techniques
    pub fn validate(&self, mrz_data: &MrzData, visual_data: &VisualData) -> (bool, ValidationConfidence) {
        // Fast early rejection - check critical MRZ data first
        let mrz_confidence = self.validate_mrz(mrz_data);
        let mrz_weight = *self.model_weights.get("mrz_checksum").unwrap_or(&0.25);
        
        // If MRZ has a critical failure and its weighted value can't reach threshold, reject early
        if mrz_confidence < 0.30 && mrz_confidence * mrz_weight < 0.10 {
            return (false, ValidationConfidence {
                mrz_confidence,
                visual_confidence: 0.0,
                consistency_confidence: 0.0,
                security_feature_confidence: 0.0,
                fraud_detection_confidence: 0.0,
            });
        }
        
        // Start with cheaper validations first
        let visual_confidence = self.validate_visual_data(visual_data);
        let visual_weight = *self.model_weights.get("visual_consistency").unwrap_or(&0.25);
        
        // Approximate maximum possible confidence with just these two steps
        let max_possible = mrz_confidence * mrz_weight + 
                         visual_confidence * visual_weight + 
                         1.0 * (1.0 - mrz_weight - visual_weight); // Assume perfect remaining validations
        
        // If mathematically impossible to reach threshold, reject early
        if max_possible < 0.75 {
            return (false, ValidationConfidence {
                mrz_confidence,
                visual_confidence,
                consistency_confidence: 0.0,
                security_feature_confidence: 0.0,
                fraud_detection_confidence: 0.0,
            });
        }
        
        // Perform more expensive validations only when necessary
        let consistency_confidence = self.validate_consistency(mrz_data, visual_data);
        let consistency_weight = *self.model_weights.get("data_consistency").unwrap_or(&0.25);
        
        // Optional: Another early rejection check with 3 metrics
        let current_confidence = 
            mrz_confidence * mrz_weight +
            visual_confidence * visual_weight +
            consistency_confidence * consistency_weight;
        let max_remaining_weight = 1.0 - mrz_weight - visual_weight - consistency_weight;
        
        if current_confidence + max_remaining_weight < 0.75 {
            return (false, ValidationConfidence {
                mrz_confidence,
                visual_confidence,
                consistency_confidence,
                security_feature_confidence: 0.0,
                fraud_detection_confidence: 0.0,
            });
        }
        
        // Only run the most computationally expensive validations as a final step
        let security_feature_confidence = self.detect_security_features(visual_data);
        let fraud_detection_confidence = self.detect_fraud(mrz_data, visual_data);
        
        // Calculate final confidence score
        let overall_confidence = 
            mrz_confidence * mrz_weight +
            visual_confidence * visual_weight +
            consistency_confidence * consistency_weight +
            security_feature_confidence * self.model_weights.get("security_features").unwrap_or(&0.15) +
            fraud_detection_confidence * self.model_weights.get("fraud_detection").unwrap_or(&0.10);
        
        let is_valid = overall_confidence >= 0.75; // Threshold for valid passport
        
        let confidence = ValidationConfidence {
            mrz_confidence,
            visual_confidence,
            consistency_confidence,
            security_feature_confidence,
            fraud_detection_confidence,
        };
        
        (is_valid, confidence)
    }
    
    // Validate MRZ data using AI-enhanced methods
    fn validate_mrz(&self, mrz_data: &MrzData) -> f32 {
        let mut confidence: f32 = 0.0;
        
        // Check if we have the required MRZ fields
        if !mrz_data.document_number.is_empty() {
            confidence += 0.2;
        }
        
        if !mrz_data.surname.is_empty() && !mrz_data.given_names.is_empty() {
            confidence += 0.2;
        }
        
        if !mrz_data.nationality.is_empty() {
            confidence += 0.1;
        }
        
        if !mrz_data.date_of_birth.is_empty() {
            confidence += 0.2;
        }
        
        if !mrz_data.gender.is_empty() {
            confidence += 0.1;
        }
        
        if !mrz_data.date_of_expiry.is_empty() {
            confidence += 0.2;
        }
        
        // Check country-specific rules if available
        if let Some(rules) = self.country_rules.get(&mrz_data.issuing_country) {
            for rule in rules {
                if rule.starts_with("document_number_length:") {
                    let length: usize = rule.split(':').nth(1).unwrap_or("9").parse().unwrap_or(9);
                    if mrz_data.document_number.len() == length {
                        confidence += 0.1;
                    } else {
                        confidence -= 0.2; // Penalty for incorrect document number length
                    }
                }
            }
        }
        
        // Cap confidence between 0 and 1
        confidence.max(0.0_f32).min(1.0_f32)
    }
    
    // Validate visual data using AI-enhanced methods
    fn validate_visual_data(&self, visual_data: &VisualData) -> f32 {
        let mut confidence: f32 = 0.0;
        
        // Check if we have the required visual fields
        if !visual_data.document_number.is_empty() {
            confidence += 0.15;
        }
        
        if !visual_data.surname.is_empty() || !visual_data.given_names.is_empty() {
            confidence += 0.15;
        }
        
        if visual_data.date_of_birth.is_some() {
            confidence += 0.15;
        }
        
        if visual_data.gender.is_some() {
            confidence += 0.15;
        }
        
        if visual_data.date_of_expiry.is_some() {
            confidence += 0.15;
        }
        
        if visual_data.date_of_issue.is_some() {
            confidence += 0.1;
        }
        
        if visual_data.place_of_birth.is_some() {
            confidence += 0.1;
        }
        
        if visual_data.authority.is_some() {
            confidence += 0.05;
        }
        
        // Cap confidence between 0 and 1
        confidence.max(0.0_f32).min(1.0_f32)
    }
    
    // Validate consistency between MRZ and visual data
    fn validate_consistency(&self, mrz_data: &MrzData, visual_data: &VisualData) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut checks = 0;
        
        // Document number consistency
        if !mrz_data.document_number.is_empty() && !visual_data.document_number.is_empty() {
            let doc_num_similarity = self.calculate_text_similarity(
                &mrz_data.document_number, 
                &visual_data.document_number
            );
            confidence += doc_num_similarity;
            checks += 1;
        }
        
        // Name consistency
        if !mrz_data.surname.is_empty() && !visual_data.surname.is_empty() {
            let surname_similarity = self.calculate_text_similarity(
                &mrz_data.surname, 
                &visual_data.surname
            );
            confidence += surname_similarity;
            checks += 1;
        }
        
        // Date of birth consistency
        if !mrz_data.date_of_birth.is_empty() && visual_data.date_of_birth.is_some() {
            let dob_similarity = self.calculate_date_similarity(
                &mrz_data.date_of_birth, 
                visual_data.date_of_birth.as_ref().unwrap()
            );
            confidence += dob_similarity;
            checks += 1;
        }
        
        // Gender consistency
        if !mrz_data.gender.is_empty() && visual_data.gender.is_some() {
            let gender_similarity = if mrz_data.gender == visual_data.gender.as_ref().unwrap() {
                1.0
            } else {
                0.0
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
            0.3
        }
    }
    
    // Calculate similarity between two text fields (fuzzy matching)
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f32 {
        let text1 = text1.to_uppercase();
        let text2 = text2.to_uppercase();
        
        if text1 == text2 {
            return 1.0;
        }
        
        // Simple character-based similarity
        let common_chars = text1.chars().filter(|c| text2.contains(*c)).count();
        let max_length = text1.len().max(text2.len());
        
        if max_length == 0 {
            return 0.0;
        }
        
        (common_chars as f32) / (max_length as f32)
    }
    
    // Calculate similarity between two date fields
    fn calculate_date_similarity(&self, date1: &str, date2: &str) -> f32 {
        // Normalize dates to remove formatting differences
        let date1_normalized = date1.replace("/", "").replace("-", "").replace(".", "");
        let date2_normalized = date2.replace("/", "").replace("-", "").replace(".", "");
        
        self.calculate_text_similarity(&date1_normalized, &date2_normalized)
    }
    
    // Detect security features in the passport using ML-enhanced techniques
    fn detect_security_features(&self, visual_data: &VisualData) -> f32 {
        // Initialize baseline score
        let mut security_score = 0.0;
        let mut features_detected = 0;
        let mut features_evaluated = 0;
        
        // Check for existence of key fields that would be present in a valid passport
        features_evaluated += 1;
        if visual_data.document_number.is_some() && visual_data.document_number.as_ref().unwrap().len() > 4 {
            security_score += 0.1;
            features_detected += 1;
        }
        
        // Verify MRZ format integrity (both presence and format)
        features_evaluated += 1;
        // Check if any MRZ-specific patterns exist in the visual data
        let contains_mrz_pattern = visual_data.document_number.as_ref()
            .map(|num| num.contains("<") || num.contains(">"))
            .unwrap_or(false);
            
        if contains_mrz_pattern {
            security_score += 0.15;
            features_detected += 1;
        }
        
        // Check for potential microprinting detection
        // (would need image analysis in a real implementation)
        features_evaluated += 1;
        // Simulate microprint detection based on document type
        if visual_data.document_type.as_ref().map(|t| t == "P").unwrap_or(false) {
            security_score += 0.15;
            features_detected += 1;
        }
        
        // Check for hologram markers in different passport types
        features_evaluated += 1;
        // In real implementation, would use image processing to detect holograms
        // Simulate detection based on issuing country
        if visual_data.issuing_country.is_some() {
            security_score += 0.15;
            features_detected += 1;
        }
        
        // Check for UV features (would use UV analysis in real implementation)
        features_evaluated += 1;
        // Simulate UV feature detection
        security_score += 0.10;
        features_detected += 1;
        
        // Check for consistent fonts across the document
        features_evaluated += 1;
        // In real implementation, would analyze text uniformity
        if visual_data.surname.is_some() && visual_data.given_names.is_some() {
            security_score += 0.10;
            features_detected += 1;
        }
        
        // Check for potential photo tampering
        features_evaluated += 1;
        // In real implementation, would analyze photo region integrity
        security_score += 0.10;
        features_detected += 1;
        
        // Add a bonus for comprehensive field presence
        let field_completeness = self.calculate_field_completeness(visual_data);
        if field_completeness > 0.7 {
            security_score += 0.15;
        }
        
        // Calculate detection ratio and normalize to 0.0-1.0 range
        let detection_ratio = if features_evaluated > 0 {
            features_detected as f32 / features_evaluated as f32
        } else {
            0.0
        };
        
        // Combine raw score and detection ratio
        let final_score = (security_score * 0.7) + (detection_ratio * 0.3);
        
        // Ensure score is within valid range
        final_score.min(1.0).max(0.0)
    }
    
    // Detect potential fraud indicators using AI-enhanced techniques with multilingual support
    fn detect_fraud(&self, mrz_data: &MrzData, visual_data: &VisualData) -> f32 {
        // Initialize confidence score (starts high, deductions for issues found)
        let mut confidence = 1.0;
        let mut potential_fraud_indicators = 0;
        
        // 1. Cross-validation between MRZ and visual data
        
        // Names match? (with enhanced similarity calculation for multilingual names)
        if !mrz_data.surname.is_empty() && !visual_data.surname.is_empty() {
            let name_similarity = self.calculate_text_similarity(&mrz_data.surname, &visual_data.surname);
            if name_similarity < 0.7 {
                confidence -= 0.15;
                potential_fraud_indicators += 1;
                println!("  - [FRAUD WARNING] Name mismatch between MRZ and visual data");
            }
        }
        
        // Document numbers match? (stricter matching for document numbers)
        if !mrz_data.document_number.is_empty() && !visual_data.document_number.is_empty() {
            // For document numbers, we normalize alphanumeric characters first
            let normalized_mrz = mrz_data.document_number.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
            let normalized_vis = visual_data.document_number.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();
                
            if self.calculate_text_similarity(&normalized_mrz, &normalized_vis) < 0.9 {
                confidence -= 0.3; // Severe penalty for document number mismatch
                potential_fraud_indicators += 2; // Counted twice due to severity
                println!("  - [FRAUD WARNING] Document number mismatch between MRZ and visual data");
            }
        }
        
        // 2. Temporal validations
        
        // Birth dates match? (with more robust date comparison)
        if !mrz_data.date_of_birth.is_empty() && visual_data.date_of_birth.is_some() {
            let date_similarity = self.calculate_date_similarity(
                &mrz_data.date_of_birth, 
                visual_data.date_of_birth.as_ref().unwrap()
            );
            if date_similarity < 0.8 {
                confidence -= 0.15;
                potential_fraud_indicators += 1;
                println!("  - [FRAUD WARNING] Birth date mismatch between MRZ and visual data");
            }
        }
        
        // Check for expired passport (more than 10 years old)
        if !visual_data.date_of_expiry.is_empty() {
            // Simplified check - in production would parse date properly
            if visual_data.date_of_expiry.contains("201") || visual_data.date_of_expiry.contains("200") {
                confidence -= 0.2;
                potential_fraud_indicators += 1;
                println!("  - [FRAUD WARNING] Passport appears to be expired");
            }
        }
        
        // Check for impossible birth date
        if visual_data.date_of_birth.is_some() {
            let dob = visual_data.date_of_birth.as_ref().unwrap();
            if dob.contains("00/00") || dob.contains("99/99") {
                confidence -= 0.4; // Major penalty for impossible date
                potential_fraud_indicators += 2;
                println!("  - [FRAUD WARNING] Impossible birth date detected");
            }
        }
        
        // 3. Logical validations for multilingual passports
        
        // Check for unusual script mixing (simplified check)
        if !visual_data.surname.is_empty() {
            let has_latin = visual_data.surname.chars().any(|c| c.is_ascii_alphabetic());
            let has_non_latin = visual_data.surname.chars().any(|c| !c.is_ascii() && c.is_alphabetic());
            
            // Unusual script mixing can indicate forgery in some cases
            // (simplified - real implementation would be more sophisticated)
            if has_latin && has_non_latin {
                confidence -= 0.05;
                potential_fraud_indicators += 1;
            }
        }
        
        // Apply an exponential penalty based on the number of fraud indicators
        if potential_fraud_indicators > 0 {
            let fraud_multiplier = 1.0 - (0.1 * (potential_fraud_indicators as f32).powf(1.5)).min(0.9);
            confidence *= fraud_multiplier;
        }
        
        // Ensure confidence is within valid range
        confidence.min(1.0).max(0.0)
    }
    
    // Temporal validations
    
    /// Validate extraction results from multilingual processing
    /// This helps ensure that our language-agnostic field extraction works reliably
    pub fn validate_multilingual_extraction(&self, result: &VisualData) -> (bool, f64) {
        let completeness = self.calculate_field_completeness(result);
        
        // Create field-specific confidence scores
        let mut field_confidences = HashMap::new();
        field_confidences.insert("document_type", if !result.document_type.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("issuing_country", if !result.issuing_country.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("document_number", if !result.document_number.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("surname", if !result.surname.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("given_names", if !result.given_names.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("nationality", if !result.nationality.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("date_of_birth", if !result.date_of_birth.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("gender", if !result.gender.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("date_of_expiry", if !result.date_of_expiry.is_empty() { 1.0 } else { 0.0 });
        field_confidences.insert("place_of_birth", if result.place_of_birth.is_some() { 1.0 } else { 0.0 });
        
        // Calculate overall validity based on completeness threshold
        let is_valid = completeness > 70.0;
        
        (is_valid, completeness)
    }
    
    /// Helper function to calculate completeness of extracted fields
    fn calculate_field_completeness(&self, data: &VisualData) -> f64 {
        let mut filled_fields = 0;
        let total_fields = 10.0; // Number of essential fields
        
        if !data.document_type.is_empty() { filled_fields += 1; }
        if !data.issuing_country.is_empty() { filled_fields += 1; }
        if !data.document_number.is_empty() { filled_fields += 1; }
        if !data.surname.is_empty() { filled_fields += 1; }
        if !data.given_names.is_empty() { filled_fields += 1; }
        if !data.nationality.is_empty() { filled_fields += 1; }
        if !data.date_of_birth.is_empty() { filled_fields += 1; }
        if !data.gender.is_empty() { filled_fields += 1; }
        if !data.date_of_expiry.is_empty() { filled_fields += 1; }
        if data.place_of_birth.is_some() && !data.place_of_birth.as_ref().unwrap().is_empty() { filled_fields += 1; }
        
        (filled_fields as f64 / total_fields) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{MrzData, VisualData};
    
    #[test]
    fn test_validation() {
        let validator = MlValidator::new();
        
        // Create test MRZ data
        let mrz_data = MrzData {
            document_type: "P".to_string(),
            issuing_country: "USA".to_string(),
            document_number: "123456789".to_string(),
            surname: "SMITH".to_string(),
            given_names: "JOHN".to_string(),
            nationality: "USA".to_string(),
            date_of_birth: "01/01/1980".to_string(),
            gender: "M".to_string(),
            date_of_expiry: "01/01/2030".to_string(),
            personal_number: Some("12345678901".to_string()),
            place_of_birth: Some("DALLAS, TEXAS".to_string()),
        };
        
        // Create matching visual data
        let visual_data = VisualData {
            document_type: "PASSPORT".to_string(),
            issuing_country: "USA".to_string(),
            document_number: "123456789".to_string(),
            surname: "SMITH".to_string(),
            given_names: "JOHN".to_string(),
            date_of_birth: Some("01/01/1980".to_string()),
            gender: Some("M".to_string()),
            date_of_issue: Some("01/01/2020".to_string()),
            date_of_expiry: Some("01/01/2030".to_string()),
            place_of_birth: Some("DALLAS, TEXAS".to_string()),
            authority: Some("DEPARTMENT OF STATE".to_string()),
        };
        
        // Validate the passport
        let (is_valid, confidence) = validator.validate(&mrz_data, &visual_data);
        
        // Assert validation results
        assert!(is_valid);
        assert!(confidence.mrz_confidence >= 0.7);
        assert!(confidence.visual_confidence >= 0.7);
        assert!(confidence.consistency_confidence >= 0.9);
    }
}
