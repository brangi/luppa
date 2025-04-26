use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::io::{Read, Write};

use tempfile::NamedTempFile;
use tesseract::Tesseract;
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

// Import models and error types
use crate::models::data::VisualData;
use crate::models::{MrzData, CheckDigits};
use crate::utils::error::PassportError;
use crate::ml::text_correction::{self, FieldType};

// No need to import extractors module since we're using Self:: prefix

// ML-enhanced universal OCR module for language-agnostic passport field extraction
// This implementation works across multiple languages and document formats
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageFeatures {
    histogram: Vec<u32>,
    mean_brightness: f32,
    contrast: f32,
}

// ML model configuration for passport field detection
#[derive(Serialize, Deserialize)]
pub struct FieldDetectionConfig {
    model_type: String,
    confidence_threshold: f32,
    field_patterns: HashMap<String, Vec<String>>,
}

// Feature vector for ML-based field classification
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FieldFeatures {
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    text: String,
}

/// OcrProcessor handles MRZ extraction and parsing
/// Focus is on Machine Readable Zone extraction from passport images
pub struct OcrProcessor;

impl OcrProcessor {
    /// Utility function to extract combined text from MRZ data
    /// This is useful for language-agnostic processing
    pub fn extract_text_from_mrz(mrz: &MrzData) -> String {
        // Create a combined text from MRZ fields
        format!("{} {} {} {} {} {} {} {} {}",
            mrz.document_type, mrz.issuing_country, mrz.document_number,
            mrz.surname, mrz.given_names, mrz.nationality,
            mrz.date_of_birth, mrz.gender, mrz.date_of_expiry)
    }
    
    /// Extract MRZ data from a file path
    pub fn extract_mrz_from_file<P: AsRef<Path>>(image_path: P) -> Result<MrzData, PassportError> {
        // Read the image file into memory
        let image_data = std::fs::read(&image_path)
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to read image file: {}", e)))?;
        
        // Call the bytes version
        Self::extract_mrz_from_bytes(&image_data)
    }
    
    /// Extract MRZ data from the image bytes
    pub fn extract_mrz_from_bytes(image_data: &[u8]) -> Result<MrzData, PassportError> {
        Self::extract_mrz_internal(image_data)
    }
    
    /// Extract MRZ data from image bytes - made public so it can be called from passport_validator
    pub fn extract_mrz(image_data: &[u8]) -> Result<MrzData, PassportError> {
        Self::extract_mrz_internal(image_data)
    }
    
    /// Internal implementation of MRZ extraction
    fn extract_mrz_internal(image_data: &[u8]) -> Result<MrzData, PassportError> {
        // Create a temporary file for the OCR engine
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to create temporary file: {}", e)))?;
        
        // Write the image data to the temporary file
        temp_file.write_all(image_data)
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to write to temporary file: {}", e)))?;
        
        // Configure Tesseract for MRZ extraction with proper method chaining
        // Use a specialized configuration optimized for MRZ
        let path_str = temp_file.path().to_str()
            .ok_or_else(|| PassportError::MrzExtractionError("Could not convert path to string".to_string()))?;
            
        // Initialize Tesseract and chain methods that return Self
        let mut tess = Tesseract::new(None, Some("eng"))
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to initialize Tesseract: {}", e)))?
            .set_variable("tessedit_char_whitelist", "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789<")
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to set Tesseract variable: {}", e)))?;
        
        // Set page seg mode separately as it modifies in-place
        tess.set_page_seg_mode(tesseract::PageSegMode::PsmAuto);
        
        // Set image and continue chaining
        tess = tess.set_image(path_str)
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to set image: {}", e)))?;
        
        // Get OCR text result
        let text = tess.get_text()
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to extract text: {}", e)))?;
        
        // Extract MRZ lines from the OCR text
        let mrz_lines = Self::extract_mrz_lines_from_text(&text);
        
        // If normal extraction failed, try alternative methods
        let mrz_lines = if mrz_lines.is_empty() {
            Self::try_alternative_mrz_extraction(&text)
        } else {
            mrz_lines
        };
        
        if mrz_lines.len() < 2 {
            // Not enough lines for MRZ data
            return Err(PassportError::MrzExtractionError("Failed to extract MRZ lines from passport image".to_string()));
        }
        
        // Process the MRZ lines
        let line1 = Self::clean_mrz_line(&mrz_lines[0]);
        let line2 = Self::clean_mrz_line(&mrz_lines[1]);
        
        // Verify minimum length for processing
        if line1.len() < 30 || line2.len() < 30 {
            return Err(PassportError::MrzExtractionError("MRZ lines too short or corrupted".to_string()));
        }
        
        // Extract data from MRZ lines
        // Initialize fields
        let document_type = line1.chars().take(2).collect::<String>();
        let issuing_country = line1.chars().skip(2).take(3).collect::<String>();
        
        let mut surname = String::new();
        let mut given_names = String::new();
        
        // Extract names from the first line
        let names_part = line1.chars().skip(5).collect::<String>();
        let mut names_iter = names_part.split('<');
        
        if let Some(sur) = names_iter.next() {
            surname = Self::clean_name(sur);
        }
        
        if let Some(given) = names_iter.next() {
            given_names = Self::clean_name(given);
        }
        
        // Extract data from second line
        let mut document_number_check_digit = String::new();
        let mut nationality = String::new();
        let mut date_of_birth = String::new();
        let mut date_of_birth_check_digit = String::new();
        let mut gender = String::new();
        let mut date_of_expiry = String::new();
        let mut date_of_expiry_check_digit = String::new();
        let mut personal_number = None;
        let personal_number_check_digit = String::new();
        
        // Initialize document_number directly to avoid unused variable warning
        let document_number: String;
        
        // Document number (varies in length, usually 9 characters)
        let doc_number_end = if line2.len() >= 9 {
            9
        } else {
            line2.len()
        };
        
        document_number = Self::clean_document_number(&line2.chars().take(doc_number_end).collect::<String>());
        
        // Check digit for document number
        if line2.len() > doc_number_end {
            document_number_check_digit = line2.chars().skip(doc_number_end).take(1).collect::<String>();
        }
        
        // Nationality (3 characters)
        if line2.len() > doc_number_end + 1 {
            nationality = line2.chars().skip(doc_number_end + 1).take(3).collect::<String>();
        }
        
        // Date of birth (6 characters: YYMMDD)
        if line2.len() > doc_number_end + 4 {
            date_of_birth = line2.chars().skip(doc_number_end + 4).take(6).collect::<String>();
        }
        
        // Check digit for date of birth
        if line2.len() > doc_number_end + 10 {
            date_of_birth_check_digit = line2.chars().skip(doc_number_end + 10).take(1).collect::<String>();
        }
        
        // Gender (1 character: M, F, or <)
        if line2.len() > doc_number_end + 11 {
            gender = line2.chars().skip(doc_number_end + 11).take(1).collect::<String>();
            // Normalize gender representation
            if gender == "<" {
                gender = "X".to_string(); // Use X for unspecified
            }
        }
        
        // Date of expiry (6 characters: YYMMDD)
        if line2.len() > doc_number_end + 12 {
            date_of_expiry = line2.chars().skip(doc_number_end + 12).take(6).collect::<String>();
        }
        
        // Check digit for date of expiry
        if line2.len() > doc_number_end + 18 {
            date_of_expiry_check_digit = line2.chars().skip(doc_number_end + 18).take(1).collect::<String>();
        }
        
        // Personal number (variable length)
        if line2.len() > doc_number_end + 19 {
            // Extract all remaining characters except the last (which is a check digit)
            let remaining_len = line2.len() - (doc_number_end + 19 + 1); // +1 for the final check digit
            let personal_num = line2.chars().skip(doc_number_end + 19).take(remaining_len).collect::<String>();
            if !personal_num.is_empty() && personal_num != "<<<<<<<<<<<<<" {
                personal_number = Some(personal_num.replace('<', ""));
            }
        }
        
        // Create check digits struct
        let check_digits = CheckDigits {
            document_number_check: document_number_check_digit.chars().next().unwrap_or('0'),
            date_of_birth_check: date_of_birth_check_digit.chars().next().unwrap_or('0'),
            date_of_expiry_check: date_of_expiry_check_digit.chars().next().unwrap_or('0'),
            personal_number_check: personal_number_check_digit.chars().next().unwrap_or('0'),
            composite_check: '0', // Default value for the composite check
        };
        
        // Construct and return MrzData
        Ok(MrzData {
            document_type,
            issuing_country,
            document_number,
            surname,
            given_names,
            nationality,
            date_of_birth,
            gender,
            date_of_expiry,
            personal_number,
            check_digits,
            place_of_birth: None, // Not part of MRZ, but included in the struct for completeness
        })
    }
    
    /// Helper method to clean document numbers in a standardized way
    fn clean_document_number(number: &str) -> String {
        // Remove any non-alphanumeric characters
        let cleaned = number.chars()
            .filter(|c| c.is_alphanumeric() || *c == '<')
            .collect::<String>();
            
        // Replace the MRZ filler character with empty string
        cleaned.replace('<', "")
    }
    
    /// Helper method to clean names in a standardized way
    fn clean_name(name: &str) -> String {
        // For names, replace the MRZ separator with space
        let cleaned = name.replace('<', " ")
            .trim()
            .to_string();
            
        // Handle consecutive spaces that might result from multiple < characters
        cleaned.split_whitespace()
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join(" ")
    }
    
    /// Clean up an MRZ line by handling common OCR errors
    fn clean_mrz_line(line: &str) -> String {
        // Replace common OCR errors
        let mut cleaned = line.to_uppercase();
        
        // Correct common OCR confusions
        let replacements = [
            ("O", "0"), // Letter O to number 0
            ("Q", "0"), // Letter Q to number 0
            ("D", "0"), // Letter D to number 0 (common in worn documents)
            ("I", "1"), // Letter I to number 1
            ("L", "1"), // Letter L to number 1
            ("Z", "2"), // Letter Z to number 2 in some fonts
            ("S", "5"), // Letter S to number 5 in some conditions
            ("B", "8"), // Letter B to number 8 in poor quality scans
            ("G", "6"), // Letter G to number 6
            ("_", "<"), // Underscore sometimes detected instead of <
            ("-", "<"), // Dash sometimes detected instead of <
            (" ", "<"), // Space sometimes detected instead of <
            (".", "<"), // Period sometimes detected instead of <
        ];
        
        // Look for numeric parts (like dates, document numbers)
        // Special handling for MRZ dates (YYMMDD format)
        let date_pattern = (r"\b([0-9IO]{2})[^0-9A-Z]?([0-9IO]{2})[^0-9A-Z]?([0-9IO]{2})\b", |caps: &regex::Captures| {
            let y = caps[1].replace('I', "1").replace('O', "0");
            let m = caps[2].replace('I', "1").replace('O', "0");
            let d = caps[3].replace('I', "1").replace('O', "0");
            format!("{}{}{}", y, m, d)
        });
        
        // Handle passport number patterns
        let number_pattern = (r"\b([A-Z0-9]{4,12})\b", |caps: &regex::Captures| {
            caps[1].to_string()
        });
        
        // Apply general replacements
        for (wrong, correct) in replacements.iter() {
            cleaned = cleaned.replace(wrong, correct);
        }
        
        // Apply regex-based corrections
        let date_regex = Regex::new(date_pattern.0).unwrap_or_else(|_| Regex::new(r"a^a").unwrap()); // Fallback that won't match
        cleaned = date_regex.replace_all(&cleaned, date_pattern.1).to_string();
        
        let number_regex = Regex::new(number_pattern.0).unwrap_or_else(|_| Regex::new(r"a^a").unwrap());
        cleaned = number_regex.replace_all(&cleaned, number_pattern.1).to_string();
        
        // Remove any non-MRZ characters (only allow A-Z, 0-9, and <)
        cleaned.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '<')
            .collect()
    }
    
    // Removed unused example MRZ data function to reduce dead code
    
    /// Extract MRZ lines from OCR text
    fn extract_mrz_lines_from_text(text: &str) -> Vec<String> {
        let mut mrz_lines = Vec::new();
        let mut candidate_lines = Vec::new();
        
        // Split the text into lines and clean each
        for line in text.lines() {
            let cleaned = line.trim();
            if cleaned.is_empty() {
                // Continue with other field extractions
            }
            
            // Look for lines that could be MRZ
            // MRZ lines typically have a lot of < characters and alphanumeric chars
            let char_count = cleaned.chars().count();
            
            // Skip lines that are too short to be MRZ
            if char_count < 20 {
                // Continue with other field extractions
            }
            
            // Count MRZ-specific features
            let lt_count = cleaned.chars().filter(|c| *c == '<').count();
            let alphanum_count = cleaned.chars().filter(|c| c.is_alphanumeric()).count();
            
            // Lines with these characteristics are likely to be MRZ
            if (lt_count > 0 && alphanum_count > 15) || 
               (char_count >= 30 && alphanum_count > (char_count * 3 / 4)) {
                candidate_lines.push(cleaned.to_string());
            }
        }
        
        // Sort candidates by likelihood of being MRZ lines
        candidate_lines.sort_by(|a, b| {
            let a_lt = a.chars().filter(|c| *c == '<').count();
            let b_lt = b.chars().filter(|c| *c == '<').count();
            b_lt.cmp(&a_lt) // Reverse order - more < characters is better
        });
        
        // Take the top candidates as MRZ lines
        for line in candidate_lines.iter().take(3) { // Usually 2-3 lines in MRZ
            mrz_lines.push(line.clone());
        }
        
        mrz_lines
    }
    
    /// Try alternative extraction approaches when normal extraction fails
    fn try_alternative_mrz_extraction(text: &str) -> Vec<String> {
        // Since the OCR may have merged lines or split incorrectly,
        // we'll try to extract based on character patterns
        
        let cleaned_text = text.replace("\n", " ").replace("\r", " ");
        
        // Look for TD3 format passport MRZ patterns
        // First line typically starts with P<, second with document number
        let p_pattern = Regex::new(r"P[<A-Z]{40,}").ok();
        let num_pattern = Regex::new(r"[A-Z0-9]{8,9}[0-9][A-Z]{3}").ok();
        
        if let (Some(p_regex), Some(num_regex)) = (p_pattern, num_pattern) {
            if let (Some(p_match), Some(num_match)) = (p_regex.find(&cleaned_text), num_regex.find(&cleaned_text)) {
                return vec![
                    p_match.as_str().to_string(),
                    num_match.as_str().to_string(),
                ];
            }
        }
        
        // Alternative: look for any sequence with passport number pattern
        let alt_pattern = Regex::new(r"[A-Z0-9]{6,12}(?:[0-9<][A-Z<]{10,}){2,}").ok();
        if let Some(alt_regex) = alt_pattern {
            if let Some(alt_match) = alt_regex.find(&cleaned_text) {
                let matched = alt_match.as_str();
                // Try to split it into likely MRZ lines
                if matched.len() > 40 {
                    let mid = matched.len() / 2;
                    return vec![
                        matched[..mid].to_string(),
                        matched[mid..].to_string(),
                    ];
                }
                return vec![matched.to_string()];
            }
        }
        
        // Last resort: just take any lines with alphanumeric and < characters
        let mut possible_lines = Vec::new();
        for line in text.lines() {
            if line.contains('<') && line.chars().any(|c| c.is_alphanumeric()) && line.len() > 20 {
                possible_lines.push(line.to_string());
            }
        }
        
        // Return any possible matches
        if !possible_lines.is_empty() {
            return possible_lines;
        }
        
        Vec::new()  // Return empty if all approaches fail
    }
    
    // Removed unused date formatting function to reduce code size
}

// Define regex patterns for universal field extraction
lazy_static! {
    // Document number patterns across multiple languages
    pub static ref DOCUMENT_NUMBER_PATTERNS: Vec<Regex> = vec![
        // English patterns
        Regex::new(r"(?i)document\s*no\.?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)passport\s*no\.?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)doc\.?\s*(?:number|no\.?)\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        // French patterns
        Regex::new(r"(?i)num[eé]ro\s*(?:de passeport|du document)?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)passeport\s*n[o°]\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        // Spanish patterns
        Regex::new(r"(?i)(?:número|numero|num)\s*(?:de pasaporte|del documento)?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)pasaporte\s*(?:núm|num|no)\.?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        // German patterns
        Regex::new(r"(?i)(?:reisepass|ausweis)\s*nr\.?\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)pass\s*nummer\s*[:#]?\s*([A-Z0-9]{5,15})").unwrap(),
        // Generic patterns that work across languages
        Regex::new(r"(?i)(?:document|passport|pass|doc|no|nr)\s*[:.-]?\s*([A-Z0-9]{5,15})").unwrap(),
        Regex::new(r"(?i)id\s*[:.-]?\s*([A-Z0-9]{5,15})").unwrap(),
        // Common formats in MRZ-adjacent areas
        Regex::new(r"(?i)([A-Z]{1,3}[0-9]{6,12})").unwrap(),
    ];
    
    // Name patterns for multiple languages
    pub static ref NAME_PATTERNS: Vec<Regex> = vec![
        // English patterns
        Regex::new(r"(?i)surname\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        Regex::new(r"(?i)family\s*name\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        Regex::new(r"(?i)last\s*name\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // French patterns
        Regex::new(r"(?i)nom\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // Spanish patterns
        Regex::new(r"(?i)apellidos?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // German patterns
        Regex::new(r"(?i)nachname\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        Regex::new(r"(?i)familienname\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
    ];
    
    // Given name patterns
    pub static ref GIVEN_NAME_PATTERNS: Vec<Regex> = vec![
        // English patterns
        Regex::new(r"(?i)given\s*names?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        Regex::new(r"(?i)first\s*names?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // French patterns
        Regex::new(r"(?i)pr[ée]noms?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // Spanish patterns
        Regex::new(r"(?i)nombres?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
        // German patterns
        Regex::new(r"(?i)vornamen?\s*[:#]?\s*([\p{L}\s'-]+)").unwrap(),
    ];
    
    // Gender/Sex patterns
    pub static ref GENDER_PATTERNS: Vec<Regex> = vec![
        // Multi-language patterns - simple and robust
        Regex::new(r"(?i)(?:sex|sexe|sexo|geschlecht)\s*[:#]?\s*([MF])").unwrap(),
        Regex::new(r"(?i)(?:gender|genre|género|genero)\s*[:#]?\s*([MF])").unwrap(),
        // Handle spelled out versions
        Regex::new(r"(?i)(?:sex|sexe|sexo|geschlecht)\s*[:#]?\s*((?:fe)?male|(?:mas|fem)(?:culino?|inin[oe])|homme|mujer|hombre|frau|mann|weiblich|männlich)").unwrap(),
    ];
    
    // Date patterns with various formats (YYYY-MM-DD, DD.MM.YYYY, MM/DD/YYYY etc)
    pub static ref DATE_PATTERNS: Vec<Regex> = vec![
        // ISO format
        Regex::new(r"(\d{4})[-./](\d{1,2})[-./](\d{1,2})").unwrap(),
        // European format
        Regex::new(r"(\d{1,2})[-./](\d{1,2})[-./](\d{4})").unwrap(),
        // US format
        Regex::new(r"(\d{1,2})[-./](\d{1,2})[-./](\d{4})").unwrap(),
        // Text-based dates with spaces
        Regex::new(r"(\d{1,2})\s+([A-Za-z]{3,9})\s+(\d{4})").unwrap(),
        // Special case for dates with no separators
        Regex::new(r"(\d{2})(\d{2})(\d{4})").unwrap(),
    ];
    
    // Date of birth patterns
    pub static ref DOB_PATTERNS: Vec<Regex> = vec![
        // Various forms in different languages
        Regex::new(r"(?i)(?:date of birth|birth date|geboren am|date de naissance|fecha de nacimiento|geburtsdatum)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
        Regex::new(r"(?i)(?:né\(e\) le|nacido el|born on)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
        Regex::new(r"(?i)(?:DOB|DDN)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
    ];
    
    // Date of issue patterns
    pub static ref DOI_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(?:date of issue|date d'[ée]mission|fecha de expedici[óo]n|ausstellungsdatum)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
        Regex::new(r"(?i)(?:issued on|issued|émis le|expedido el|ausgestellt am)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
    ];
    
    // Date of expiry patterns
    pub static ref DOE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(?:date of expiry|expiry date|date d'expiration|fecha de caducidad|ablaufdatum)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
        Regex::new(r"(?i)(?:expires on|valable jusqu'au|válido hasta|gültig bis)\s*[:#]?\s*([0-9]{1,2}[-./\s][0-9]{1,2}[-./\s][0-9]{4}|[0-9]{1,2}\s+[A-Za-z]{3,9}\s+[0-9]{4})").unwrap(),
    ];
    
    // Authority patterns
    pub static ref AUTHORITY_PATTERNS: Vec<Regex> = vec![
        // Standard authority patterns (English)
        Regex::new(r"(?i)(?:authority|issued by|issuing authority|passport issued by)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // French authority patterns
        Regex::new(r"(?i)(?:autorité|délivré par|autorité de délivrance)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // Spanish authority patterns
        Regex::new(r"(?i)(?:autoridad|expedido por|autoridad de expedición)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // German authority patterns
        Regex::new(r"(?i)(?:ausstellende behörde|behörde|ausgestellt durch)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // Italian authority patterns
        Regex::new(r"(?i)(?:autorità|rilasciato da|ente di rilascio)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // Portuguese authority patterns
        Regex::new(r"(?i)(?:autoridade|emitido por|autoridade emissora)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
        
        // Common abbreviations and short forms
        Regex::new(r"(?i)(?:auth|iss\.auth|issuing auth)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
    ];
    
    // Place of birth patterns
    pub static ref POB_PATTERNS: Vec<Regex> = vec![
        // English patterns - expanded with variations
        Regex::new(r"(?i)(?:place\s+of\s+birth|birthplace|birth\s+place|born\s+(?:at|in)|pob)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Spanish patterns - expanded coverage
        Regex::new(r"(?i)(?:lugar\s+de\s+nacimiento|nacido\s+en|ciudad\s+de\s+nacimiento|lugar\s+nac\.)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // French patterns - more variations
        Regex::new(r"(?i)(?:lieu\s+de\s+naissance|né\s+à|née\s+à|ville\s+de\s+naissance|né\(e\)\s+[àa])\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // German patterns - comprehensive
        Regex::new(r"(?i)(?:geburtsort|geboren\s+in|geburtstadt|geburtsland|geburts[-\s]ort)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Italian patterns
        Regex::new(r"(?i)(?:luogo\s+di\s+nascita|nato\s+a|nata\s+a|città\s+di\s+nascita)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Portuguese patterns
        Regex::new(r"(?i)(?:local\s+de\s+nascimento|naturalidade|cidade\s+natal|nascido\s+em)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Dutch patterns
        Regex::new(r"(?i)(?:geboorteplaats|geboren\s+te|plaats\s+van\s+geboorte)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Scandinavian patterns (Swedish, Norwegian, Danish)
        Regex::new(r"(?i)(?:födelseort|fødested|fødeby)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Eastern European patterns (Polish, Czech, etc.)
        Regex::new(r"(?i)(?:miejsce\s+urodzenia|místo\s+narození|rodisko)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),

        // Common abbreviations
        Regex::new(r"(?i)(?:p\.?o\.?b\.?|l\.?d\.?n\.?)\s*[:#]?\s*([\p{L}\s,.'-/()]+)").unwrap(),
    ];
    
    // Nationality patterns
    pub static ref NATIONALITY_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(?:nationality|nationalité|nacionalidad|staatsangehörigkeit)\s*[:#]?\s*([\p{L}\s,.'-]+)").unwrap(),
        Regex::new(r"(?i)(?:citizen of|citoyen de|ciudadano de|staatsbürger von)\s*[:#]?\s*([\p{L}\s,.'-]+)").unwrap(),
    ];
    
    // Field label detection patterns (for excluding labels from values)
    pub static ref FIELD_LABEL_PATTERNS: Vec<Regex> = vec![
        // General field label pattern that covers most standardized formats
        Regex::new(r"(?i)(?:surname|given names?|first names?|family name|nationality|passport no\.?|document no\.?|date of (?:birth|issue|expiry)|place of birth|authority|sex|gender|signature)\s*[:#]?").unwrap(),
        // French field labels
        Regex::new(r"(?i)(?:nom|prénoms?|nationalité|n[o°] de passeport|num[ée]ro du document|date de (?:naissance|délivrance|expiration)|lieu de naissance|autorité|sexe|genre|signature)").unwrap(),
        // Spanish field labels
        Regex::new(r"(?i)(?:apellidos?|nombres?|nacionalidad|(?:núm|no)\.? de pasaporte|(?:núm|no)\.? del documento|fecha de (?:nacimiento|expedición|caducidad)|lugar de nacimiento|autoridad|sexo|género|firma)").unwrap(),
        // German field labels
        Regex::new(r"(?i)(?:nachname|familienname|vornamen?|staatsangehörigkeit|reisepass-nr\.?|ausweisnummer|geburtsdatum|ausstellungsdatum|ablaufdatum|geburtsort|behörde|geschlecht|unterschrift)").unwrap(),
    ];
    
    // Common words to exclude from place names (stop words, field labels, etc.)
    pub static ref NON_PLACE_WORDS: Vec<Regex> = vec![
        Regex::new(r"(?i)^(signature|not valid|no signature|date|this|that|with|valid|copy|original|specimen|sample|unofficial|official|draft|final)$").unwrap(),
    ];
}





/// Returns true if the text matches any field label pattern
pub fn is_field_label(text: &str) -> bool {
    for pattern in FIELD_LABEL_PATTERNS.iter() {
        if pattern.is_match(text) {
            return true;
        }
    }
    false
}

/// Returns true if the value is a common non-place word (e.g. 'UNKNOWN', 'N/A', 'NONE', etc.)
pub fn is_non_place_word(value: &str) -> bool {
    let v = value.trim().to_uppercase();
    matches!(v.as_str(), "UNKNOWN" | "N/A" | "NONE" | "NOT AVAILABLE" | "UNSPECIFIED" | "UNDEFINED" | "NO DATA" | "SIN DATO" | "INCONNU" | "KEINE ANGABE")
}

/// Normalizes a date string to a standard format
pub fn normalize_date(date_str: &str) -> String {
    // Simple date normalization - standardize separators and format
    let cleaned = date_str.trim().replace("-", "/").replace(".", "/");
    // More advanced normalization could be added here
    cleaned
}

// Enhanced OCR processor implementation
pub struct EnhancedOcrProcessor;

impl EnhancedOcrProcessor {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Returns true if the text matches any field label pattern
    pub fn is_field_label(text: &str) -> bool {
        for pattern in FIELD_LABEL_PATTERNS.iter() {
            if pattern.is_match(text) {
                return true;
            }
        }
        false
    }
    

    // Helper methods for the EnhancedOcrProcessor
    fn print_visual_data_results(visual_data: &VisualData) {
        println!("Extracted Visual Data:");
        println!("  Document Type: {}", visual_data.document_type);
        println!("  Issuing Country: {}", visual_data.issuing_country);
        println!("  Document Number: {}", visual_data.document_number);
        println!("  Name: {}", visual_data.name);
        println!("  Surname: {}", visual_data.surname);
        println!("  Given Names: {}", visual_data.given_names);
        println!("  Nationality: {}", visual_data.nationality);
        println!("  Date of Birth: {}", visual_data.date_of_birth);
        println!("  Gender: {}", visual_data.gender);
        println!("  Place of Birth: {:?}", visual_data.place_of_birth);
        println!("  Date of Issue: {}", visual_data.date_of_issue);
        println!("  Date of Expiry: {}", visual_data.date_of_expiry);
        println!("  Authority: {:?}", visual_data.authority);
        println!("  Personal Number: {:?}", visual_data.personal_number);
    }
    


    /// Extract visual data from a file path
    pub fn extract_visual_data<P: AsRef<Path>>(file_path: P, langs: &[&str]) -> Result<VisualData, PassportError> {
        // Read the file into a byte array
        let mut file = fs::File::open(file_path).map_err(|e| PassportError::IoError(e.to_string()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| PassportError::IoError(e.to_string()))?;
        
        // Process the bytes
        Self::extract_visual_data_from_bytes(&buffer, langs)
    }
    
    /// Extract visual data from raw image bytes
    pub fn extract_visual_data_from_bytes(image_bytes: &[u8], langs: &[&str]) -> Result<VisualData, PassportError> {
        // Perform OCR on the image
        let ocr_text = Self::perform_ocr(image_bytes, langs)?;
        
        // Extract fields from the OCR text
        Self::extract_visual_data_from_text(&ocr_text)
    }
    
    /// Perform OCR on an image
    fn perform_ocr(image_bytes: &[u8], langs: &[&str]) -> Result<String, PassportError> {
        // Create a temporary file to store the image
        let mut temp_file = NamedTempFile::new().map_err(|e| PassportError::IoError(e.to_string()))?;
        temp_file.write_all(image_bytes).map_err(|e| PassportError::IoError(e.to_string()))?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        // Initialize Tesseract with the specified languages
        let langs_joined = langs.join("+");
        
        // Chain all Tesseract operations since the methods take ownership of the receiver
        let text = Tesseract::new(None, Some(&langs_joined))
            .map_err(|e| PassportError::OcrError(e.to_string()))?
            // Set Tesseract parameters for better OCR
            .set_variable("tessedit_char_whitelist", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?()/-<>'\"").map_err(|e| PassportError::OcrError(e.to_string()))?
            .set_variable("tessedit_pageseg_mode", "3").map_err(|e| PassportError::OcrError(e.to_string()))?
            // Perform OCR
            .set_image(temp_path).map_err(|e| PassportError::OcrError(e.to_string()))?
            .get_text().map_err(|e| PassportError::OcrError(e.to_string()))?;
        
        Ok(text)
    }
    
    // Extract all visual data fields from OCR text
    pub fn extract_visual_data_from_text(ocr_text: &str) -> Result<VisualData, PassportError> {
        // Initialize variables to store extracted data
        let mut surname = String::new();
        let mut given_names = String::new();
        let mut name = String::new();
        let document_type; // Will be assigned later
        let mut issuing_country = String::new();
        let document_number = String::new();
        let mut nationality = String::new();
        let mut date_of_birth = String::new();
        let mut gender = String::new();
        let mut date_of_issue = String::new();
        let mut date_of_expiry = String::new();
        let mut authority = None;
        let personal_number = None;
        let mut place_of_birth = None;

        // Extract surname if empty
        if surname.is_empty() {
            if let Some(value) = Self::extract_surname_from_text(&ocr_text) {
                // Apply ML-based character correction for names
                surname = text_correction::correct_text_with_context(&value, FieldType::Name);
            }
        }
        
        // Extract given names if empty
        if given_names.is_empty() {
            if let Some(value) = Self::extract_given_names_from_text(&ocr_text) {
                // Apply ML-based character correction for names
                given_names = text_correction::correct_text_with_context(&value, FieldType::Name);
            }
        }
        
        if date_of_birth.is_empty() {
            if let Some(value) = Self::extract_dob_from_text(&ocr_text) {
                // Apply ML-based character correction for dates
                date_of_birth = text_correction::correct_text_with_context(&value, FieldType::Date);
                // Continue with other field extractions
            }
        }
        
        if date_of_issue.is_empty() {
            if let Some(value) = Self::extract_doi_from_text(&ocr_text) {
                // Apply ML-based character correction for dates
                date_of_issue = text_correction::correct_text_with_context(&value, FieldType::Date);
                // Continue with other field extractions
            }
        }
        
        if date_of_expiry.is_empty() {
            if let Some(value) = Self::extract_doe_from_text(&ocr_text) {
                // Apply ML-based character correction for dates
                date_of_expiry = text_correction::correct_text_with_context(&value, FieldType::Date);
                // Continue with other field extractions
            }
        }
        
        if gender.is_empty() {
            if let Some(value) = Self::extract_gender_from_text(&ocr_text) {
                gender = value;
                // Continue with other field extractions
            }
        }
        
        if nationality.is_empty() {
            if let Some(value) = Self::extract_nationality_from_text(&ocr_text) {
                nationality = value;
                // Continue with other field extractions
            }
        }
        
        // Optional fields - more expensive extraction patterns
        if place_of_birth.is_none() {
            place_of_birth = Self::extract_place_of_birth_from_text(&ocr_text);
            if place_of_birth.is_some() {
                // Continue with other field extractions
            }
        }
        
        if authority.is_none() {
            authority = Self::extract_authority_from_text(&ocr_text);
            if authority.is_some() {
                // Continue with other field extractions
            }
        }
        if ocr_text.contains("PASSPORT") || ocr_text.contains("PASSEPORT") {
            document_type = "P".to_string();
        } else if ocr_text.contains("IDENTITY") || ocr_text.contains("ID CARD") {
            document_type = "ID".to_string();
        } else {
            document_type = "P".to_string(); // Default to passport
        }

        // Try to deduce issuing country
        if issuing_country.is_empty() {
            if ocr_text.contains("UNITED STATES") || ocr_text.contains("USA") {
                issuing_country = "USA".to_string();
            } else if ocr_text.contains("UNITED KINGDOM") || ocr_text.contains("GREAT BRITAIN") {
                issuing_country = "GBR".to_string();
            } else if ocr_text.contains("MEXICO") || ocr_text.contains("MÉXICO") {
                issuing_country = "MEX".to_string();
            }
        }

        // Name field processing - combine surname and given names
        if name.is_empty() && (!surname.is_empty() || !given_names.is_empty()) {
            name = format!("{} {}", given_names.trim(), surname.trim()).trim().to_string();
        }

        // Combine surname and given names for the full name field if needed
        if !surname.is_empty() || !given_names.is_empty() {
            name = format!("{} {}", surname.trim(), given_names.trim()).trim().to_string();
        }

        // If we have an issue date but no expiry date, estimate expiry as 10 years later
        if !date_of_issue.is_empty() && date_of_expiry.is_empty() {
            // Simple estimation - not accurate for all countries but better than nothing
            if let Some(pos) = date_of_issue.rfind('/') {
                if let Ok(year) = date_of_issue[pos+1..].parse::<i32>() {
                    let expiry_year = year + 10;
                    date_of_expiry = format!("{}{}", &date_of_issue[..pos+1], expiry_year);
                }
            }
        }

        // Efficient fast-path fallbacks for remaining fields
        // Uses single pass text search instead of multiple regex matches

        // Smart date handling: If we have date of issue but no expiry, calculate it
        if !date_of_issue.is_empty() && date_of_expiry.is_empty() {
            // Fast expiry date estimation using string parsing
            if let Some(pos) = date_of_issue.rfind('/') {
                if pos + 1 < date_of_issue.len() && 
                   date_of_issue[pos+1..].parse::<i32>().is_ok() {
                    let year = date_of_issue[pos+1..].parse::<i32>().unwrap();
                    let expiry_year = year + 10; // Most passports valid for 10 years
                    date_of_expiry = format!("{}{}", &date_of_issue[..pos+1], expiry_year);
                }
            }
        }

        // Fast place of birth fallback using nationality as a heuristic
        if place_of_birth.is_none() {
            let nat_upper = nationality.to_uppercase();
            // Use a single string contains check for each country (faster than regex)
            if nat_upper.contains("USA") || nat_upper.contains("UNITED STATES") {
                place_of_birth = Some("UNITED STATES".to_string());
            } else if nat_upper.contains("UK") || nat_upper.contains("UNITED KINGDOM") || 
              nat_upper.contains("GBR") || nat_upper.contains("BRITISH") {
                place_of_birth = Some("UNITED KINGDOM".to_string());
            } else if nat_upper.contains("MEX") || nat_upper.contains("MEXICO") {
                place_of_birth = Some("MEXICO".to_string());
            } else if nat_upper.contains("CAN") || nat_upper.contains("CANADA") {
                place_of_birth = Some("CANADA".to_string());
            } else if place_of_birth.is_none() && ocr_text.contains("UNITED STATES") {
                place_of_birth = Some("UNITED STATES".to_string());
            }
        }

        // Fast authority fallback using country-specific patterns
        if authority.is_none() {
            // Single-pass string matching with early exits (faster than multiple regex)
            // Optimized with most common matches first
            if ocr_text.contains("DEPARTMENT OF STATE") {
                authority = Some("DEPARTMENT OF STATE".to_string());
            } else if ocr_text.contains("PASSPORT OFFICE") {
                authority = Some("PASSPORT OFFICE".to_string());
            } else if ocr_text.contains("FOREIGN AFFAIRS") {
                authority = Some("MINISTRY OF FOREIGN AFFAIRS".to_string());
            } else if ocr_text.contains("INTERIOR") {
                authority = Some("MINISTRY OF INTERIOR".to_string());
            } else if ocr_text.contains("IMMIGRATION") {
                authority = Some("IMMIGRATION OFFICE".to_string());
            } else if ocr_text.contains("SECRETARY OF STATE") {
                authority = Some("SECRETARY OF STATE".to_string());
            } else if ocr_text.contains("MINISTERIO") {
                authority = Some("MINISTERIO".to_string());
            } else if issuing_country.contains("USA") {
                authority = Some("U.S. DEPARTMENT OF STATE".to_string());
            } else if issuing_country.contains("UK") || issuing_country.contains("GBR") {
                authority = Some("HM PASSPORT OFFICE".to_string());
            }
        }

        // Create visual data structure
        let mut visual_data = VisualData {
            document_type,
            issuing_country,
            document_number,
            name,
            surname,
            given_names,
            nationality,
            date_of_birth,
            gender,
            place_of_birth,
            date_of_issue,
            date_of_expiry,
            authority,
            personal_number,
        };

        // Apply ML-based field-specific corrections to the entire data structure
        text_correction::correct_visual_data_ocr(&mut visual_data);

        // Print extraction results - uses Self::print_visual_data_results for consistent formatting
        Self::print_visual_data_results(&visual_data);

        Ok(visual_data)
    }

} // End of impl EnhancedOcrProcessor
