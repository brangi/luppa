// Field-specific extraction functions for EnhancedOcrProcessor
use super::EnhancedOcrProcessor;
use crate::processing::enhanced_ocr::{
    DOCUMENT_NUMBER_PATTERNS, NAME_PATTERNS, GIVEN_NAME_PATTERNS, DOB_PATTERNS, DOI_PATTERNS, DOE_PATTERNS, DATE_PATTERNS, GENDER_PATTERNS, POB_PATTERNS, NATIONALITY_PATTERNS, AUTHORITY_PATTERNS
};


impl EnhancedOcrProcessor {
    /// Universal document number extraction with language-agnostic patterns
    pub fn extract_document_number_from_text(text: &str) -> Option<String> {
        for pattern in DOCUMENT_NUMBER_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim().to_uppercase();
                    // Validate: most document numbers are 5-15 alphanumeric characters
                    if value.len() >= 5 && value.len() <= 15 && value.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-') {
                        return Some(value.replace(" ", ""));
                    }
                }
            }
        }
        None
    }
    /// Universal surname extraction with language-agnostic patterns
    pub fn extract_surname_from_text(text: &str) -> Option<String> {
        for pattern in NAME_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim().to_uppercase();
                    // Basic validation: names should be alpha characters
                    if value.len() >= 2 && !Self::is_field_label(&value) {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
    /// Universal given names extraction with language-agnostic patterns
    pub fn extract_given_names_from_text(text: &str) -> Option<String> {
        for pattern in GIVEN_NAME_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim().to_uppercase();
                    // Basic validation: names should be alpha characters
                    if value.len() >= 2 && !Self::is_field_label(&value) {
                        return Some(value);
                    }
                }
            }
        }
        None
    }
    /// Extract date of birth with support for multiple formats
    pub fn extract_dob_from_text(text: &str) -> Option<String> {
        // First try specific DOB patterns
        for pattern in DOB_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    return Some(crate::processing::enhanced_ocr::normalize_date(matched.as_str()));
                }
            }
        }
        // Fallback to looking for generic dates near birth-related keywords
        if text.to_lowercase().contains("birth") || 
           text.to_lowercase().contains("born") || 
           text.to_lowercase().contains("naissance") || 
           text.to_lowercase().contains("nacido") || 
           text.to_lowercase().contains("geburt") {
            return Self::extract_date_from_text(text);
        }
        None
    }
    /// Extract date of issue with support for multiple formats
    pub fn extract_doi_from_text(text: &str) -> Option<String> {
        // First try specific DOI patterns
        for pattern in DOI_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    return Some(crate::processing::enhanced_ocr::normalize_date(matched.as_str()));
                }
            }
        }
        // Fallback to looking for generic dates near issue-related keywords
        if text.to_lowercase().contains("issue") || 
           text.to_lowercase().contains("issued") || 
           text.to_lowercase().contains("emission") || 
           text.to_lowercase().contains("émis") || 
           text.to_lowercase().contains("expedido") || 
           text.to_lowercase().contains("ausgestellt") {
            return Self::extract_date_from_text(text);
        }
        None
    }
    /// Extract date of expiry with support for multiple formats
    pub fn extract_doe_from_text(text: &str) -> Option<String> {
        // First try specific DOE patterns
        for pattern in DOE_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    return Some(crate::processing::enhanced_ocr::normalize_date(matched.as_str()));
                }
            }
        }
        // Fallback to looking for generic dates near expiry-related keywords
        if text.to_lowercase().contains("expiry") || 
           text.to_lowercase().contains("expiration") || 
           text.to_lowercase().contains("valid until") || 
           text.to_lowercase().contains("valable") || 
           text.to_lowercase().contains("válido") || 
           text.to_lowercase().contains("gültig") {
            return Self::extract_date_from_text(text);
        }
        None
    }
    /// Generic date extraction from text with multiple format support
    pub fn extract_date_from_text(text: &str) -> Option<String> {
        for pattern in DATE_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                // Determine date format based on the pattern and normalize
                return Some(crate::processing::enhanced_ocr::normalize_date(captures.get(0).unwrap().as_str()));
            }
        }
        None
    }
    /// Extract gender field with multilingual support
    pub fn extract_gender_from_text(text: &str) -> Option<String> {
        for pattern in GENDER_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim().to_uppercase();
                    // Normalize gender values
                    return match value.chars().next().unwrap_or('X') {
                        'M' | 'H' => Some("M".to_string()),
                        'F' | 'W' => Some("F".to_string()),
                        _ => {
                            // Handle spelled out versions
                            let lower_value = value.to_lowercase();
                            if lower_value.contains("male") || 
                               lower_value.contains("masculin") || 
                               lower_value.contains("männlich") || 
                               lower_value.contains("hombre") ||
                               lower_value.contains("homme") ||
                               lower_value.contains("mann") {
                                Some("M".to_string())
                            } else if lower_value.contains("female") || 
                                      lower_value.contains("feminin") || 
                                      lower_value.contains("weiblich") || 
                                      lower_value.contains("mujer") ||
                                      lower_value.contains("femme") ||
                                      lower_value.contains("frau") {
                                Some("F".to_string())
                            } else {
                                None
                            }
                        }
                    }
                }
            }
        }
        None
    }
    /// Extract place of birth with confidence-based scoring and enhanced multilingual support
    pub fn extract_place_of_birth_from_text(text: &str) -> Option<String> {
        // Keywords for place of birth in different languages and formats
        let pob_keywords = [
            "PLACE OF BIRTH", "BIRTH PLACE", "BIRTHPLACE", "BORN AT", "BORN IN",
            "LUGAR DE NACIMIENTO", "NACIDO EN", "CIUDAD DE NACIMIENTO", 
            "LIEU DE NAISSANCE", "NÉ À", "NÉE À", "VILLE DE NAISSANCE",
            "GEBURTSORT", "GEBOREN IN", "GEBURTSTADT",
            "LUOGO DI NASCITA", "NATO A", "NATA A",
            "LOCAL DE NASCIMENTO", "NATURALIDADE",
            "POB", "P.O.B", "LDN", "L.D.N"
        ];
        let geo_patterns = [
            "CITY", "VILLE", "STADT", "CIUDAD", "PROVINCE", "STATE", "COUNTY"
        ];
        for pattern in POB_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim();
                    if !value.is_empty() && !crate::processing::enhanced_ocr::is_field_label(value) && !crate::processing::enhanced_ocr::is_non_place_word(value) {
                        return Some(value.to_string());
                    }
                }
            }
        }
        let mut candidates = Vec::new();
        let mut best_confidence = 0.0;
        let mut best_candidate = None;
        let lines: Vec<&str> = text.split('\n').collect();
        for line in &lines {
            let line_upper = line.to_uppercase();
            for &keyword in &pob_keywords {
                if let Some(pos) = line_upper.find(keyword) {
                    if pos + keyword.len() < line_upper.len() {
                        let after_keyword = &line_upper[pos + keyword.len()..];
                        let cleaned = after_keyword.trim_start_matches(|c: char| c == ':' || c == ',' || c == '-' || c.is_whitespace());
                        let place = if let Some(end) = cleaned.find(|c| c == ',' || c == ';' || c == '.' || c == '\n') {
                            cleaned[..end].trim().to_string()
                        } else {
                            cleaned.trim().to_string()
                        };
                        let confidence = if keyword.len() > 10 { 0.8 } else { 0.6 };
                        let geo_confidence = geo_patterns.iter().any(|&pattern| place.contains(pattern)) as u8 as f64 * 0.2;
                        let total_confidence = confidence + geo_confidence;
                        if place.len() > 2 && !crate::processing::enhanced_ocr::is_non_place_word(&place) {
                            candidates.push((place.clone(), total_confidence));
                            if total_confidence > best_confidence {
                                best_confidence = total_confidence;
                                best_candidate = Some(place);
                            }
                        }
                    }
                }
            }
        }
        if candidates.is_empty() {
            for line in &lines {
                let line_upper = line.to_uppercase();
                if line_upper.contains("/") || line_upper.chars().filter(|c| c.is_ascii_digit()).count() > 5 {
                    continue;
                }
                for &pattern in &geo_patterns {
                    if let Some(pos) = line_upper.find(pattern) {
                        let start = if pos > 10 { pos - 10 } else { 0 };
                        let end = std::cmp::min(pos + pattern.len() + 10, line_upper.len());
                        let context = line_upper[start..end].trim().to_string();
                        if context.len() > 2 && !crate::processing::enhanced_ocr::is_non_place_word(&context) {
                            candidates.push((context.clone(), 0.5));
                            if 0.5 > best_confidence {
                                best_confidence = 0.5;
                                best_candidate = Some(context);
                            }
                        }
                    }
                }
            }
        }
        if let Some(candidate) = best_candidate {
            return Some(candidate);
        }
        None
    }
    /// Extract nationality with multilingual support
    pub fn extract_nationality_from_text(text: &str) -> Option<String> {
        for pattern in NATIONALITY_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim();
                    if !value.is_empty() && !Self::is_field_label(value) {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }
    /// Extract authority/issuing agency with enhanced multilingual support and positional heuristics
    pub fn extract_authority_from_text(text: &str) -> Option<String> {
        let known_authorities = [
            "DEPARTMENT OF STATE", "SECRETARY OF STATE", "U.S. DEPARTMENT OF STATE", "UNITED STATES",
            "HM PASSPORT OFFICE", "HOME OFFICE", "UKPA", "UK PASSPORT AGENCY", "IDENTITY & PASSPORT SERVICE",
            "PASSPORT CANADA", "IMMIGRATION CANADA", "CITIZENSHIP AND IMMIGRATION CANADA", 
            "IMMIGRATION, REFUGEES AND CITIZENSHIP CANADA",
            "BUNDESREPUBLIK DEUTSCHLAND", "BUNDESMINISTERIUM", "AUSWÄRTIGES AMT",
            "RÉPUBLIQUE FRANÇAISE", "MINISTÈRE DE L'EUROPE ET DES AFFAIRES ÉTRANGÈRES",
            "REINO DE ESPAÑA", "MINISTERIO DE ASUNTOS EXTERIORES",
            "REPUBBLICA ITALIANA", "MINISTERO DEGLI AFFARI ESTERI",
            "KINGDOM OF THE NETHERLANDS", "KONINKRIJK DER NEDERLANDEN",
            "МИНИСТЕРСТВО ИНОСТРАННЫХ ДЕЛ", "МИД РОССИИ", "РОССИЙСКАЯ ФЕДЕРАЦИЯ",
            "MINISTRY", "MINISTER", "MINISTÈRE", "MINISTERIO", "MINISTERO", "MINISTERIUM",
            "HOME OFFICE", "IMMIGRATION", "BORDER", "PASSPORT OFFICE", "PASSPORT AGENCY",
            "FOREIGN AFFAIRS", "INTERIOR", "AFFAIRES ÉTRANGÈRES", "ASUNTOS EXTERIORES",
            "PASSEPORT", "REISEPASS", "PASAPORTE", "PASSAPORTO", "REPÚBLICA", "REPUBLIC OF", 
            "KINGDOM OF", "FEDERAL", "NATIONAL", "INTERNATIONAL"
        ];
        for pattern in AUTHORITY_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(text) {
                if let Some(matched) = captures.get(1) {
                    let value = matched.as_str().trim();
                    if !value.is_empty() && !crate::processing::enhanced_ocr::is_field_label(value) {
                        let cleaned_value = value
                            .replace("0", "O")
                            .replace("1", "I")
                            .replace("-\n", "")
                            .replace("\n", " ")
                            .trim()
                            .to_string();
                        for &known in &known_authorities {
                            if cleaned_value.to_uppercase().contains(known) {
                                return Some(known.to_string());
                            }
                        }
                        return Some(cleaned_value);
                    }
                }
            }
        }
        let lines: Vec<&str> = text.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            let line_upper = line.to_uppercase();
            if line_upper.contains("AUTHORITY") || line_upper.contains("ISSUED BY") ||
               line_upper.contains("AUTORITÉ") || line_upper.contains("AUTORIDAD") ||
               line_upper.contains("BEHÖRDE") || line_upper.contains("AUTORITÀ") ||
               line_upper.contains("ВЫДАН") || line_upper.contains("AUTORITEIT") ||
               line_upper.contains("UTFÄRDAT AV") || line_upper.contains("UDSTEDT AF") {
                if let Some(pos) = line_upper.find(|c| c == ':' || c == '-' || c == '—' || c == '–') {
                    let after_delimiter = line[pos+1..].trim();
                    if !after_delimiter.is_empty() && after_delimiter.len() < 60 {
                        return Some(after_delimiter.to_string());
                    }
                }
                for offset in 1..=2 {
                    if i + offset < lines.len() {
                        let next_line = lines[i + offset].trim();
                        if !next_line.is_empty() && next_line.len() < 60 && !crate::processing::enhanced_ocr::is_field_label(next_line) {
                            return Some(next_line.to_string());
                        }
                    }
                }
            }
        }
        None
    }
}
