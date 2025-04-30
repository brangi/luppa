use std::path::Path;
use luppa::processing::{ImageProcessor, OcrProcessor};
use luppa::verification::MRTDVerifier;
use luppa::utils::PassportError;

fn main() -> Result<(), PassportError> {
    // Set TESSDATA_PREFIX for OCR
    std::env::set_var("TESSDATA_PREFIX", "/usr/local/share/tessdata");

    println!("MRTD System Demo");
    println!("---------------");

    let image_path = Path::new("test_data/passport.jpg");
    
    if !image_path.exists() {
        println!("\nError: Test passport image not found!");
        println!("\nPlease add a test passport image at: test_data/passport.jpg");
        println!("See test_data/README.md for image requirements and guidelines.");
        return Ok(());
    }
    
    println!("Processing passport image...");
    let processed_image = ImageProcessor::process_image(image_path)?;
    
    println!("Extracting MRZ data...");
    let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;
    
    println!("Extracting visual data...");
    let visual_data = OcrProcessor::extract_visual_data(&processed_image)?;
    
    println!("Verifying document...");
    let result = MRTDVerifier::new().verify(&processed_image, &mrz_data, &visual_data)?;

    println!("\nVERIFICATION RESULT:");
    println!("  Document is {}", if result.is_valid { "VALID" } else { "INVALID" });
    
    if !result.is_valid {
        println!("\nISSUES FOUND:");
        for issue in &result.issues {
            println!("  - {}", issue.message);
        }
    }
    
    Ok(())
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
