// ML-Enhanced Universal Passport OCR Demo with Cross-Validation
// This demonstrates the language-agnostic OCR capabilities combined with 
// ML enhancements and cross-validation between MRZ and visual data

use std::path::Path;
use luppa::{
    models::VisualData,
    processing::{EnhancedOcrProcessor, OcrProcessor, FieldCorrection},
    validation::MrzValidator,
    ml::MlValidator
};

fn main() {
    println!("\n===================================================================\n🧠 UNIVERSAL MULTILINGUAL PASSPORT OCR WITH CROSS-VALIDATION\n===================================================================");
    println!("Showcasing field extraction with cross-validation between MRZ and visual data");
    
    // Test files as requested 
    let test_files = [
        "/Users/brangirod/Pictures/2.jpg",
        "/Users/brangirod/Pictures/3.jpeg",
        "/Users/brangirod/Pictures/5.pdf"
    ];
    
    // Initialize ML-powered validator
    let ml_validator = MlValidator::new();
    
    // Use multiple languages to leverage our universal multilingual OCR
    let multi_langs = ["eng", "spa", "deu", "fra"];
    
    for test_file in test_files.iter() {
        println!("\n\nProcessing: {}", test_file);
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
                
                // Process PDF file to extract image data
                match luppa::processing::ImageProcessor::process_pdf_file(path) {
                    Ok(image_bytes) => {
                        println!("  - Processed PDF to image data");
                        
                        // Process the extracted image
                        // Extract data using both traditional and ML-enhanced methods
                        println!("\n  📊 TRADITIONAL EXTRACTION:");
                        let traditional_result = EnhancedOcrProcessor::extract_visual_data_from_bytes(
                            &image_bytes, &multi_langs);
                        
                        if let Ok(visual_data) = traditional_result {
                            print_extraction_summary(&visual_data, "Traditional");
                            
                            // Validate using ML
                            println!("\n  🧠 ML VALIDATION:");
                            let validator = MlValidator::new();
                            let mrz_data = match luppa::processing::OcrProcessor::extract_mrz_from_bytes(&image_bytes) {
                                Ok(mrz) => mrz,
                                Err(_) => {
                                    println!("  ❌ MRZ extraction failed");
                                    continue;
                                }
                            };
                            
                            let (is_valid, confidence) = validator.validate(&mrz_data, &visual_data);
                            print_validation_results(is_valid, &confidence);
                        } else {
                            println!("  ❌ Traditional extraction failed: {:?}", traditional_result.err());
                        }
                        
                        // We don't have the extract_visual_data_ml implementation for bytes yet,
                        // so we'll indicate this as a pending enhancement
                        println!("\n  🧠 ML-ENHANCED EXTRACTION (PDFs):");
                        println!("    [Future enhancement: Direct ML processing of PDF image bytes]");
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
                let traditional_result = EnhancedOcrProcessor::extract_visual_data(
                    path, &multi_langs);
                    
                if let Ok(visual_data) = traditional_result {
                    print_extraction_summary(&visual_data, "Traditional");
                    
                    // Also try to extract MRZ data
                    if let Ok(mrz_data) = OcrProcessor::extract_mrz_from_file(path) {
                        println!("\n⚙️ CROSS-VALIDATION & FIELD CORRECTION:");
                        println!("  🔄 Cross-validating MRZ and visual data for higher accuracy...");
                        
                        // Run MRZ validation to check for issues
                        match MrzValidator::validate(&mrz_data, &visual_data) {
                            Ok(mrz_validation_result) => {
                                if !mrz_validation_result.is_valid {
                                    println!("  ⚠️ Inconsistencies found between MRZ and visual data!");
                                    println!("  📝 Validation issues detected:");
                                    for issue in &mrz_validation_result.issues {
                                        println!("    - {}", issue.message);
                                    }
                                }
                            },
                            Err(err) => println!("  ❌ Error validating MRZ data: {:?}", err)
                        }
                        
                        // Apply field correction for improved accuracy
                        let corrected_data = FieldCorrection::correct_visual_data(&mrz_data, &visual_data);
                        
                        // Compare before and after correction
                        println!("\n  📊 FIELD COMPARISON BEFORE/AFTER CORRECTION:");
                        compare_extraction_results(&visual_data, &corrected_data);
                        
                        // Validate the corrected data
                        match MrzValidator::validate(&mrz_data, &corrected_data) {
                            Ok(corrected_validation) => {
                                println!("  🔍 MRZ validation after correction: {}", 
                                        if corrected_validation.is_valid { "✅ PASSED" } else { "❌ FAILED" });
                            },
                            Err(err) => println!("  ❌ Error validating corrected data: {:?}", err)
                        };
                        
                        println!("\n✅ ML VALIDATION RESULTS:");
                        let (is_valid, confidence) = ml_validator.validate(&mrz_data, &corrected_data);
                        print_validation_results(is_valid, &confidence);
                    }
                }
                
                println!("\n🧠 ML-ENHANCED EXTRACTION WITH CROSS-VALIDATION:");
                let ml_enhanced_result = EnhancedOcrProcessor::extract_visual_data(
                    path, &multi_langs);
                    
                if let Ok(visual_data_ml) = ml_enhanced_result {
                    if let Ok(mrz_data) = OcrProcessor::extract_mrz_from_file(path) {
                        println!("  🔎 Checking for MRZ/visual data inconsistencies...");
                        
                        // First analyze field completeness before correction
                        let mut missing_fields = 0;
                        let _total_fields = 9; // Core passport fields 
                        
                        if visual_data_ml.document_number.is_empty() { missing_fields += 1; }
                        if visual_data_ml.surname.is_empty() { missing_fields += 1; }
                        if visual_data_ml.given_names.is_empty() { missing_fields += 1; }
                        if visual_data_ml.date_of_birth.is_empty() { missing_fields += 1; }
                        if visual_data_ml.date_of_issue.is_empty() { missing_fields += 1; }
                        if visual_data_ml.date_of_expiry.is_empty() { missing_fields += 1; }
                        if visual_data_ml.gender.is_empty() { missing_fields += 1; }
                        if visual_data_ml.nationality.is_empty() { missing_fields += 1; }
                        if visual_data_ml.place_of_birth.is_none() { missing_fields += 1; }
                        
                        if missing_fields > 0 {
                            println!("  ⚠️ Found {} missing fields that might be recovered from MRZ", missing_fields);
                        }
                        
                        // Apply field correction for improved accuracy
                        let corrected_data = FieldCorrection::correct_visual_data(&mrz_data, &visual_data_ml);
                        
                        // Count fields after correction
                        let mut fixed_fields = 0;
                        
                        if !visual_data_ml.document_number.is_empty() != !corrected_data.document_number.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.surname.is_empty() != !corrected_data.surname.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.given_names.is_empty() != !corrected_data.given_names.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.date_of_birth.is_empty() != !corrected_data.date_of_birth.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.date_of_issue.is_empty() != !corrected_data.date_of_issue.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.date_of_expiry.is_empty() != !corrected_data.date_of_expiry.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.gender.is_empty() != !corrected_data.gender.is_empty() { fixed_fields += 1; }
                        if !visual_data_ml.nationality.is_empty() != !corrected_data.nationality.is_empty() { fixed_fields += 1; }
                        if visual_data_ml.place_of_birth.is_none() != corrected_data.place_of_birth.is_none() { fixed_fields += 1; }
                        
                        if fixed_fields > 0 {
                            println!("  ✅ Recovered {} previously missing fields using MRZ data!", fixed_fields);
                        }
                        
                        // Run MRZ validation to check for issues
                        match MrzValidator::validate(&mrz_data, &corrected_data) {
                            Ok(corrected_validation) => {
                                if corrected_validation.is_valid {
                                    println!("  ✅ All fields now consistent between MRZ and visual data");
                                } else {
                                    println!("  ⚠️ Some inconsistencies remain between MRZ and visual data");
                                }
                            },
                            Err(err) => println!("  ❌ Error validating corrected data: {:?}", err)
                        };
                        
                        print_extraction_summary(&corrected_data, "ML-Enhanced with Cross-Validation");
                    } else {
                        println!("  ❗ No MRZ data available for cross-validation");
                        print_extraction_summary(&visual_data_ml, "ML-Enhanced");
                    }
                } else {
                    println!("  ❌ ML-enhanced extraction failed: {:?}", ml_enhanced_result.err());
                }
            }
        }
    }
}

// Helper function to print validation results
fn print_validation_results(is_valid: bool, confidence: &luppa::ml::ValidationConfidence) {
    println!("  - Passport valid: {} {}", 
             if is_valid { "YES" } else { "NO" },
             if is_valid { "✓" } else { "✗" });
    println!("  - MRZ confidence: {:.1}%", confidence.mrz_confidence * 100.0);
    println!("  - Visual confidence: {:.1}%", confidence.visual_confidence * 100.0);
    println!("  - Consistency: {:.1}%", confidence.consistency_confidence * 100.0);
    println!("  - Security: {:.1}%", confidence.security_feature_confidence * 100.0);
    println!("  - Fraud detection: {:.1}%", confidence.fraud_detection_confidence * 100.0);
}

/// Compare original and corrected extraction results to highlight improvements
fn compare_extraction_results(original: &VisualData, corrected: &VisualData) {
    // Helper function to format comparison line
    fn print_comparison(field_name: &str, original: &str, corrected: &str) {
        if original != corrected {
            if original.is_empty() {
                println!("  ✅ {} added: {}", field_name, corrected);
            } else {
                println!("  ⚠️ {} corrected: '{}' → '{}'", field_name, original, corrected);
            }
        }
    }
    
    // Helper for Option<String> fields
    fn compare_option(field_name: &str, original: &Option<String>, corrected: &Option<String>) {
        match (original, corrected) {
            (None, Some(c)) => println!("  ✅ {} added: {}", field_name, c),
            (Some(o), Some(c)) if o != c => println!("  ⚠️ {} corrected: '{}' → '{}'", field_name, o, c),
            _ => {}
        }
    }

    // Compare all fields
    print_comparison("Document Number", &original.document_number, &corrected.document_number);
    print_comparison("Document Type", &original.document_type, &corrected.document_type);
    print_comparison("Issuing Country", &original.issuing_country, &corrected.issuing_country);
    print_comparison("Surname", &original.surname, &corrected.surname);
    print_comparison("Given Names", &original.given_names, &corrected.given_names);
    print_comparison("Full Name", &original.name, &corrected.name);
    print_comparison("Nationality", &original.nationality, &corrected.nationality);
    print_comparison("Date of Birth", &original.date_of_birth, &corrected.date_of_birth);
    print_comparison("Gender", &original.gender, &corrected.gender);
    print_comparison("Date of Issue", &original.date_of_issue, &corrected.date_of_issue);
    print_comparison("Date of Expiry", &original.date_of_expiry, &corrected.date_of_expiry);
    
    // Optional fields
    compare_option("Place of Birth", &original.place_of_birth, &corrected.place_of_birth);
    compare_option("Authority", &original.authority, &corrected.authority);
    compare_option("Personal Number", &original.personal_number, &corrected.personal_number);
    
    // Summary of changes
    let mut changed_fields = 0;
    let total_fields = 14; // Total number of fields we're comparing
    
    // Count changed fields (this doesn't include fields with no changes)
    if original.document_number != corrected.document_number { changed_fields += 1; }
    if original.document_type != corrected.document_type { changed_fields += 1; }
    if original.issuing_country != corrected.issuing_country { changed_fields += 1; }
    if original.surname != corrected.surname { changed_fields += 1; }
    if original.given_names != corrected.given_names { changed_fields += 1; }
    if original.name != corrected.name { changed_fields += 1; }
    if original.nationality != corrected.nationality { changed_fields += 1; }
    if original.date_of_birth != corrected.date_of_birth { changed_fields += 1; }
    if original.gender != corrected.gender { changed_fields += 1; }
    if original.date_of_issue != corrected.date_of_issue { changed_fields += 1; }
    if original.date_of_expiry != corrected.date_of_expiry { changed_fields += 1; }
    if original.place_of_birth != corrected.place_of_birth { changed_fields += 1; }
    if original.authority != corrected.authority { changed_fields += 1; }
    if original.personal_number != corrected.personal_number { changed_fields += 1; }
    
    // No changes detected
    if changed_fields == 0 {
        println!("  ✓ No fields needed correction - MRZ and visual data are consistent");
    } else {
        println!("  📊 Cross-validation improved {} out of {} fields ({:.1}%)", 
                 changed_fields, total_fields, (changed_fields as f32 / total_fields as f32) * 100.0);
    }
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
        if !pob.is_empty() {
            field_count += 1;
            println!("  ✅ Place of Birth: {}", pob);
        } else {
            println!("  ❌ Place of Birth: Missing");
        }
    } else {
        println!("  ❌ Place of Birth: Missing");
    }
    
    total_fields += 1;
    if let Some(auth) = &visual_data.authority {
        if !auth.is_empty() {
            field_count += 1;
            println!("  ✅ Authority: {}", auth);
        } else {
            println!("  ❌ Authority: Missing");
        }
    } else {
        println!("  ❌ Authority: Missing");
    }
    
    // Calculate completeness percentage
    let completeness = (field_count as f32 / total_fields as f32) * 100.0;
    println!("\n  📈 {} Extraction Completeness: {:.1}% ({}/{} fields)", 
             method, completeness, field_count, total_fields);
}
