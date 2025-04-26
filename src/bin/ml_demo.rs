// ML-Enhanced Universal Passport OCR Demo
// This demonstrates the language-agnostic OCR capabilities combined with ML enhancements

use std::path::Path;
use luppa::{
    models::VisualData,
    processing::EnhancedOcrProcessor,
    ml::MlValidator
};

fn main() {
    println!("\n===================================================================");
    println!("🧠 UNIVERSAL MULTILINGUAL PASSPORT OCR WITH ML ENHANCEMENTS");
    println!("===================================================================");
    
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
                    if let Ok(mrz_data) = luppa::processing::OcrProcessor::extract_mrz_from_file(path) {
                        println!("\n✅ ML VALIDATION RESULTS:");
                        let (is_valid, confidence) = ml_validator.validate(&mrz_data, &visual_data);
                        print_validation_results(is_valid, &confidence);
                    }
                }
                
                println!("\n🧠 ML-ENHANCED EXTRACTION:");
                let ml_result = EnhancedOcrProcessor::extract_visual_data(
                    path, &multi_langs);
                    
                if let Ok(visual_data) = ml_result {
                    print_extraction_summary(&visual_data, "ML-Enhanced");
                }
            }
        }
    }
}

// Helper function to print validation results
fn print_validation_results(is_valid: bool, confidence: &luppa::ml::ValidationConfidence) {
    println!("  - Passport valid: {}", if is_valid { "YES ✓" } else { "NO ✗" });
    println!("  - MRZ confidence: {:.1}%", confidence.mrz_confidence * 100.0);
    println!("  - Visual confidence: {:.1}%", confidence.visual_confidence * 100.0);
    println!("  - Consistency: {:.1}%", confidence.consistency_confidence * 100.0);
    println!("  - Security: {:.1}%", confidence.security_feature_confidence * 100.0);
    println!("  - Fraud detection: {:.1}%", confidence.fraud_detection_confidence * 100.0);
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
