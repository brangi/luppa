use luppa::models::DocumentFormat;
use luppa::processing::{BiometricProcessor, ImageProcessor, OcrProcessor, SecurityProcessor};
use luppa::verification::MRTDVerifier;
use std::path::Path;

fn main() {
    let datapath = OcrProcessor::tessdata_prefix();
    if let Some(path) = datapath {
        std::env::set_var("TESSDATA_PREFIX", &path);
        println!("[DEBUG] Set TESSDATA_PREFIX to {} in main", path);
    }
    println!("MRTD System Demo - ICAO Doc 9303 Compliance");
    println!("=============================================\n");

    // Universal Passport OCR System Features
    println!("Universal Passport OCR System - Key Features:");
    println!("  - Language-Agnostic Field Extraction: supports multiple languages (English, Spanish, French, German)");
    println!("  - Improved Text Extraction: enhanced fuzzy matching, text cleaning, and image preprocessing");
    println!("  - Universal Field Detection: place-of-birth extraction, multi-format dates, label & position-based detection");
    println!("  - Error Handling & Resilience: handles missing language files, graceful fallbacks, multi-OCR configurations");
    println!("  - ML-Enhanced Validation: confidence-based field checks with ML heuristics");
    println!("  - PDF & Image Processing: preprocessing, PDF extraction stub, deskewing placeholder");
    println!("  - Batch Processing: process multiple images with detailed reports on quality & completeness");
    println!("  - Issue Fixing Guidelines: immediate fixes without backups, one issue at a time by code review\n");

    // Create an MRTD verifier
    let verifier = MRTDVerifier::new();

    // Process sample passport images
    let image_paths = [
        "/Users/brangirod/Pictures/2.jpg",
        "/Users/brangirod/Pictures/3.jpeg",
    ];
    for path_str in &image_paths {
        let image_path = Path::new(path_str);
        println!("Processing passport image at: {:?}", image_path);

        // Process the image (load, deskew, crop, enhance)
        let processed_image = match ImageProcessor::process_image(image_path) {
            Ok(data) => {
                println!("Image processing completed");
                data
            }
            Err(e) => {
                eprintln!("Error processing image: {}", e);
                return;
            }
        };

        // Extract MRZ data
        let mrz_data = match OcrProcessor::extract_mrz(&processed_image) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error extracting MRZ: {}", e);
                return;
            }
        };

        // Extract security features
        let security_features = match SecurityProcessor::detect_security_features(&processed_image)
        {
            Ok(features) => features,
            Err(e) => {
                eprintln!("Error extracting security features: {}", e);
                return;
            }
        };

        // Extract biometric data
        let biometric_data = match BiometricProcessor::extract_biometric_data(&processed_image) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error extracting biometric data: {}", e);
                return;
            }
        };

        // Extract document data from visual zone
        let _document_data = match OcrProcessor::extract_visual_data(&processed_image) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error extracting visual data: {}", e);
                return;
            }
        };

        // Verify the document
        match verifier.verify(
            &processed_image,
            &mrz_data,
            &security_features,
            &biometric_data,
        ) {
            Ok(result) => {
                println!("\nVerifying document against ICAO Doc 9303 standards...\n");

                println!("===============================================");
                println!("      MRTD VERIFICATION DETAILED REPORT");
                println!("===============================================\n");

                println!("DOCUMENT INFORMATION:");
                println!("  Document Type: {}", mrz_data.document_type);
                println!(
                    "  Document Format: {}",
                    match mrz_data.document_format {
                        Some(DocumentFormat::TD1) => "ID Card (TD1)",
                        Some(DocumentFormat::TD2) => "ID Card (TD2)",
                        Some(DocumentFormat::TD3) => "Passport",
                        Some(DocumentFormat::MRVA) => "Visa (MRVA)",
                        Some(DocumentFormat::MRVB) => "Visa (MRVB)",
                        None => "Unknown",
                    }
                );
                println!(
                    "  Issuing Country: {}",
                    clean_country_code(&mrz_data.issuing_country)
                );
                println!("  Document Number: {}", mrz_data.document_number);

                let full_name = format!("{} {}", mrz_data.surname, mrz_data.given_names);
                println!("  Name: {}", full_name.trim());

                println!(
                    "  Nationality: {}",
                    clean_country_code(&mrz_data.nationality)
                );
                println!(
                    "  Date of Birth: {}",
                    format_date(&mrz_data.date_of_birth, Some(&mrz_data.issuing_country))
                );
                println!("  Gender: {}", clean_gender(&mrz_data.gender));
                // Show expiry date with month name in parentheses
                let expiry_num =
                    format_date(&mrz_data.date_of_expiry, Some(&mrz_data.issuing_country));
                let parts: Vec<&str> = expiry_num.split(' ').collect();
                let expiry_human = if parts.len() == 3 {
                    let day = parts[0].trim_start_matches('0');
                    let month_name = match parts[1] {
                        "01" => "January",
                        "02" => "February",
                        "03" => "March",
                        "04" => "April",
                        "05" => "May",
                        "06" => "June",
                        "07" => "July",
                        "08" => "August",
                        "09" => "September",
                        "10" => "October",
                        "11" => "November",
                        "12" => "December",
                        _ => parts[1],
                    };
                    format!("{} {}, {}", month_name, day, parts[2])
                } else {
                    expiry_num.clone()
                };
                println!("  Date of Expiry: {} ({})", expiry_num, expiry_human);

                println!("\nVERIFICATION STEPS:");
                println!(
                    "  1. MRZ Validation: {}",
                    if result.mrz_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  2. Security Features: {}",
                    if result.security_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  3. Format Validation: {}",
                    if result.format_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  4. Biometric Validation: {}",
                    if result.biometric_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  5. Database Validation: {}",
                    if result.database_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  6. Expiry Validation: {}",
                    if result.expiry_validation.is_valid {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );
                println!(
                    "  7. PKI Validation: {}",
                    if result.pki_validation.map_or(false, |v| v.is_valid) {
                        "PASSED"
                    } else {
                        "FAILED"
                    }
                );

                println!("\nSECURITY FEATURES DETECTED:");
                println!("  Level 1 (Visual) Features:");
                for feature in &security_features.level_1_features {
                    println!("    - {}", feature);
                }
                println!("  Level 2 (Inspection Equipment) Features:");
                for feature in &security_features.level_2_features {
                    println!("    - {}", feature);
                }
                println!("  Level 3 (Forensic) Features:");
                for feature in &security_features.level_3_features {
                    println!("    - {}", feature);
                }

                println!("\nCHIP DATA:");
                if let Some(chip_data) = &biometric_data.chip_data {
                    println!("  Data Groups Present:");
                    for dg in &chip_data.data_groups_present {
                        println!("    - {}", dg);
                    }
                    println!(
                        "  Authentication: {}",
                        if chip_data.authentication_success {
                            "Successful"
                        } else {
                            "Failed"
                        }
                    );
                    println!(
                        "  Basic Access Control: {}",
                        if chip_data.basic_access_control {
                            "Enabled"
                        } else {
                            "Disabled"
                        }
                    );
                    println!(
                        "  Extended Access Control: {}",
                        if chip_data.extended_access_control {
                            "Enabled"
                        } else {
                            "Disabled"
                        }
                    );
                    println!(
                        "  PACE Authentication: {}",
                        if chip_data.pace_authentication {
                            "Enabled"
                        } else {
                            "Disabled"
                        }
                    );
                    println!(
                        "  Active Authentication: {}",
                        if chip_data.active_authentication {
                            "Enabled"
                        } else {
                            "Disabled"
                        }
                    );
                } else {
                    println!("  No chip data available");
                }

                println!(
                    "\nDocument verification result: {}",
                    if result.is_valid { "VALID" } else { "INVALID" }
                );
            }
            Err(e) => {
                eprintln!("Error verifying document: {}", e);
            }
        }
    } // end for loop
}

// Format dates for better readability
// Format date based on country conventions and input format
fn format_date(date: &str, country_code: Option<&str>) -> String {
    // If the date is already in a readable format, return it
    if date.contains(" ") && date.len() >= 8 {
        // Check if we need to reorder based on country format
        if let Some(parts) = date.split_whitespace().collect::<Vec<&str>>().get(0..3) {
            if parts.len() == 3 {
                return format_date_by_country_convention(
                    parts[0],
                    parts[1],
                    parts[2],
                    country_code,
                );
            }
        }
        return date.to_string();
    }

    // Try to parse common date formats
    if date.len() == 6 {
        // Format YYMMDD (ICAO standard for MRZ)
        let year = &date[0..2];
        let month = &date[2..4];
        let day = &date[4..6];

        // Convert 2-digit year to 4-digit year (assuming 21st century)
        let full_year = format!("20{}", year);

        // Format according to country convention
        return format_date_by_country_convention(day, month, &full_year, country_code);
    }

    // Try to parse other common formats with separators
    if date.contains('.') || date.contains('/') || date.contains('-') {
        let separator = if date.contains('.') {
            '.'
        } else if date.contains('/') {
            '/'
        } else {
            '-'
        };

        let parts: Vec<&str> = date.split(separator).collect();
        if parts.len() == 3 {
            // Determine if this is YYYY-MM-DD or DD-MM-YYYY or MM-DD-YYYY
            if parts[0].len() == 4 {
                // YYYY-MM-DD
                return format_date_by_country_convention(
                    parts[2],
                    parts[1],
                    parts[0],
                    country_code,
                );
            } else {
                // Could be DD-MM-YYYY or MM-DD-YYYY
                // Assume it's DD-MM-YYYY by default, but will be adjusted by country convention
                let year = if parts[2].len() == 2 {
                    format!("20{}", parts[2])
                } else {
                    parts[2].to_string()
                };
                return format_date_by_country_convention(parts[0], parts[1], &year, country_code);
            }
        }
    }

    // Return the original date if we can't parse it
    date.to_string()
}

// Format date according to country conventions
fn format_date_by_country_convention(
    day: &str,
    month: &str,
    year: &str,
    country_code: Option<&str>,
) -> String {
    // Determine date format based on country
    let uses_mdy_format = match country_code {
        Some(code) => {
            match code {
                // Countries that use MM/DD/YYYY format
                "USA" | "US" | "PHL" | "BLZ" | "FM" | "MH" | "PW" => true,
                // All other countries use DD/MM/YYYY format
                _ => false,
            }
        }
        None => false, // Default to DD/MM/YYYY if country is unknown
    };

    // Validate day and month values
    let day_num = day.parse::<u8>().unwrap_or(1);
    let month_num = month.parse::<u8>().unwrap_or(1);

    // Ensure day and month are valid
    let (valid_day, valid_month) = if uses_mdy_format && month_num <= 12 && day_num <= 31 {
        // For US format, if values are ambiguous, assume MM/DD
        (day, month)
    } else if !uses_mdy_format && month_num <= 12 && day_num <= 31 {
        // For international format, if values are ambiguous, assume DD/MM
        (day, month)
    } else if day_num <= 12 && month_num <= 31 {
        // If day could be a month (≤12) and what we thought was month is ≤31
        // This is likely a US format date incorrectly parsed
        if uses_mdy_format {
            (month, day) // Swap for US format
        } else {
            (day, month) // Keep as is for international format
        }
    } else {
        // If values are completely invalid, just use them as-is
        (day, month)
    };

    // Format the date according to ISO standard (DD MM YYYY) for consistency
    // This is the format we'll use for display regardless of input format
    format!("{} {} {}", valid_day, valid_month, year)
}

// Clean gender field
fn clean_gender(gender: &str) -> String {
    match gender.trim() {
        "M" | "m" | "MALE" | "Male" => "Male".to_string(),
        "F" | "f" | "FEMALE" | "Female" => "Female".to_string(),
        "X" | "x" | "OTHER" | "Other" => "Other".to_string(),
        _ => gender.to_string(),
    }
}

// Clean country codes
fn clean_country_code(code: &str) -> String {
    match code {
        "USA" => "United States".to_string(),
        "GBR" => "United Kingdom".to_string(),
        "CAN" => "Canada".to_string(),
        "AUS" => "Australia".to_string(),
        "FRA" => "France".to_string(),
        "DEU" => "Germany".to_string(),
        "ITA" => "Italy".to_string(),
        "ESP" => "Spain".to_string(),
        "JPN" => "Japan".to_string(),
        "CHN" => "China".to_string(),
        "RUS" => "Russia".to_string(),
        "IND" => "India".to_string(),
        "BRA" => "Brazil".to_string(),
        "MEX" => "Mexico".to_string(),
        "ZAF" => "South Africa".to_string(),
        "NLD" => "Netherlands".to_string(),
        "SWE" => "Sweden".to_string(),
        "NOR" => "Norway".to_string(),
        "DNK" => "Denmark".to_string(),
        "FIN" => "Finland".to_string(),
        "CHE" => "Switzerland".to_string(),
        "AUT" => "Austria".to_string(),
        "BEL" => "Belgium".to_string(),
        "PRT" => "Portugal".to_string(),
        "GRC" => "Greece".to_string(),
        "TUR" => "Turkey".to_string(),
        "POL" => "Poland".to_string(),
        "UKR" => "Ukraine".to_string(),
        "THA" => "Thailand".to_string(),
        "VNM" => "Vietnam".to_string(),
        "IDN" => "Indonesia".to_string(),
        "MYS" => "Malaysia".to_string(),
        "SGP" => "Singapore".to_string(),
        "PHL" => "Philippines".to_string(),
        "KOR" => "South Korea".to_string(),
        "PAK" => "Pakistan".to_string(),
        "BGD" => "Bangladesh".to_string(),
        "NGA" => "Nigeria".to_string(),
        "EGY" => "Egypt".to_string(),
        "KEN" => "Kenya".to_string(),
        "GHA" => "Ghana".to_string(),
        "MAR" => "Morocco".to_string(),
        "DZA" => "Algeria".to_string(),
        "TUN" => "Tunisia".to_string(),
        "ARG" => "Argentina".to_string(),
        "CHL" => "Chile".to_string(),
        "COL" => "Colombia".to_string(),
        "PER" => "Peru".to_string(),
        "VEN" => "Venezuela".to_string(),
        "URY" => "Uruguay".to_string(),
        "PRY" => "Paraguay".to_string(),
        "BOL" => "Bolivia".to_string(),
        "ECU" => "Ecuador".to_string(),
        "GTM" => "Guatemala".to_string(),
        "SLV" => "El Salvador".to_string(),
        "HND" => "Honduras".to_string(),
        "NIC" => "Nicaragua".to_string(),
        "CRI" => "Costa Rica".to_string(),
        "PAN" => "Panama".to_string(),
        "DOM" => "Dominican Republic".to_string(),
        "JAM" => "Jamaica".to_string(),
        "TTO" => "Trinidad and Tobago".to_string(),
        "BHS" => "Bahamas".to_string(),
        "BRB" => "Barbados".to_string(),
        "CUB" => "Cuba".to_string(),
        "HTI" => "Haiti".to_string(),
        _ => code.to_string(),
    }
}

// Clean up OCR text
#[allow(dead_code)]
fn clean_ocr_text(text: &str) -> String {
    text.replace("0", "O")
        .replace("1", "I")
        .replace("5", "S")
        .replace("8", "B")
        .replace("6", "G")
        .replace("<", " ")
        .replace("  ", " ")
        .trim()
        .to_string()
}
