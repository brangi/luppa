use std::fs;
use std::path::Path;
use std::io::Error as IoError;
use crate::processing::enhanced_ocr::EnhancedOcrProcessor;
use crate::models::VisualData;
use std::error::Error;

/// Batch process all images in a directory using EnhancedOcrProcessor::extract_visual_data
pub fn batch_visual_verification<P: AsRef<Path>>(directory: P, _tesseract_langs: &[&str]) -> Result<(), IoError> {
    let dir = directory.as_ref();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read directory {}: {}", dir.display(), e);
            return Err(e);
        }
    };
    println!("Processing images in {}...", dir.display());
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_ascii_lowercase();
                
                // Skip specific files (1.jpeg and 4.jpg) as requested
                let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                if file_name == "1.jpeg" || file_name == "4.jpg" {
                    println!("Skipping excluded file: {}", path.display());
                    continue;
                }
                
                if ["jpg", "jpeg", "png", "bmp", "tiff"].contains(&ext.as_str()) {
                    println!("\n--- Processing: {} ---", path.display());
                    
                    // Use multiple languages for better extraction quality
                    let primary_langs: Vec<&str> = vec!["eng", "spa", "deu", "fra"];
                    println!("Attempting OCR with languages: {}", primary_langs.join(", "));
                    
                    let result = EnhancedOcrProcessor::extract_visual_data(&path, &primary_langs);
                    
                    // If that fails, fall back to English only
                    let result = if result.is_err() {
                        println!("Falling back to English-only OCR...");
                        let eng_only = ["eng"];
                        EnhancedOcrProcessor::extract_visual_data(&path, &eng_only[..])
                    } else {
                        result
                    };
                    
                    match result {
                        Ok(visual_data) => print_visual_data(&visual_data, &path),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Process only the specific files requested by the user (2.jpg, 3.jpeg, 5.pdf)
/// This is a simplified version of batch processing for demonstration purposes
pub fn batch_process_files() -> Result<(), Box<dyn Error>> {
    println!("\n====================================================================");
    println!("🔍 BATCH PROCESSING SPECIFIC PASSPORT FILES");
    println!("====================================================================");
    
    // Define the specific files to process
    let files = ["2.jpg", "3.jpeg", "5.pdf"];
    let base_dir = "./test"; // Look in the test directory where passport_sample.jpg is located
    
    for &file in &files {
        let file_path = Path::new(base_dir).join(file);
        println!("\nProcessing file: {}", file_path.display());
        
        if !file_path.exists() {
            println!("⚠️ File not found: {}", file_path.display());
            continue;
        }
        
        // For demonstration, we'll just print the file name and skip actual processing
        // In a real implementation, use EnhancedOcrProcessor::extract_visual_data
        println!("✅ Found file: {}", file_path.display());
        
        // Process the file if it exists
        if file_path.extension().and_then(|e| e.to_str()).map_or(false, |ext| {
            let ext = ext.to_lowercase();
            ext == "jpg" || ext == "jpeg" || ext == "png"
        }) {
            match EnhancedOcrProcessor::extract_visual_data(&file_path, &["eng"]) {
                Ok(visual_data) => {
                    print_visual_data(&visual_data, &file_path);
                }
                Err(e) => {
                    println!("❌ Error processing file {}: {}", file_path.display(), e);
                }
            };
        } else if file_path.extension().and_then(|e| e.to_str()).map_or(false, |ext| ext.to_lowercase() == "pdf") {
            println!("📄 PDF file detected: {}", file_path.display());
            println!("   PDF processing available through the main validator");
        }
    }
    
    Ok(())
}

/// Print visual data extracted from an image with a visual dashboard
fn print_visual_data(visual_data: &VisualData, path: &Path) {
    println!("==================================================================");
    println!("📄 OCR RESULTS: {}", path.display());
    println!("==================================================================");
    
    // Calculate extraction completeness
    let mut fields_with_data = 0;
    let total_fields = 9; // Count of main fields we're checking
    
    // Document Number
    let doc_num_status = if !visual_data.document_number.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Surname
    let surname_status = if !visual_data.surname.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Given Names
    let given_names_status = if !visual_data.given_names.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Date of Birth
    let dob_status = if !visual_data.date_of_birth.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Date of Issue
    let doi_status = if !visual_data.date_of_issue.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Date of Expiry
    let doe_status = if !visual_data.date_of_expiry.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Place of Birth
    let pob_status = if visual_data.place_of_birth.is_some() && visual_data.place_of_birth.as_ref().map_or(false, |s| !s.is_empty()) {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Gender
    let gender_status = if !visual_data.gender.is_empty() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Authority
    let authority_status = if visual_data.authority.is_some() {
        fields_with_data += 1;
        "✅"
    } else { "❌" };
    
    // Calculate percentage
    let percentage = (fields_with_data as f32 / total_fields as f32) * 100.0;
    
    // Print dashboard
    println!("📋 Extraction Completeness: {:.1}% ({}/{} fields)", percentage, fields_with_data, total_fields);
    println!("-----------------------------------------------------------------");
    println!("  {} Document Number  : {}", doc_num_status, visual_data.document_number);
    println!("  {} Surname          : {}", surname_status, visual_data.surname);
    println!("  {} Given Names      : {}", given_names_status, visual_data.given_names);
    println!("  {} Date of Birth    : {}", dob_status, visual_data.date_of_birth);
    println!("  {} Date of Issue    : {}", doi_status, visual_data.date_of_issue);
    println!("  {} Date of Expiry   : {}", doe_status, visual_data.date_of_expiry);
    println!("  {} Gender           : {}", gender_status, visual_data.gender);
    println!("  {} Place of Birth   : {}", pob_status, visual_data.place_of_birth.as_deref().unwrap_or("None"));
    println!("  {} Authority        : {}", authority_status, visual_data.authority.as_deref().unwrap_or("None"));
    println!("-----------------------------------------------------------------");
}
