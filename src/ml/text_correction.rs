// Advanced text correction module with ML-inspired techniques
// This improves OCR accuracy with field-specific context awareness

use std::collections::HashMap;
use lazy_static::lazy_static;

/// Field types for context-aware correction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    DocumentNumber,
    Name,
    Date,
    Gender,
    Nationality,
    Address,
    Generic,
}

// Character confusion probabilities organized by source and target characters
// These values are based on common OCR errors in passport processing
lazy_static! {
    pub static ref CHAR_CONFUSION_MATRIX: HashMap<char, Vec<(char, f64)>> = {
        let mut m = HashMap::new();
        // Common digit/letter confusions
        m.insert('0', vec![('O', 0.95), ('D', 0.45), ('Q', 0.40)]);
        m.insert('1', vec![('I', 0.97), ('l', 0.95), ('|', 0.90), ('!', 0.80)]);
        m.insert('2', vec![('Z', 0.60), ('z', 0.55)]);
        m.insert('5', vec![('S', 0.70), ('s', 0.65)]);
        m.insert('6', vec![('G', 0.60), ('b', 0.55)]);
        m.insert('8', vec![('B', 0.75), ('S', 0.45)]);
        m.insert('9', vec![('g', 0.55), ('q', 0.40)]);
        
        // Letter confusions
        m.insert('B', vec![('8', 0.75), ('R', 0.50)]);
        m.insert('D', vec![('0', 0.45), ('O', 0.40)]);
        m.insert('G', vec![('C', 0.65), ('6', 0.60)]);
        m.insert('I', vec![('1', 0.97), ('l', 0.95), ('|', 0.90)]);
        m.insert('O', vec![('0', 0.95), ('Q', 0.75), ('D', 0.45)]);
        m.insert('Q', vec![('O', 0.75), ('0', 0.40)]);
        m.insert('S', vec![('5', 0.70), ('8', 0.45)]);
        m.insert('Z', vec![('2', 0.60)]);
        
        // Lowercase confusions
        m.insert('a', vec![('o', 0.60), ('e', 0.40)]);
        m.insert('c', vec![('e', 0.70), ('o', 0.60)]);
        m.insert('e', vec![('c', 0.70), ('o', 0.50)]);
        m.insert('g', vec![('q', 0.70), ('9', 0.55)]);
        m.insert('i', vec![('j', 0.70), ('l', 0.90)]);
        m.insert('l', vec![('1', 0.95), ('I', 0.95), ('|', 0.90)]);
        // Special case for multi-char confusion - handled separately
        m.insert('m', vec![('n', 0.75)]);
        m.insert('n', vec![('m', 0.60), ('h', 0.50)]);
        m.insert('o', vec![('a', 0.60), ('e', 0.50)]);
        m.insert('q', vec![('g', 0.70), ('9', 0.40)]);
        m.insert('r', vec![('n', 0.60)]);
        m.insert('u', vec![('v', 0.70), ('n', 0.55)]);
        m.insert('v', vec![('u', 0.70), ('y', 0.50)]);
        // Special case for multi-char confusion - handled separately
        m.insert('w', vec![('v', 0.70)]);
        m.insert('y', vec![('v', 0.50)]);
        m.insert('z', vec![('2', 0.55)]);
        
        // Special character confusions
        m.insert('.', vec![(',', 0.80), (':', 0.60)]);
        m.insert(',', vec![('.', 0.80), (';', 0.60)]);
        m.insert('(', vec![('C', 0.50), ('c', 0.45)]);
        m.insert(')', vec![('D', 0.50), ('0', 0.40)]);
        m.insert('-', vec![('_', 0.90), ('—', 0.95)]);
        m.insert(' ', vec![('_', 0.60), ('-', 0.50)]);
        
        m
    };
    
    // Field-specific character preferences
    pub static ref FIELD_CHAR_PREFERENCES: HashMap<FieldType, HashMap<char, Vec<char>>> = {
        let mut prefs = HashMap::new();
        
        // Document number preferences
        let mut doc_prefs = HashMap::new();
        doc_prefs.insert('O', vec!['0']);  // Prefer '0' over 'O' in document numbers
        doc_prefs.insert('I', vec!['1']);  // Prefer '1' over 'I' in document numbers
        doc_prefs.insert('l', vec!['1']);  // Prefer '1' over 'l' in document numbers
        doc_prefs.insert('B', vec!['8']);  // Prefer '8' over 'B' in document numbers
        doc_prefs.insert('S', vec!['5']);  // Prefer '5' over 'S' in document numbers
        doc_prefs.insert('Z', vec!['2']);  // Prefer '2' over 'Z' in document numbers
        prefs.insert(FieldType::DocumentNumber, doc_prefs);
        
        // Name preferences
        let mut name_prefs = HashMap::new();
        name_prefs.insert('0', vec!['O']);  // Prefer 'O' over '0' in names
        name_prefs.insert('1', vec!['I', 'l']);  // Prefer 'I' or 'l' over '1' in names
        name_prefs.insert('5', vec!['S']);  // Prefer 'S' over '5' in names
        name_prefs.insert('8', vec!['B']);  // Prefer 'B' over '8' in names
        prefs.insert(FieldType::Name, name_prefs);
        
        // Date preferences
        let mut date_prefs = HashMap::new();
        date_prefs.insert('O', vec!['0']);  // Prefer '0' over 'O' in dates
        date_prefs.insert('I', vec!['1']);  // Prefer '1' over 'I' in dates
        date_prefs.insert('l', vec!['1']);  // Prefer '1' over 'l' in dates
        date_prefs.insert('S', vec!['5']);  // Prefer '5' over 'S' in dates
        date_prefs.insert('B', vec!['8']);  // Prefer '8' over 'B' in dates
        date_prefs.insert('Z', vec!['2']);  // Prefer '2' over 'Z' in dates
        date_prefs.insert('|', vec!['/']);  // Prefer '/' over '|' in dates
        prefs.insert(FieldType::Date, date_prefs);
        
        prefs
    };
}

/// Advanced text cleaning with field-specific context
pub fn correct_text_with_context(text: &str, field_type: FieldType) -> String {
    let mut corrected = String::with_capacity(text.len());
    
    // Get field-specific preferences, or fall back to generic preferences
    let default_map = HashMap::new();
    let preferences = FIELD_CHAR_PREFERENCES.get(&field_type)
        .unwrap_or(&FIELD_CHAR_PREFERENCES.get(&FieldType::Generic).unwrap_or(&default_map));
    
    // Apply field-specific corrections
    for c in text.chars() {
        // Check if we have a preferred character for this field type
        if let Some(preferred_chars) = preferences.get(&c) {
            if !preferred_chars.is_empty() {
                corrected.push(preferred_chars[0]);
                continue;
            }
        }
        
        // Otherwise keep the original character
        corrected.push(c);
    }
    
    // Apply document type specific post-processing
    match field_type {
        FieldType::DocumentNumber => {
            // Remove unwanted characters from document numbers
            corrected = corrected.chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '<')
                .collect();
        },
        FieldType::Name => {
            // Names should be uppercase
            corrected = corrected.to_uppercase();
            // Remove common OCR errors in names
            corrected = corrected.replace("0EL", "DEL").replace("0E", "DE");
        },
        FieldType::Date => {
            // Clean up dates - remove non-date characters
            corrected = corrected.chars()
                .filter(|c| c.is_ascii_digit() || *c == '/' || *c == '-' || *c == '.')
                .collect();
        },
        _ => {}
    }
    
    corrected
}

/// Corrects a complete VisualData structure using field-specific context
pub fn correct_visual_data_ocr(visual_data: &mut crate::models::VisualData) {
    // Apply field-specific corrections
    visual_data.document_number = correct_text_with_context(&visual_data.document_number, FieldType::DocumentNumber);
    visual_data.surname = correct_text_with_context(&visual_data.surname, FieldType::Name);
    visual_data.given_names = correct_text_with_context(&visual_data.given_names, FieldType::Name);
    visual_data.date_of_birth = correct_text_with_context(&visual_data.date_of_birth, FieldType::Date);
    visual_data.date_of_issue = correct_text_with_context(&visual_data.date_of_issue, FieldType::Date);
    visual_data.date_of_expiry = correct_text_with_context(&visual_data.date_of_expiry, FieldType::Date);
    
    // Recombine name fields if necessary
    if !visual_data.surname.is_empty() && !visual_data.given_names.is_empty() {
        visual_data.name = format!("{} {}", visual_data.surname, visual_data.given_names);
    } else if !visual_data.name.is_empty() {
        // Attempt to improve the full name field
        visual_data.name = correct_text_with_context(&visual_data.name, FieldType::Name);
    }
}

/// Applies advanced text cleaning to raw OCR text
pub fn correct_ocr_raw_text(text: &str) -> String {
    let mut corrected = text.to_string();
    
    // Fix common OCR errors in raw text
    corrected = corrected.replace("l<", "I<")
                         .replace("l1", "II")
                         .replace("rn", "m")
                         .replace("vv", "w")
                         .replace("cl", "d")
                         .replace("0O", "00")
                         .replace("O0", "00")
                         .replace("I1", "11")
                         .replace("l1", "11");
    
    // Remove nonstandard spaces and control characters
    corrected = corrected.chars()
        .filter(|&c| c >= ' ' || c == '\n' || c == '\t')
        .collect();
    
    // Fix multiple spaces
    while corrected.contains("  ") {
        corrected = corrected.replace("  ", " ");
    }
    
    corrected
}

/// Advanced context-aware correction using neighboring characters and patterns
pub fn correct_with_ngram_context(text: &str, field_type: FieldType) -> String {
    let mut corrected = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    
    for i in 0..chars.len() {
        let curr_char = chars[i];
        
        // Skip certain processing based on field type
        match field_type {
            FieldType::DocumentNumber => {
                // For document numbers, apply stricter corrections
                if curr_char == 'O' || curr_char == 'o' {
                    corrected.push('0');
                } else if curr_char == 'I' || curr_char == 'l' || curr_char == '|' {
                    // Check if there are other digits nearby, suggesting this is a number
                    let has_digits_nearby = (i > 0 && chars[i-1].is_ascii_digit()) || 
                                           (i < chars.len() - 1 && chars[i+1].is_ascii_digit());
                    if has_digits_nearby {
                        corrected.push('1');
                    } else {
                        corrected.push(curr_char);
                    }
                } else {
                    corrected.push(curr_char);
                }
            },
            FieldType::Name => {
                // For names, preserve letter forms but fix common OCR errors
                if curr_char == '0' && (i > 0 && chars[i-1] == 'D' || i < chars.len() - 1 && chars[i+1] == 'E') {
                    corrected.push('O'); // Likely "DOE" not "D0E"
                } else if curr_char == '1' {
                    // Check context for letter 'I' vs number '1'
                    let is_in_word = (i > 0 && chars[i-1].is_ascii_alphabetic()) || 
                                   (i < chars.len() - 1 && chars[i+1].is_ascii_alphabetic());
                    if is_in_word {
                        corrected.push('I');
                    } else {
                        corrected.push(curr_char);
                    }
                } else {
                    corrected.push(curr_char);
                }
            },
            _ => corrected.push(curr_char),
        }
    }
    
    corrected
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_document_number_correction() {
        let doc_num = "A01234S6789O";
        let corrected = correct_text_with_context(doc_num, FieldType::DocumentNumber);
        assert_eq!(corrected, "A012345678900");
    }
    
    #[test]
    fn test_name_correction() {
        let name = "SM1TH J0HN";
        let corrected = correct_text_with_context(name, FieldType::Name);
        assert_eq!(corrected, "SMITH JOHN");
    }
    
    #[test]
    fn test_date_correction() {
        let date = "O1/I2/2O22";
        let corrected = correct_text_with_context(date, FieldType::Date);
        assert_eq!(corrected, "01/12/2022");
    }
}
