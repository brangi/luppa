// Passport validation system in Rust
// Refactored into a modular structure

use std::path::Path;
use luppa::processing::{ImageProcessor, OcrProcessor};
use luppa::verification::MRTDVerifier;
use luppa::utils::PassportError;

fn main() -> Result<(), PassportError> {
    let image_path = Path::new("test_data/passport.jpg");
    
    let processed_image = ImageProcessor::process_image(image_path)?;
    let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;
    let visual_data = OcrProcessor::extract_visual_data(&processed_image)?;
    let result = MRTDVerifier::new().verify(&processed_image, &mrz_data, &visual_data)?;

    println!("Document verification result: {}", if result.is_valid { "Valid" } else { "Invalid" });
    
    Ok(())
}
