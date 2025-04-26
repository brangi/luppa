// Machine learning-enhanced feature extraction for passport processing
// Uses computer vision techniques to improve field detection and extraction

use std::collections::HashMap;
use std::path::Path;
use std::io;
use regex::Regex;

// Feature vector for ML-based field classification
#[derive(Debug, Clone)]
pub struct FieldFeature {
    pub field_type: String,      // Type of field (e.g., "document_number", "name", etc.)
    pub confidence: f32,         // Confidence score (0.0-1.0)
    pub bounding_box: (u32, u32, u32, u32), // x, y, width, height
    pub text_content: String,    // Extracted text content
    pub features: Vec<f32>,      // Feature vector for ML classification
}

// ML-based feature extractor
pub struct FeatureExtractor {
    // Pre-trained model weights (simplified for demonstration)
    field_classifiers: HashMap<String, Vec<f32>>,
    
    // Confidence thresholds for each field type
    confidence_thresholds: HashMap<String, f32>,
}

impl FeatureExtractor {
    // Create a new feature extractor with default parameters
    pub fn new() -> Self {
        let mut field_classifiers = HashMap::new();
        let mut confidence_thresholds = HashMap::new();
        
        // Initialize with pre-trained weights for field classification
        // In a real implementation, these would be loaded from a model file
        
        // Document number classifier
        field_classifiers.insert("document_number".to_string(), vec![0.8, 0.2, 0.7, 0.3, 0.9]);
        confidence_thresholds.insert("document_number".to_string(), 0.65);
        
        // Name classifier
        field_classifiers.insert("name".to_string(), vec![0.3, 0.9, 0.2, 0.8, 0.4]);
        confidence_thresholds.insert("name".to_string(), 0.70);
        
        // Date of birth classifier
        field_classifiers.insert("date_of_birth".to_string(), vec![0.7, 0.4, 0.6, 0.2, 0.5]);
        confidence_thresholds.insert("date_of_birth".to_string(), 0.68);
        
        // Gender classifier
        field_classifiers.insert("gender".to_string(), vec![0.2, 0.1, 0.9, 0.3, 0.5]);
        confidence_thresholds.insert("gender".to_string(), 0.60);
        
        // Expiry date classifier
        field_classifiers.insert("expiry_date".to_string(), vec![0.6, 0.4, 0.8, 0.3, 0.7]);
        confidence_thresholds.insert("expiry_date".to_string(), 0.68);
        
        Self {
            field_classifiers,
            confidence_thresholds,
        }
    }
    
    // Load a custom model from a file
    pub fn from_model_file<P: AsRef<Path>>(model_path: P) -> io::Result<Self> {
        // In a real implementation, this would deserialize a trained model
        // For now, we'll just create a default instance
        println!("Loading ML model from: {}", model_path.as_ref().display());
        Ok(Self::new())
    }
    
    // Extract features from an image for ML-based field detection
    pub fn extract_features(&self, image_data: &[u8], ocr_text: &str) -> Vec<FieldFeature> {
        let mut features = Vec::new();
        
        // Parse OCR text into lines and words
        let lines: Vec<&str> = ocr_text.lines().collect();
        
        // Pre-processing: clean up and normalize OCR text
        let cleaned_text = self.preprocess_text(ocr_text);
        
        // Find potential field regions using ML-inspired heuristics
        // Document Number
        if let Some(doc_num) = self.extract_document_number(&cleaned_text) {
            features.push(doc_num);
        }
        
        // Names
        if let Some(surname) = self.extract_surname(&cleaned_text) {
            features.push(surname);
        }
        
        if let Some(given_names) = self.extract_given_names(&cleaned_text) {
            features.push(given_names);
        }
        
        // Gender
        if let Some(gender) = self.extract_gender(&cleaned_text) {
            features.push(gender);
        }
        
        // Dates
        if let Some(dob) = self.extract_date_of_birth(&cleaned_text) {
            features.push(dob);
        }
        
        if let Some(doe) = self.extract_date_of_expiry(&cleaned_text) {
            features.push(doe);
        }
        
        // MRZ region
        if let Some(mrz) = self.detect_mrz_region(&cleaned_text) {
            features.push(mrz);
        }
        
        features
    }
    
    // Pre-process OCR text for better feature extraction
    fn preprocess_text(&self, text: &str) -> String {
        let text = text
            .replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace() && c != '-' && c != '/', " ")
            .replace("  ", " ")
            .trim()
            .to_string();
            
        // Normalize common OCR mistakes
        let text = text
            .replace("0", "O")
            .replace("1", "I")
            .replace("5", "S")
            .replace("8", "B");
            
        text
    }
    
    // ML-driven extraction of document number with confidence score
    fn extract_document_number(&self, text: &str) -> Option<FieldFeature> {
        // Use regex and positional information to find potential document numbers
        let doc_patterns = [
            r"(?i)document\s*no\.?\s*[:#]?\s*([A-Z0-9]{5,15})",
            r"(?i)passport\s*no\.?\s*[:#]?\s*([A-Z0-9]{5,15})",
            r"(?i)no\.?\s*[:#]?\s*([A-Z0-9]{5,15})",
            r"(?i)\b([A-Z][0-9]{7,9})\b",  // Common format: letter followed by digits
        ];
        
        let mut highest_confidence = 0.0;
        let mut best_match = None;
        
        for pattern in &doc_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(text) {
                    if let Some(m) = cap.get(1) {
                        // Calculate confidence based on features like:
                        // - Length (most document numbers are 7-10 characters)
                        // - Format (mix of letters and numbers)
                        // - Position in document (usually near top)
                        let doc_num = m.as_str().to_string();
                        let confidence = self.calculate_doc_number_confidence(&doc_num);
                        
                        if confidence > highest_confidence {
                            highest_confidence = confidence;
                            best_match = Some(FieldFeature {
                                field_type: "document_number".to_string(),
                                confidence,
                                bounding_box: (0, 0, 0, 0), // Placeholder, would be real coordinates in full implementation
                                text_content: doc_num,
                                features: vec![confidence, 0.0, 0.0, 0.0, 0.0], // Simplified feature vector
                            });
                        }
                    }
                }
            }
        }
        
        best_match
    }
    
    // Calculate confidence score for a potential document number
    fn calculate_doc_number_confidence(&self, doc_num: &str) -> f32 {
        let length = doc_num.len();
        let has_letters = doc_num.chars().any(|c| c.is_alphabetic());
        let has_digits = doc_num.chars().any(|c| c.is_numeric());
        let has_spaces = doc_num.contains(' ');
        
        // Length between 5-15 characters is typical for passport numbers
        let length_score = if length >= 5 && length <= 15 { 0.3 } else { 0.0 };
        
        // Most passport numbers have both letters and digits
        let format_score = if has_letters && has_digits { 0.4 } else { 0.1 };
        
        // Passport numbers typically don't contain spaces
        let space_score = if !has_spaces { 0.2 } else { 0.0 };
        
        // Combined confidence score
        let confidence = length_score + format_score + space_score;
        
        // Apply threshold from pre-trained model
        if let Some(&threshold) = self.confidence_thresholds.get("document_number") {
            if confidence < threshold {
                return 0.0;
            }
        }
        
        confidence
    }
    
    // ML-driven extraction of surname with confidence score
    fn extract_surname(&self, text: &str) -> Option<FieldFeature> {
        let surname_patterns = [
            r"(?i)surname[s]?[\s:]*([A-Za-z\s-]{2,30})",
            r"(?i)last\s*name[\s:]*([A-Za-z\s-]{2,30})",
            r"(?i)family\s*name[\s:]*([A-Za-z\s-]{2,30})",
            r"(?i)apellido[s]?[\s:]*([A-Za-z\s-]{2,30})",
            r"(?i)nom[\s:]*([A-Za-z\s-]{2,30})",
            r"(?i)nachname[\s:]*([A-Za-z\s-]{2,30})",
        ];
        
        self.extract_field_with_patterns(surname_patterns, "name", 0.7)
    }
    
    // ML-driven extraction of given names with confidence score
    fn extract_given_names(&self, text: &str) -> Option<FieldFeature> {
        let given_name_patterns = [
            r"(?i)given\s*names?[\s:]*([A-Za-z\s-]{2,50})",
            r"(?i)first\s*names?[\s:]*([A-Za-z\s-]{2,50})",
            r"(?i)nombres?[\s:]*([A-Za-z\s-]{2,50})",
            r"(?i)prénoms?[\s:]*([A-Za-z\s-]{2,50})",
            r"(?i)vornamen?[\s:]*([A-Za-z\s-]{2,50})",
        ];
        
        self.extract_field_with_patterns(given_name_patterns, "name", 0.65)
    }
    
    // ML-driven extraction of gender with confidence score
    fn extract_gender(&self, text: &str) -> Option<FieldFeature> {
        // Basic gender extraction patterns
        let gender_patterns = [
            r"(?i)sex[\s:]*([MF])",
            r"(?i)gender[\s:]*([MF])",
            r"(?i)sexo[\s:]*([MF])",
            r"(?i)sexe[\s:]*([MF])",
            r"(?i)geschlecht[\s:]*([MF])",
        ];
        
        // Extended gender word patterns
        let gender_word_patterns = [
            r"(?i)sex[\s:]*(?:male|masculino|männlich|homme)",
            r"(?i)gender[\s:]*(?:male|masculino|männlich|homme)",
            r"(?i)sex[\s:]*(?:female|femenino|weiblich|femme)",
            r"(?i)gender[\s:]*(?:female|femenino|weiblich|femme)",
        ];
        
        // Simple extractor for M/F values
        if let Some(feature) = self.extract_field_with_patterns(gender_patterns, "gender", 0.8) {
            return Some(feature);
        }
        
        // For word-based gender indicators
        for pattern in &gender_word_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text) {
                    let is_male = pattern.contains("male|masculino|männlich|homme");
                    let gender_value = if is_male { "M" } else { "F" };
                    
                    return Some(FieldFeature {
                        field_type: "gender".to_string(),
                        confidence: 0.75,
                        bounding_box: (0, 0, 0, 0),
                        text_content: gender_value.to_string(),
                        features: vec![0.75, 0.0, 0.0, 0.0, 0.0],
                    });
                }
            }
        }
        
        // Look for isolated M or F characters
        if let Ok(re) = Regex::new(r"(?i)\bsex\b[\s\S]{1,20}\b([MF])\b") {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    return Some(FieldFeature {
                        field_type: "gender".to_string(),
                        confidence: 0.68,
                        bounding_box: (0, 0, 0, 0),
                        text_content: m.as_str().to_uppercase(),
                        features: vec![0.68, 0.0, 0.0, 0.0, 0.0],
                    });
                }
            }
        }
        
        None
    }
    
    // ML-driven extraction of date of birth with confidence score
    fn extract_date_of_birth(&self, text: &str) -> Option<FieldFeature> {
        let dob_patterns = [
            r"(?i)date\s*of\s*birth[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)birth[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)dob[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)fecha\s*de\s*nacimiento[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)date\s*de\s*naissance[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)geburtsdatum[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
        ];
        
        self.extract_field_with_patterns(dob_patterns, "date_of_birth", 0.75)
    }
    
    // ML-driven extraction of date of expiry with confidence score
    fn extract_date_of_expiry(&self, text: &str) -> Option<FieldFeature> {
        let expiry_patterns = [
            r"(?i)date\s*of\s*expiry[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)expiry[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)expiration[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)fecha\s*de\s*caducidad[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)date\s*d['e]expiration[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
            r"(?i)gültig\s*bis[\s:]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{4})",
        ];
        
        self.extract_field_with_patterns(expiry_patterns, "expiry_date", 0.75)
    }
    
    // Generic pattern-based field extractor with confidence scoring
    fn extract_field_with_patterns<const N: usize>(&self, patterns: [&str; N], field_type: &str, base_confidence: f32) -> Option<FieldFeature> {
        let mut highest_confidence = 0.0;
        let mut best_match = None;
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(pattern) {
                    if let Some(m) = cap.get(1) {
                        let value = m.as_str().trim().to_string();
                        
                        // Calculate confidence based on pattern match and content
                        let confidence = base_confidence * (1.0 - (0.1 * (patterns.iter().position(|&p| p == *pattern).unwrap_or(0) as f32)));
                        
                        if confidence > highest_confidence {
                            highest_confidence = confidence;
                            best_match = Some(FieldFeature {
                                field_type: field_type.to_string(),
                                confidence,
                                bounding_box: (0, 0, 0, 0),
                                text_content: value,
                                features: vec![confidence, 0.0, 0.0, 0.0, 0.0],
                            });
                        }
                    }
                }
            }
        }
        
        best_match
    }
    
    // Detect MRZ region in passport image
    fn detect_mrz_region(&self, text: &str) -> Option<FieldFeature> {
        // MRZ format patterns (TD3 passport format)
        let mrz_patterns = [
            r"(?i)P[A-Z<][A-Z<]{3}.*",  // First line starts with P followed by country code
            r"(?i)[A-Z0-9<]{9}[0-9][A-Z0-9<].*",  // Second line often has 9 alphanumeric chars, then check digit
        ];
        
        for pattern in &mrz_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for line in text.lines() {
                    if re.is_match(line) && line.len() >= 30 && line.contains('<') {
                        return Some(FieldFeature {
                            field_type: "mrz".to_string(),
                            confidence: 0.85,
                            bounding_box: (0, 0, 0, 0),
                            text_content: line.to_string(),
                            features: vec![0.85, 0.0, 0.0, 0.0, 0.0],
                        });
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_extraction() {
        let extractor = FeatureExtractor::new();
        
        // Sample OCR text for testing
        let test_text = "PASSPORT\nDocument No: AB123456\nSurname: SMITH\nGiven Names: JOHN JAMES\nNationality: USA\nDate of Birth: 01/01/1980\nSex: M\nDate of Issue: 01/01/2015\nDate of Expiry: 01/01/2025\nAuthority: DEPARTMENT OF STATE\nP<USASMITH<<JOHN<JAMES<<<<<<<<<<<<<<<<<<<<<\n1234567890USA8001019M2501019<<<<<<<<<<<<<<00";
        
        let features = extractor.extract_features(&[], test_text);
        
        // Check if we extracted the expected fields
        assert!(features.iter().any(|f| f.field_type == "document_number" && f.text_content == "AB123456"));
        assert!(features.iter().any(|f| f.field_type == "name" && f.text_content == "SMITH"));
        assert!(features.iter().any(|f| f.field_type == "gender" && f.text_content == "M"));
        assert!(features.iter().any(|f| f.field_type == "date_of_birth" && f.text_content == "01/01/1980"));
        assert!(features.iter().any(|f| f.field_type == "expiry_date" && f.text_content == "01/01/2025"));
        assert!(features.iter().any(|f| f.field_type == "mrz"));
    }
}
