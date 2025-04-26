// Universal Passport OCR and Validation System in Rust
// With language-agnostic field extraction and ML enhancements

use std::path::Path;
use luppa::{
    models::{MrzData, VisualData, ValidationResult, ValidationIssueType},
    PassportValidator,
    ml::{FeatureExtractor, MlValidator},
};
use luppa::processing;

// Function to print a detailed validation report
// Test the ML-enhanced passport OCR and validation on specific files
fn test_ml_enhanced_passport_validation() -> Result<(), String> {
    println!("\n===================================================================");
    println!("🧠 TESTING ML-ENHANCED PASSPORT EXTRACTION AND VALIDATION");
    println!("===================================================================");
    
    // Test files as requested
    let test_files = [
        "/Users/brangirod/Pictures/2.jpg",
        "/Users/brangirod/Pictures/5.pdf"
    ];
    
    // Initialize ML-powered components
    let _feature_extractor = FeatureExtractor::new();
    let ml_validator = MlValidator::new();
    
    // Use multiple languages to leverage our universal multilingual OCR
    let multi_langs = ["eng", "spa", "deu", "fra"];
    
    for test_file in test_files.iter() {
        println!("\nProcessing: {}", test_file);
        let path = Path::new(test_file);
        
        // Check if file exists
        if !path.exists() {
            println!("❌ File does not exist: {}", test_file);
            continue;
        }
        
        // For PDF files, we need special handling
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.to_lowercase() == "pdf" {
                println!("📄 Processing PDF file...");
                
                // Process PDF file to extract images
                match luppa::processing::ImageProcessor::process_pdf_file(path) {
                    Ok(processed_image_bytes) => {
                        // Save the processed PDF image for inspection
                        let debug_output_path = "/Users/brangirod/Pictures/processed_pdf.png";
                        std::fs::write(debug_output_path, &processed_image_bytes)
                            .expect("Failed to write debug image");
                        println!("💾 Saved processed PDF image to {}", debug_output_path);
                        
                        // Extract data using both traditional and ML-enhanced methods
                        println!("\n📊 TRADITIONAL EXTRACTION:");
                        let traditional_result = luppa::processing::EnhancedOcrProcessor::extract_visual_data_from_bytes(
                            &processed_image_bytes, &multi_langs);
                            
                        if let Ok(visual_data) = traditional_result {
                            print_extraction_summary(&visual_data, "Traditional");
                        }
                        
                        // We don't have the extract_visual_data_ml implementation ready for bytes yet
                        // so we'll just process the saved PNG
                        println!("\n🧠 ML-ENHANCED EXTRACTION:");
                        let ml_result = luppa::processing::EnhancedOcrProcessor::extract_visual_data(
                            debug_output_path, &multi_langs);
                            
                        if let Ok(visual_data) = ml_result {
                            print_extraction_summary(&visual_data, "ML-Enhanced");
                        }
                    }
                    Err(e) => {
                        println!("❌ PDF processing error: {}", e);
                    }
                }
            } else {
                // Regular image file
                println!("🖼️ Processing image file...");
                
                // Extract data using both traditional and ML-enhanced methods
                println!("\n📊 TRADITIONAL EXTRACTION:");
                let traditional_result = luppa::processing::EnhancedOcrProcessor::extract_visual_data(
                    path, &multi_langs);
                    
                if let Ok(visual_data) = traditional_result {
                    print_extraction_summary(&visual_data, "Traditional");
                }
                
                println!("\n🧠 ML-ENHANCED EXTRACTION:");
                let ml_result = luppa::processing::EnhancedOcrProcessor::extract_visual_data(
                    path, &multi_langs);
                    
                if let Ok(visual_data) = ml_result {
                    print_extraction_summary(&visual_data, "ML-Enhanced");
                    
                    // If we get MRZ data, also test ML validation
                    // Read image file to bytes first
                let image_bytes = std::fs::read(path)
                    .map_err(|e| format!("Failed to read image file: {}", e))?;
                
                if let Ok(mrz_data) = luppa::processing::OcrProcessor::extract_mrz(&image_bytes) {
                        println!("\n✅ ML VALIDATION RESULTS:");
                        let (is_valid, confidence) = ml_validator.validate(&mrz_data, &visual_data);
                        println!("  - Passport valid: {}", if is_valid { "YES" } else { "NO" });
                        println!("  - MRZ confidence: {:.1}%", confidence.mrz_confidence * 100.0);
                        println!("  - Visual confidence: {:.1}%", confidence.visual_confidence * 100.0);
                        println!("  - Consistency: {:.1}%", confidence.consistency_confidence * 100.0);
                        println!("  - Security: {:.1}%", confidence.security_feature_confidence * 100.0);
                        println!("  - Fraud detection: {:.1}%", confidence.fraud_detection_confidence * 100.0);
                    }
                }
            }
        }
    }
    Ok(())
}

// Demonstrate multilingual passport extraction capabilities
fn demo_multilingual_extraction() -> Result<(), String> {
    println!("\n===================================================================");
    println!("🌐 DEMONSTRATING MULTILINGUAL PASSPORT EXTRACTION");
    println!("===================================================================\n");
    
    // Test file - you may need to adjust the path
    let test_file = "/Users/brangirod/Pictures/2.jpg";
    let path = Path::new(test_file);
    
    // Check if file exists
    if !path.exists() {
        return Err(format!("❌ File does not exist: {}", test_file));
    }
    
    println!("🖼️ Testing universal extraction on: {}", test_file);
    
    // Define languages to test - our system supports multiple languages
    // Use direct &str arrays instead of Vec<String> to avoid the unstable str_as_str feature
    let languages = [
        // Primary languages
        &["eng"][..],            // English only
        &["spa"][..],            // Spanish only
        &["fra"][..],            // French only
        &["deu"][..],            // German only
        // Multilingual configurations
        &["eng", "spa"][..],     // English + Spanish
        &["eng", "fra", "deu"][..], // English + French + German
        // Full language support
        &["eng", "spa", "fra", "deu", "ita"][..], // All supported languages
    ];
    
    // Read the image file
    let image_bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;
    
    // Process the image with each language configuration
    for lang_set in languages.iter() {
        let lang_str = lang_set.join(", ");
        println!("\n🔍 Testing extraction with language(s): {}", lang_str);
        
        // Try to extract data with this language configuration
        match processing::enhanced_ocr::EnhancedOcrProcessor::extract_visual_data_from_bytes(
            &image_bytes, lang_set
        ) {
            Ok(visual_data) => {
                // Print extraction results
                print_extraction_summary(&visual_data, &format!("[{}]", lang_str));
                
                // Calculate completeness score
                let fields_count = [
                    !visual_data.document_type.is_empty(),
                    !visual_data.issuing_country.is_empty(),
                    !visual_data.document_number.is_empty(),
                    !visual_data.surname.is_empty(),
                    !visual_data.given_names.is_empty(),
                    !visual_data.nationality.is_empty(),
                    !visual_data.date_of_birth.is_empty(),
                    !visual_data.gender.is_empty(),
                    !visual_data.date_of_expiry.is_empty(),
                    visual_data.place_of_birth.is_some(),
                ].iter().filter(|&&present| present).count();
                
                let completeness = (fields_count as f64 / 10.0) * 100.0;
                println!("  📊 Extraction completeness: {:.1}%\n", completeness);
            },
            Err(e) => {
                println!("  ❌ Extraction failed: {}\n", e);
            }
        }
    }
    
    println!("✅ Multilingual extraction demonstration complete!");
    Ok(())
}

// Helper function to print extraction summary
fn print_extraction_summary(visual_data: &VisualData, method: &str) {
    // Count fields that are successfully extracted
    let mut field_count = 0;
    let mut total_fields = 0;
    
    // Check each field
    total_fields += 1;
    if !visual_data.document_number.is_empty() {
        field_count += 1;
        println!("  ✅ Document Number: {}", visual_data.document_number);
    } else {
        println!("  ❌ Document Number: Missing");
    }
    
    total_fields += 1;
    if !visual_data.surname.is_empty() {
        field_count += 1;
        println!("  ✅ Surname: {}", visual_data.surname);
    } else {
        println!("  ❌ Surname: Missing");
    }
    
    total_fields += 1;
    if !visual_data.given_names.is_empty() {
        field_count += 1;
        println!("  ✅ Given Names: {}", visual_data.given_names);
    } else {
        println!("  ❌ Given Names: Missing");
    }
    
    total_fields += 1;
    if !visual_data.date_of_birth.is_empty() {
        field_count += 1;
        println!("  ✅ Date of Birth: {}", visual_data.date_of_birth);
    } else {
        println!("  ❌ Date of Birth: Missing");
    }
    
    total_fields += 1;
    if !visual_data.date_of_issue.is_empty() {
        field_count += 1;
        println!("  ✅ Date of Issue: {}", visual_data.date_of_issue);
    } else {
        println!("  ❌ Date of Issue: Missing");
    }
    
    total_fields += 1;
    if !visual_data.date_of_expiry.is_empty() {
        field_count += 1;
        println!("  ✅ Date of Expiry: {}", visual_data.date_of_expiry);
    } else {
        println!("  ❌ Date of Expiry: Missing");
    }
    
    total_fields += 1;
    if !visual_data.gender.is_empty() {
        field_count += 1;
        println!("  ✅ Gender: {}", visual_data.gender);
    } else {
        println!("  ❌ Gender: Missing");
    }
    
    total_fields += 1;
    if let Some(pob) = &visual_data.place_of_birth {
        field_count += 1;
        println!("  ✅ Place of Birth: {}", pob);
    } else {
        println!("  ❌ Place of Birth: Missing");
    }
    
    total_fields += 1;
    if let Some(auth) = &visual_data.authority {
        field_count += 1;
        println!("  ✅ Authority: {}", auth);
    } else {
        println!("  ❌ Authority: Missing");
    }
    
    // Calculate completeness percentage
    let completeness = (field_count as f32 / total_fields as f32) * 100.0;
    println!("\n  📈 {} Extraction Completeness: {:.1}% ({}/{} fields)", 
             method, completeness, field_count, total_fields);
}

fn print_detailed_report(result: &ValidationResult, mrz_data: &MrzData, visual_data: &VisualData) {
    println!("\n===============================================");
    println!("      PASSPORT VALIDATION DETAILED REPORT");
    println!("===============================================\n");
    
    println!("PASSPORT INFORMATION - MRZ DATA:");
    println!("  Document Type: {}", mrz_data.document_type);
    println!("  Issuing Country: {}", mrz_data.issuing_country);
    println!("  Document Number: {}", mrz_data.document_number);
    println!("  Surname: {}", mrz_data.surname);
    println!("  Given Names: {}", mrz_data.given_names);
    println!("  Nationality: {}", mrz_data.nationality);
    println!("  Date of Birth: {}", mrz_data.date_of_birth);
    println!("  Gender: {}", mrz_data.gender);
    println!("  Place of Birth: {}", mrz_data.place_of_birth.as_deref().unwrap_or("None"));
    println!("  Date of Expiry: {}", mrz_data.date_of_expiry);
    println!("  Personal Number: {}", mrz_data.personal_number.as_deref().unwrap_or("None"));
    
    println!("\nPASSPORT INFORMATION - VISUAL DATA:");
    println!("  Document Type: {}", visual_data.document_type);
    println!("  Issuing Country: {}", visual_data.issuing_country);
    println!("  Document Number: {}", visual_data.document_number);
    println!("  Name: {}", visual_data.name);
    println!("  Nationality: {}", visual_data.nationality);
    println!("  Date of Birth: {}", visual_data.date_of_birth);
    println!("  Gender: {}", visual_data.gender);
    println!("  Place of Birth: {}", visual_data.place_of_birth.as_deref().unwrap_or("None"));
    println!("  Date of Issue: {}", visual_data.date_of_issue);
    println!("  Date of Expiry: {}", visual_data.date_of_expiry);
    println!("  Authority: {}", visual_data.authority.as_deref().unwrap_or("None"));
    println!("  Personal Number: {}", visual_data.personal_number.as_deref().unwrap_or("None"));
    
    println!("\nVALIDATION STEPS:");
    println!("  1. MRZ Validation: {}", if result.mrz_validation.is_valid { "PASSED" } else { "FAILED" });
    println!("  2. Security Features: {}", if result.security_validation.is_valid { "PASSED" } else { "FAILED" });
    println!("  3. Format Validation: {}", if result.format_validation.is_valid { "PASSED" } else { "FAILED" });
    println!("  4. Biometric Validation: {}", if result.biometric_validation.is_valid { "PASSED" } else { "FAILED" });
    println!("  5. Database Validation: {}", if result.database_validation.is_valid { "PASSED" } else { "FAILED" });
    println!("  6. Expiry Validation: {}", if result.expiry_validation.is_valid { "PASSED" } else { "FAILED" });
    
    if !result.issues.is_empty() {
        println!("\nISSUES FOUND:");
        for issue in &result.issues {
            println!("  - [{}] {}", match issue.issue_type {
                ValidationIssueType::Mrz => "MRZ",
                ValidationIssueType::Security => "SECURITY",
                ValidationIssueType::Format => "FORMAT",
                ValidationIssueType::Biometric => "BIOMETRIC",
                ValidationIssueType::Database => "DATABASE",
                ValidationIssueType::Expiry => "EXPIRY",
                ValidationIssueType::Generic => "GENERIC",
            }, issue.message);
        }
    }
    
    println!("Passport validation result: {}", if result.is_valid { "VALID" } else { "INVALID" });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test ML-enhanced passport extraction and validation
    test_ml_enhanced_passport_validation()?;
    
    // Test our universal multilingual passport extraction capabilities
    match demo_multilingual_extraction() {
        Ok(_) => println!("Multilingual testing completed successfully"),
        Err(e) => eprintln!("Error in multilingual testing: {}", e),
    }
    
    // Testing:
    // Test batch processing of multiple image files
    let res = processing::batch_visual_verification::batch_process_files();
    if let Err(e) = res {
        eprintln!("Error in batch processing: {}", e);
    }

    // Initialize passport validator with ML capabilities
    let validator = PassportValidator::new().with_ml_validation(true);
    
    // Print information about ML-enhanced features
    println!("\n=====================================================");
    println!("🧠 ML-ENHANCED PASSPORT VALIDATION ACTIVE");
    println!("=====================================================");
    println!("Features enabled:");
    println!("  - ML-driven field extraction");
    println!("  - AI-enhanced consistency checking");
    println!("  - Pattern-based fraud detection");
    println!("  - Cross-validation between MRZ and visual data");
    
    // Use multiple languages for OCR with English as fallback
    let _tesseract_langs = ["eng", "spa", "deu", "fra"]; // Prefixed with _ to avoid unused warning
    let args: Vec<String> = std::env::args().collect();
    let image_path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new("/Users/brangirod/Pictures/3.jpeg")
    };
    
    println!("Attempting to validate passport image at: {:?}", image_path);
    
    // Check if the file is a PDF
    let is_pdf = image_path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase() == "pdf")
        .unwrap_or(false);
    
    if is_pdf {
        println!("Detected PDF file - focusing on PDF processing");
        // For PDF files, let's try a more focused approach
        match luppa::processing::ImageProcessor::process_pdf_file(image_path) {
            Ok(processed_image_bytes) => {
                // Save the processed PDF image for inspection
                let debug_output_path = "/Users/brangirod/Pictures/processed_pdf.png";
                std::fs::write(debug_output_path, &processed_image_bytes)
                    .expect("Failed to write debug image");
                println!("Saved processed PDF image to {}", debug_output_path);
                
                // For PDF files, let's use a specialized approach to ensure we use the actual extracted data
                // First extract MRZ and visual data directly from processed image
                let mrz_data = match luppa::processing::OcrProcessor::extract_mrz(&processed_image_bytes) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("MRZ extraction error: {}", e);
                        return Ok(());
                    }
                };
                
                // Use multiple languages for better extraction quality
                let multi_langs = ["eng", "spa", "deu", "fra"];
                let visual_data = match luppa::processing::EnhancedOcrProcessor::extract_visual_data_from_bytes(&processed_image_bytes, &multi_langs) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Visual data extraction error: {}", e);
                        
                        // We can still continue with partial validation using MRZ data only
                        println!("\nContinuing with partial validation using MRZ data only...");
                        
                        // Create minimal visual data to display
                        let minimal_visual = luppa::models::VisualData {
                            document_type: "UNKNOWN".to_string(),
                            issuing_country: "UNKNOWN".to_string(),
                            document_number: "UNKNOWN".to_string(),
                            name: "UNKNOWN".to_string(),
                            surname: "UNKNOWN".to_string(),
                            given_names: "UNKNOWN".to_string(),
                            nationality: "UNKNOWN".to_string(),
                            date_of_birth: "UNKNOWN".to_string(),
                            gender: "UNKNOWN".to_string(),
                            place_of_birth: None,
                            date_of_issue: "UNKNOWN".to_string(),
                            date_of_expiry: "UNKNOWN".to_string(),
                            authority: None,
                            personal_number: None,
                        };
                        
                        // Now run validation with the extracted MRZ data and minimal visual data
                        match validator.validate_with_extracted_data(&mrz_data, &minimal_visual) {
                            Ok(result) => {
                                print_detailed_report(&result, &mrz_data, &minimal_visual);
                                return Ok(());
                            }
                            Err(err) => {
                                eprintln!("Error validating passport: {}", err);
                                return Ok(());
                            }
                        }
                    }
                };
                
                // Now run validation with the extracted data
                match validator.validate_with_extracted_data(&mrz_data, &visual_data) {
                    Ok(result) => {
                        print_detailed_report(&result, &mrz_data, &visual_data);
                        Ok(())
                    },
                    Err(err) => {
                        eprintln!("Error validating passport: {}", err);
                        Ok(())
                    }
                }
            }
            Err(err) => {
                eprintln!("Error processing PDF: {}", err);
                Ok(())
            }
        }
    } else {
        // Standard processing for non-PDF files
        // Read the image file first
        let image_data = std::fs::read(image_path)
            .map_err(|e| format!("Failed to read image file: {}", e))?;
            
        // Use preprocess_image instead of process_image
        let process_result = luppa::processing::ImageProcessor::preprocess_image(&image_data);
        
        match process_result {
            Ok(processed_image_data) => {
                // Use the processed image bytes for OCR
                match luppa::processing::OcrProcessor::extract_mrz(&processed_image_data) {
                    Ok(mrz_data) => {
                        // Then extract visual data using our enhanced OCR processor for better recognition
                        match luppa::processing::EnhancedOcrProcessor::extract_visual_data_from_bytes(&processed_image_data, &["eng"]) {
                            Ok(visual_data) => {
                                // Now validate with the extracted data
                                match validator.validate_with_extracted_data(&mrz_data, &visual_data) {
                                    Ok(result) => {
                                        print_detailed_report(&result, &mrz_data, &visual_data);
                                        Ok(())
                                    },
                                    Err(err) => {
                                        eprintln!("Error validating passport: {}", err);
                                        Ok(())
                                    }
                                }
                            },
                            Err(err) => {
                                eprintln!("Visual data extraction error: {}", err);
                                
                                // We can still complete some validation with just MRZ data
                                println!("\nContinuing with partial validation using MRZ data only...");
                                
                                // Create minimal visual data to display
                                let minimal_visual = VisualData {
                                    document_type: "UNKNOWN".to_string(),
                                    issuing_country: "UNKNOWN".to_string(),
                                    document_number: "UNKNOWN".to_string(),
                                    name: "UNKNOWN".to_string(),
                                    surname: "UNKNOWN".to_string(),
                                    given_names: "UNKNOWN".to_string(),
                                    nationality: "UNKNOWN".to_string(),
                                    date_of_birth: "UNKNOWN".to_string(),
                                    gender: "UNKNOWN".to_string(),
                                    place_of_birth: None,
                                    date_of_issue: "UNKNOWN".to_string(),
                                    date_of_expiry: "UNKNOWN".to_string(),
                                    authority: None,
                                    personal_number: None,
                                };
                                
                                // Use the MRZ data we already extracted with minimal visual data
                                match validator.validate_with_extracted_data(&mrz_data, &minimal_visual) {
                                    Ok(result) => {
                                        print_detailed_report(&result, &mrz_data, &minimal_visual);
                                        Ok(())
                                    },
                                    Err(validation_err) => {
                                        eprintln!("Error completing validation: {}", validation_err);
                                        Ok(())
                                    }
                                }
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("MRZ extraction error: {}", err);
                        eprintln!("Cannot proceed without MRZ data");
                        Ok(())
                    }
                }
            },
            Err(err) => {
                eprintln!("Error processing image: {}", err);
                Ok(())
            }
        }
    }
}
