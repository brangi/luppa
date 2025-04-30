use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use tesseract::Tesseract;
use crate::utils::PassportError;
use crate::models::{MrzData, CheckDigits, VisualData, DocumentFormat};
use whatlang::{detect, Lang};
use regex::Regex;

pub struct OcrProcessor;

impl OcrProcessor {
    /// Detect tessdata directory and return as Option
    pub fn tessdata_prefix() -> Option<String> {
        // Check Brew installation on macOS
        if cfg!(target_os = "macos") {
            if let Ok(output) = Command::new("brew").arg("--prefix").output() {
                if let Ok(prefix) = String::from_utf8(output.stdout) {
                    let tessdata_dir = Path::new(prefix.trim()).join("share/tessdata");
                    if tessdata_dir.join("ocrb.traineddata").exists() {
                        return Some(tessdata_dir.to_string_lossy().into_owned());
                    }
                }
            }
        }

        // Check TESSDATA_PREFIX environment variable
        if let Ok(prefix) = std::env::var("TESSDATA_PREFIX") {
            let td = Path::new(&prefix).join("tessdata");
            if td.join("ocrb.traineddata").exists() {
                return Some(prefix);
            }
        }

        // Check common tessdata directories
        let common_paths = [
            "/usr/share/tessdata",
            "/usr/local/share/tessdata",
            "/opt/homebrew/share/tessdata",
        ];

        for path in &common_paths {
            let path = Path::new(path);
            if path.join("ocrb.traineddata").exists() {
                return Some(path.to_string_lossy().into_owned());
            }
        }

        None
    }

    /// Clean alphanumeric MRZ field by correcting OCR confusions and removing fillers.
    fn clean_mrz_alphanumeric_field(field: &str) -> String {
        field.chars()
            .map(|c| {
                let c = c.to_ascii_uppercase();
                match c {
                    'I' | 'L' | '|' => '1',
                    'Z' => '2',
                    'S' => '5',
                    'G' => '6',
                    'B' => '8',
                    other => other,
                }
            })
            .filter(|c| c.is_ascii_alphanumeric())
            .collect()
    }

    // Get full OCR text from image
    pub fn get_full_ocr_text(image_data: &[u8]) -> Result<String, PassportError> {
        // Create a temporary file to write the image data to
        let mut temp_file = NamedTempFile::new().map_err(|e| PassportError::IoError(e.to_string()))?;
        temp_file.write_all(image_data).map_err(|e| PassportError::IoError(e.to_string()))?;
        
        // Get the path as a string
        let image_path_str = temp_file.path().to_str()
            .ok_or_else(|| PassportError::IoError("Failed to convert path to string".to_string()))?;
        
        let datapath_prefix = Self::tessdata_prefix();
        if let Some(ref path) = datapath_prefix {
            std::env::set_var("TESSDATA_PREFIX", path);
            println!("[DEBUG] Set TESSDATA_PREFIX to {}", path);
        }
        let datapath_opt = datapath_prefix.as_deref();
        // Try OCR-B model, fallback to multilingual then English
        let tess = match Tesseract::new(datapath_opt, Some("ocrb+eng+spa+fra+deu")) {
            Ok(api) => api,
            Err(e) => {
                println!("[DEBUG] Tesseract error for ocrb+eng+spa+fra+deu: {}", e);
                println!("Warning: OCR-B model not available ({})", e);
                println!("Falling back to default multilingual model");
                match Tesseract::new(datapath_opt, Some("eng+spa+fra+deu")) {
                    Ok(api2) => api2,
                    Err(e2) => {
                        println!("[DEBUG] Tesseract error for eng+spa+fra+deu: {}", e2);
                        println!("Warning: multilingual model not available ({})", e2);
                        println!("Falling back to English only model");
                        Tesseract::new(datapath_opt, Some("eng"))
                            .map_err(|e3| PassportError::MrzExtractionError(format!("Tesseract init error: {}", e3)))?
                    }
                }
            }
        };
        let text = tess.set_image(image_path_str)
            .map_err(|e| PassportError::MrzExtractionError(format!("Tesseract set image error: {}", e)))?
            .get_text()
            .map_err(|e| PassportError::MrzExtractionError(format!("Tesseract get text error: {}", e)))?;
        
        Ok(text)
    }
    // Extract MRZ data from the processed image
    pub fn extract_mrz(image_data: &[u8]) -> Result<MrzData, PassportError> {
        println!("Extracting MRZ data from image...");
        
        // Create a temporary file from the image data
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to create temp file: {}", e)))?;
            
        temp_file.write_all(image_data)
            .map_err(|e| PassportError::MrzExtractionError(format!("Failed to write to temp file: {}", e)))?;
            
        let image_path_str = temp_file.path().to_str()
            .ok_or_else(|| PassportError::MrzExtractionError("Failed to convert path to string".to_string()))?;
        
        let datapath_prefix = Self::tessdata_prefix();
        if let Some(ref path) = datapath_prefix {
            std::env::set_var("TESSDATA_PREFIX", path);
            println!("[DEBUG] Set TESSDATA_PREFIX to {}", path);
        }
        let datapath_opt = datapath_prefix.as_deref();
        // Try OCR-B for MRZ, fallback to multilingual then English
        let tess = match Tesseract::new(datapath_opt, Some("ocrb")) {
            Ok(api) => api,
            Err(e) => {
                println!("[DEBUG] Tesseract error for ocrb: {}", e);
                println!("Warning: OCR-B model not available for MRZ ({})", e);
                println!("Falling back to multilingual MRZ model");
                match Tesseract::new(datapath_opt, Some("eng+spa+fra+deu")) {
                    Ok(api2) => api2,
                    Err(e2) => {
                        println!("[DEBUG] Tesseract error for eng+spa+fra+deu: {}", e2);
                        println!("Warning: multilingual MRZ model not available ({})", e2);
                        println!("Falling back to English only model for MRZ");
                        Tesseract::new(datapath_opt, Some("eng"))
                            .map_err(|e3| PassportError::MrzExtractionError(format!("Tesseract init error: {}", e3)))?
                    }
                }
            }
        };
        let text = tess.set_image(image_path_str)
            .map_err(|e| PassportError::MrzExtractionError(format!("Tesseract set image error: {}", e)))?
            .set_variable("tessedit_char_whitelist", "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789<")
            .map_err(|e| PassportError::MrzExtractionError(format!("Tesseract set variable error: {}", e)))?
            .get_text()
            .map_err(|e| PassportError::MrzExtractionError(format!("Tesseract error: {}", e)))?;
            
        println!("MRZ OCR result:\n{}", text);
        
        // Collect and clean each OCR line
        let lines: Vec<&str> = text.lines().collect();
        let cleaned_candidates: Vec<String> = lines.iter()
            .map(|line| Self::clean_mrz_line(line))
            .collect();

        // Filter MRZ candidate lines by length (40 to 44 chars) and valid MRZ charset
        let mut candidates: Vec<String> = cleaned_candidates.iter().cloned()
            .filter(|l| (40..=44).contains(&l.len())
                         && l.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '<'))
            .collect();
        // Fallback: if filter yielded less than 2, try lines starting with 'P<' and next line
        if candidates.len() < 2 {
            if let Some(pos) = cleaned_candidates.iter().position(|l| l.starts_with("P<")) {
                if pos + 1 < cleaned_candidates.len() {
                    candidates = vec![
                        cleaned_candidates[pos].clone(),
                        cleaned_candidates[pos + 1].clone(),
                    ];
                }
            }
        }
        // Sort by descending length to prioritize full MRZ lines
        candidates.sort_by(|a, b| b.len().cmp(&a.len()));
        // Select the top two MRZ lines and pad/truncate to the expected length
        let raw_lines: Vec<String> = candidates.into_iter().take(2).collect();
        let expected_chars = DocumentFormat::TD3.mrz_chars_per_line();
        let mrz_lines: Vec<String> = raw_lines
            .iter()
            .map(|l| {
                let mut s = l.clone();
                if s.len() > expected_chars {
                    s.truncate(expected_chars);
                } else if s.len() < expected_chars {
                    s.extend(std::iter::repeat('<').take(expected_chars - s.len()));
                }
                s
            })
            .collect();
        println!("Filtered and padded MRZ lines: {:?}", mrz_lines);
        
        // If we found at least 2 valid MRZ lines, parse them
        if mrz_lines.len() >= 2 {
            // In TD3 format (passport), MRZ consists of two lines of 44 characters each
            let line1 = if mrz_lines[0].len() > 43 { &mrz_lines[0][0..44] } else { &mrz_lines[0] };
            let line2 = if mrz_lines[1].len() > 43 { &mrz_lines[1][0..44] } else { &mrz_lines[1] };
            
            // Clean and validate birth date
            let raw_birth_date = &line2[13..19];
            // Use specialized MRZ date parser for more tolerance
            let mrz_date_of_birth = match Self::parse_mrz_date(raw_birth_date) {
                Some(date) => {
                    // The date is in YYMMDD format, but we need to check if it's actually correct
                    // For birth dates in MRZ, we need to interpret the date correctly
                    let year_str = &date[0..2];
                    let month_str = &date[2..4];
                    let day_str = &date[4..6];
                    
                    let year = year_str.parse::<u32>().unwrap_or(0);
                    let month = month_str.parse::<u32>().unwrap_or(0);
                    let day = day_str.parse::<u32>().unwrap_or(0);
                    
                    // Determine full year (assuming 19xx for birth years, 20xx for recent years)
                    let full_year = if year < 30 { 2000 + year } else { 1900 + year };
                    
                    println!("Successfully parsed birth date: {:02}/{:02}/{:04}", day, month, full_year);
                    
                    // Format properly for display in verification report
                    let formatted_date = format!("{:02} {:02} {:04}", day, month, full_year);
                    formatted_date
                },
                None => return Err(PassportError::MrzExtractionError("Invalid birth date after parsing".to_string())),
            };
            
            // Store the formatted date
            let date_of_birth = mrz_date_of_birth;
            
            // Clean and validate expiry date
            let raw_expiry_date = &line2[21..27];
            // Use specialized MRZ date parser for more tolerance
            let mrz_date_of_expiry = match Self::parse_mrz_date(raw_expiry_date) {
                Some(date) => {
                    // For expiry dates in MRZ, we need to interpret the date correctly
                    let year_str = &date[0..2];
                    let month_str = &date[2..4];
                    let day_str = &date[4..6];
                    
                    let year = year_str.parse::<u32>().unwrap_or(0);
                    let month = month_str.parse::<u32>().unwrap_or(0);
                    let day = day_str.parse::<u32>().unwrap_or(0);
                    
                    // For expiry dates, we always assume 20xx
                    let full_year = 2000 + year;
                    
                    println!("Successfully parsed expiry date: {:02}/{:02}/{:04}", day, month, full_year);
                    
                    // Format properly for display in verification report
                    let formatted_date = format!("{:02} {:02} {:04}", day, month, full_year);
                    formatted_date
                },
                None => return Err(PassportError::MrzExtractionError("Invalid expiry date after parsing".to_string())),
            };
            
            // Store the formatted date
            let date_of_expiry = mrz_date_of_expiry;
            
            // Parse MRZ data following the ICAO 9303 standard for TD3 documents
            // Line 1: Positions 1-2 (Document type), 3-5 (Issuing country), 6-44 (Name)
            let document_type = if line1.len() > 1 { line1[0..1].to_string() } else { "P".to_string() };
            let raw_issuing_country = if line1.len() > 5 { &line1[2..5] } else { "MEX" };
            let issuing_country = Self::clean_mrz_alpha_field(raw_issuing_country);
            
            // Extract and clean name parts
            let name_part = if line1.len() > 6 { &line1[5..] } else { "" };
            let name_parts: Vec<&str> = name_part.split("<<").collect();
            let raw_surname = name_parts.get(0).unwrap_or(&"");
            let surname = if !raw_surname.is_empty() {
                let cleaned = Self::clean_mrz_alpha_field(raw_surname);
                cleaned.replace("<", " ").trim().to_string()
            } else {
                "UNKNOWN".to_string()
            };
            let raw_given = name_parts.get(1).unwrap_or(&"");
            let given_names = if !raw_given.is_empty() {
                let cleaned = Self::clean_mrz_alpha_field(raw_given);
                cleaned.replace("<", " ").trim().to_string()
            } else {
                "UNKNOWN".to_string()
            };
            
            // Line 2: Positions 1-9 (Document number), 10 (Check digit), 11-13 (Nationality), 
            // 14-19 (Birth date), 20 (Check digit), 21 (Sex), 22-27 (Expiry date), 
            // 28 (Check digit), 29-42 (Personal number), 43 (Check digit), 44 (Composite check digit)
            let raw_doc = if line2.len() > 9 { &line2[0..9] } else { "" };
            let document_number = if !raw_doc.is_empty() {
                let s = Self::clean_mrz_alphanumeric_field(raw_doc);
                if s.is_empty() { "UNKNOWN".to_string() } else { s }
            } else {
                "UNKNOWN".to_string()
            };
            let raw_nationality = if line2.len() > 12 { &line2[9..12] } else { "MEX" };
            let nationality = Self::clean_mrz_alpha_field(raw_nationality);
            let gender = if line2.len() > 21 { line2[20..21].to_string() } else { "X".to_string() };
            // Extract and clean expiry date (positions 22-27 in MRZ)
            let date_of_expiry = date_of_expiry;
            
            // Extract check digits
            let doc_check = if line2.len() > 10 { line2.chars().nth(9).unwrap_or('0') } else { '0' };
            let dob_check = if line2.len() > 20 { line2.chars().nth(19).unwrap_or('0') } else { '0' };
            let exp_check = if line2.len() > 28 { line2.chars().nth(27).unwrap_or('0') } else { '0' };
            let pers_check = if line2.len() > 43 { line2.chars().nth(42).unwrap_or('0') } else { '0' };
            let comp_check = if line2.len() > 44 { line2.chars().nth(43).unwrap_or('0') } else { '0' };
            
            let mrz_data = MrzData {
                document_type,
                issuing_country,
                surname,
                given_names,
                document_number,
                nationality,
                date_of_birth,
                gender,
                date_of_expiry,
                personal_number: None,
                check_digits: CheckDigits {
                    document_number_check: doc_check,
                    date_of_birth_check: dob_check,
                    date_of_expiry_check: exp_check,
                    personal_number_check: pers_check,
                    composite_check: comp_check,
                },
                document_format: Some(DocumentFormat::TD3),
                optional_data: None,
                raw_mrz_lines: mrz_lines,
            };
            
            println!("Successfully extracted MRZ data");
            Ok(mrz_data)
        } else {
            return Err(PassportError::MrzExtractionError(
                "Failed to extract valid MRZ lines".to_string()
            ));
        }
    }

    /// Clean up an MRZ line by stripping whitespace and uppercasing
    fn clean_mrz_line(line: &str) -> String {
        let line = line.trim();
        let mut cleaned = String::with_capacity(line.len());
        for c in line.chars() {
            if c.is_whitespace() {
                continue;
            }
            cleaned.push(c.to_ascii_uppercase());
        }
 
        // Return cleaned MRZ line without altering character count
        cleaned
    }

    /// Clean alphabetic MRZ field by correcting OCR confusions between digits and letters.
    fn clean_mrz_alpha_field(field: &str) -> String {
        field.chars().map(|c| {
            let c = c.to_ascii_uppercase();
            match c {
                '0' => 'O',
                '1' => 'I',
                '2' => 'Z',
                '3' => 'E',
                '4' => 'A',
                '5' => 'S',
                '6' => 'G',
                '7' => 'T',
                '8' => 'B',
                other => other,
            }
        }).collect()
    }

    // Extract visual data using OCR
    pub fn extract_visual_data(image_data: &[u8]) -> Result<VisualData, PassportError> {
        println!("Extracting visual data from image...");
        
        // Extract MRZ data first to prepopulate core fields
        let mrz_data = Self::extract_mrz(image_data)?;
        let mut document_number = mrz_data.document_number.clone();
        let mut surname = mrz_data.surname.clone();
        let mut given_names = mrz_data.given_names.clone();
        let mut date_of_birth = mrz_data.date_of_birth.clone();
        let nationality = mrz_data.nationality.clone();
        let issuing_country = mrz_data.issuing_country.clone();
        let gender = mrz_data.gender.clone();
        let mut place_of_birth: Option<String> = None;
        let mut date_of_issue = String::new();
        let mut date_of_expiry = String::new();
        let mut authority: Option<String> = None;
        let personal_number = mrz_data.personal_number.clone();
        // Prepare regex patterns for extracting optional fields
        let re_place = Regex::new(r"(?i)place of birth[:\s]*(.+)").unwrap();
        let re_issue = Regex::new(r"(?i)date of issu(?:e)?[:\s]*(\d{2}[ ./\-]\d{2}[ ./\-]\d{4})").unwrap();
        let re_expiry = Regex::new(r"(?i)date of expi(?:ry)?[:\s]*(\d{2}[ ./\-]\d{2}[ ./\-]\d{4})").unwrap();
        let re_authority = Regex::new(r"(?i)authority[:\s]*(.+)").unwrap();
        let mut visual_document_number: Option<String> = None;
        let mut visual_name: Option<String> = None;
        let re_doc_number = Regex::new(r"(?i)document number[:\s]*([A-Z0-9<]+)").unwrap();
        let re_name = Regex::new(r"(?i)(surname|given names)[:\s]*(.+)").unwrap();
        let mut visual_date_of_birth: Option<String> = None;
        let re_birth = Regex::new(r"(?i)(date of birth|fecha de nacimiento)[:\s]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}|\d{6,8})").unwrap(); // Capture date pattern directly
        let mut visual_date_of_issue: Option<String> = None;
        let re_issue_date = Regex::new(r"(?i)(date of issue|fecha de expedicion)[:\s]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}|\d{6,8})").unwrap(); // Capture date pattern
        let mut visual_date_of_expiry: Option<String> = None;
        let re_expiry_date = Regex::new(r"(?i)(date of expiry|fecha de caducidad|date of expiration)[:\s]*(\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}|\d{6,8})").unwrap(); // Capture date pattern
        // Automatic language detection using full OCR text
        let initial_text = Self::get_full_ocr_text(image_data)?;
        let info = detect(&initial_text)
            .ok_or_else(|| PassportError::FormatError("Language detection failed".to_string()))?;
        let detected_lang_code = match info.lang() {
            Lang::Eng => "eng",
            Lang::Spa => "spa",
            Lang::Fra => "fra",
            Lang::Deu => "deu",
            _ => "eng",
        }.to_string();
        println!("Detected language: {}", detected_lang_code);
         
        // Use OCR-B with detected language for best passport font support
        let tess_langs = format!("ocrb+{}", detected_lang_code);
        println!("Using OCR model: {}", tess_langs);
        
        // Create a temporary file from the image data
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| PassportError::FormatError(format!("Failed to create temp file: {}", e)))?;
            
        temp_file.write_all(image_data)
            .map_err(|e| PassportError::FormatError(format!("Failed to write to temp file: {}", e)))?;
            
        let image_path_str = temp_file.path().to_str()
            .ok_or_else(|| PassportError::FormatError("Failed to convert path to string".to_string()))?;
        
        let datapath_prefix = Self::tessdata_prefix();
        if let Some(ref path) = datapath_prefix {
            std::env::set_var("TESSDATA_PREFIX", path);
            println!("[DEBUG] Set TESSDATA_PREFIX to {}", path);
        }
        let datapath_opt = datapath_prefix.as_deref();
        // Try OCR-B plus detected language, fallback to detected language then English
        let tess = match Tesseract::new(datapath_opt, Some(&tess_langs)) {
            Ok(api) => api,
            Err(e) => {
                println!("Warning: OCR-B model not available for visual OCR ({})", e);
                println!("Falling back to detected language only");
                match Tesseract::new(datapath_opt, Some(&detected_lang_code)) {
                    Ok(api2) => api2,
                    Err(e2) => {
                        println!("Warning: detected language model not available ({})", e2);
                        println!("Falling back to English only model for visual OCR");
                        Tesseract::new(datapath_opt, Some("eng"))
                            .map_err(|e3| PassportError::FormatError(format!("Tesseract init error: {}", e3)))?
                    }
                }
            }
        };
        let text = tess.set_image(image_path_str)
            .map_err(|e| PassportError::FormatError(format!("Tesseract set image error: {}", e)))?
            .get_text()
            .map_err(|e| PassportError::FormatError(format!("Tesseract error: {}", e)))?;
            
        println!("Visual OCR result:\n{}", text);
        
        // Extract optional visual fields from OCR text using regex
        for line in text.lines() {
            let line = line.trim();
            if place_of_birth.is_none() {
                if let Some(cap) = re_place.captures(line) {
                    place_of_birth = Some(cap[1].trim().to_string());
                }
            }
            if date_of_issue.is_empty() {
                if let Some(cap) = re_issue.captures(line) {
                    date_of_issue = cap[1].trim().to_string();
                }
            }
            if date_of_expiry.is_empty() {
                if let Some(cap) = re_expiry.captures(line) {
                    date_of_expiry = cap[1].trim().to_string();
                }
            }
            if authority.is_none() {
                if let Some(cap) = re_authority.captures(line) {
                    authority = Some(cap[1].trim().to_string());
                }
            }
            if visual_document_number.is_none() {
                if let Some(cap) = re_doc_number.captures(line) {
                    visual_document_number = Some(cap[1].trim().to_string());
                }
            }
            if visual_name.is_none() {
                if let Some(cap) = re_name.captures(line) {
                    visual_name = Some(cap[1].trim().to_string());
                }
            }
            if visual_date_of_birth.is_none() {
                if let Some(cap) = re_birth.captures(line) {
                    let date_str = cap.get(2).map_or("", |m| m.as_str()); // Get the captured date string group
                    visual_date_of_birth = Some(date_str.to_string());
                }
            }
            if visual_date_of_issue.is_none() {
                if let Some(cap) = re_issue_date.captures(line) {
                    let date_str = cap.get(2).map_or("", |m| m.as_str()); // Get the captured date string group
                    visual_date_of_issue = Some(date_str.to_string());
                }
            }
            if visual_date_of_expiry.is_none() {
                if let Some(cap) = re_expiry_date.captures(line) {
                    let date_str = cap.get(2).map_or("", |m| m.as_str()); // Get the captured date string group
                    visual_date_of_expiry = Some(date_str.to_string());
                }
            }
        }
        
        // Override MRZ data if visual extraction is valid
        if let Some(vis_doc) = &visual_document_number {
            if vis_doc.len() == document_number.len() && vis_doc.chars().all(|c| c.is_alphanumeric()) {
                document_number = vis_doc.clone();
            }
        }
        if let Some(vis_name) = &visual_name {
            if !vis_name.is_empty() {
                // Simple override for name; in a real scenario, consider parsing or confidence scoring
                let name_parts: Vec<&str> = vis_name.split_whitespace().collect();
                if name_parts.len() >= 2 {
                    surname = name_parts[0].to_string();
                    given_names = name_parts[1..].join(" ");
                } else if !name_parts.is_empty() {
                    surname = name_parts[0].to_string(); // Fallback if parsing fails
                }
            }
        }
        if date_of_birth == "INVALID" {
            if let Some(vis_birth) = &visual_date_of_birth {
                // Simple parsing: assume visual date is in DD/MM/YYYY or similar, convert to YYMMDD for consistency
                if let Some(parsed_date) = Self::parse_visual_date(vis_birth) {
                    date_of_birth = parsed_date; // Set to formatted YYMMDD string
                }
            }
        }
        if date_of_expiry == "INVALID" {
            if let Some(vis_expiry) = &visual_date_of_expiry { // Assuming visual_date_of_expiry is already extracted
                if let Some(parsed_date) = Self::parse_visual_date(vis_expiry) {
                    date_of_expiry = parsed_date;
                }
            }
        }
        if date_of_issue.is_empty() {
            if let Some(vis_issue) = &visual_date_of_issue {
                if let Some(parsed_date) = Self::parse_visual_date(vis_issue) {
                    date_of_issue = parsed_date;
                }
            }
        }
        
        // Core fields prepopulated from MRZ, optional fields may be empty if not found
        let mut additional_fields = std::collections::HashMap::new();
        additional_fields.insert("passport_type".to_string(), "P".to_string());
        
        Ok(VisualData {
            document_type: "P".to_string(),
            issuing_country,
            document_number,
            name: format!("{} {}", surname.trim(), given_names.trim()),
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
            document_format: Some(DocumentFormat::TD3),
            portrait: None,
            signature: None,
            secondary_portrait: None,
            additional_fields,
        })
    }

    // Format MRZ date from YYMMDD to YYYY-MM-DD
    // Removed unused format_mrz_date and format_mrz_expiry_date functions
    // They have been replaced by the more robust format_mrz_date_for_display function

    // Add a specialized MRZ date parser that's more tolerant of OCR errors
    fn parse_mrz_date(date_str: &str) -> Option<String> {
        println!("Parsing MRZ date: {}", date_str);
        
        // Additional cleaning for MRZ dates - more aggressive than visual dates
        let cleaned: String = date_str.chars()
            .map(|c| match c {
                'O' | 'Q' | 'D' | 'C' => '0', // Added 'C' as it's often confused with '0'
                'I' | 'L' | '|' | '!' | 'l' | 'i' => '1', // Added more common confusions
                'Z' | 'z' => '2',
                'E' | 'e' => '3',
                'A' | 'a' => '4',
                'S' | 's' => '5',
                'G' | 'b' => '6',
                'T' | 't' => '7',
                'B' | 'R' => '8',
                'g' | 'q' => '9',
                c if c.is_digit(10) => c,
                _ => '0', // Replace any non-digit with '0' for MRZ
            })
            .collect();
            
        println!("MRZ cleaned date: {}", cleaned);
        
        // For MRZ, we expect exactly 6 digits in YYMMDD format
        if cleaned.len() == 6 {
            // Extract components with fallbacks
            let yy = cleaned[0..2].parse::<u32>().unwrap_or(0);
            let mm = cleaned[2..4].parse::<u32>().unwrap_or(1); // Default to January if invalid
            let dd = cleaned[4..6].parse::<u32>().unwrap_or(1); // Default to 1st if invalid
            
            // Normalize month and day to valid ranges
            let month = if mm < 1 || mm > 12 { 1 } else { mm };
            let day = if dd < 1 || dd > 31 { 1 } else { dd };
            
            // For MRZ, we always return in YYMMDD format
            return Some(format!("{:02}{:02}{:02}", yy, month, day));
        } else if cleaned.len() > 6 {
            // Try to extract 6 digits from a longer string
            let digits_only: String = cleaned.chars().filter(|c| c.is_digit(10)).collect();
            if digits_only.len() >= 6 {
                // Take the first 6 digits
                let yy = digits_only[0..2].parse::<u32>().unwrap_or(0);
                let mm = digits_only[2..4].parse::<u32>().unwrap_or(1);
                let dd = digits_only[4..6].parse::<u32>().unwrap_or(1);
                
                let month = if mm < 1 || mm > 12 { 1 } else { mm };
                let day = if dd < 1 || dd > 31 { 1 } else { dd };
                
                return Some(format!("{:02}{:02}{:02}", yy, month, day));
            }
        }
        
        // Fallback to standard date parser
        Self::parse_visual_date(&cleaned)
    }
    
    // Format MRZ date from YYMMDD format to a human-readable DD MM YYYY format
    // Removed unused format_mrz_date and format_mrz_expiry_date functions
    // They have been replaced by the more robust format_mrz_date_for_display function

    fn parse_visual_date(date_str: &str) -> Option<String> {
        println!("Parsing date: {}", date_str); // Log input string
        // Clean the string: keep only digits and common separators
        let cleaned: String = date_str.chars().filter(|c| c.is_digit(10) || *c == '/' || *c == '-' || *c == '.').collect();
        println!("Cleaned string: {}", cleaned); // Log cleaned string
        // Try regex with separators
        let re_separated = Regex::new(r"(\d{1,2})\s*[/. -]\s*(\d{1,2})\s*[/. -]\s*(\d{2,4})").unwrap();
        if let Some(cap) = re_separated.captures(&cleaned) {
            println!("Capture groups (separated): {}, {}, {}", &cap[1], &cap[2], &cap[3]);
            if let (Ok(num1), Ok(num2), Ok(num3)) = (cap[1].parse::<u32>(), cap[2].parse::<u32>(), cap[3].parse::<u32>()) {
                let (day, month, year) = if num2 >= 1 && num2 <= 12 { (num1, num2, num3) } else if num1 >= 1 && num1 <= 12 { (num2, num1, num3) } else { return None; };
                if (1..=31).contains(&day) && (1..=12).contains(&month) && (1900..=2100).contains(&year) {
                    let yy = (year % 100) as u32;
                    return Some(format!("{:02}{:02}{:02}", yy, month, day));
                }
            }
        } else {
            // Try contiguous digit string, e.g., YYMMDD or YYYYMMDD
            let re_contiguous = Regex::new(r"(\d{6,8})").unwrap();
            if let Some(cap) = re_contiguous.captures(&cleaned) {
                let digits = &cap[1];
                println!("Contiguous digits found: {}", digits); // Log contiguous digits
                let len = digits.len();
                if len == 6 {
                    if let (Ok(yy), Ok(mm), Ok(dd)) = (digits[0..2].parse::<u32>(), digits[2..4].parse::<u32>(), digits[4..6].parse::<u32>()) {
                        if (1..=31).contains(&dd) && (1..=12).contains(&mm) && yy <= 99 {
                            return Some(format!("{:02}{:02}{:02}", yy, mm, dd));
                        }
                    }
                } else if len == 8 {
                    if let (Ok(year), Ok(mm), Ok(dd)) = (digits[0..4].parse::<u32>(), digits[4..6].parse::<u32>(), digits[6..8].parse::<u32>()) {
                        if (1..=31).contains(&dd) && (1..=12).contains(&mm) && (1900..=2100).contains(&year) {
                            let yy = year % 100;
                            return Some(format!("{:02}{:02}{:02}", yy, mm, dd));
                        }
                    }
                }
            } else {
                println!("No date pattern found in cleaned string");
            }
        }
        None
    }
}
