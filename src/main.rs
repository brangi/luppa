// Passport validation system in Rust
// Refactored into a modular structure

use luppa::{
    models::{MrzData, ValidationIssueType, ValidationResult, VisualData},
    PassportValidator,
};
use std::path::Path;

// Function to print a detailed validation report
fn print_detailed_report(result: &ValidationResult, _mrz_data: &MrzData, visual_data: &VisualData) {
    println!("\n===============================================");
    println!("      PASSPORT VALIDATION DETAILED REPORT");
    println!("===============================================\n");

    println!("PASSPORT INFORMATION:");
    println!("  Document Type: {}", visual_data.document_type);
    println!("  Issuing Country: {}", visual_data.issuing_country);
    println!("  Document Number: {}", visual_data.document_number);
    println!("  Name: {}", visual_data.name);
    println!("  Nationality: {}", visual_data.nationality);
    println!("  Date of Birth: {}", visual_data.date_of_birth);
    println!("  Gender: {}", visual_data.gender);
    println!("  Place of Birth: {:?}", visual_data.place_of_birth);
    println!("  Date of Issue: {}", visual_data.date_of_issue);
    println!("  Date of Expiry: {}", visual_data.date_of_expiry);
    println!("  Authority: {:?}", visual_data.authority);
    println!("  Personal Number: {:?}", visual_data.personal_number);

    println!("\nVALIDATION STEPS:");
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

    if !result.issues.is_empty() {
        println!("\nISSUES FOUND:");
        for issue in &result.issues {
            println!(
                "  - [{}] {}",
                match issue.issue_type {
                    ValidationIssueType::Mrz => "MRZ",
                    ValidationIssueType::Security => "SECURITY",
                    ValidationIssueType::Format => "FORMAT",
                    ValidationIssueType::Biometric => "BIOMETRIC",
                    ValidationIssueType::Database => "DATABASE",
                    ValidationIssueType::Expiry => "EXPIRY",
                    ValidationIssueType::Generic => "GENERIC",
                    _ => "UNKNOWN",
                },
                issue.message
            );
        }
    }

    println!(
        "Passport validation result: {}",
        if result.is_valid { "VALID" } else { "INVALID" }
    );
}

fn main() {
    let validator = PassportValidator::new();
    let image_path = Path::new("/Users/brangirod/Pictures/3.jpeg");

    println!("Attempting to validate passport image at: {:?}", image_path);

    match validator.validate(image_path) {
        Ok(result) => {
            let mrz_data = luppa::processing::OcrProcessor::extract_mrz(
                &luppa::processing::ImageProcessor::process_image(image_path).unwrap(),
            )
            .unwrap();
            let visual_data = luppa::processing::OcrProcessor::extract_visual_data(
                &luppa::processing::ImageProcessor::process_image(image_path).unwrap(),
            )
            .unwrap();

            print_detailed_report(&result, &mrz_data, &visual_data);
        }
        Err(err) => {
            eprintln!("Error validating passport: {}", err);
        }
    }
}
