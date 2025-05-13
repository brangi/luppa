# Passport MRZ OCR and Validation System

## Overview
A Rust-based OCR solution for extracting and validating Machine Readable Zone (MRZ) data from passport images. The system focuses on accurate MRZ data extraction and validation according to ICAO Doc 9303 standards, with robust handling of common OCR errors.

## Core Functionality

### MRZ Processing
- Extracts MRZ data from passport images using Tesseract OCR
- Handles common OCR errors through character normalization
- Supports multiple MRZ formats (TD1, TD2, TD3)
- Cleans and normalizes MRZ data fields

### Date Handling
- Specialized parsing for MRZ date formats (YYMMDD)
- Handles common OCR confusions (e.g., 'C'→'0', 'E'→'3')
- Normalizes invalid date values to valid ranges
- Formats dates in human-readable format (DD/MM/YYYY)

### Validation
- Validates MRZ data structure and check digits
- Verifies document format compliance with ICAO Doc 9303
- Checks document expiry status
- Provides detailed validation results with specific issues

### Visual Data Extraction
- Extracts and processes visual inspection zone (VIZ) data
- Handles common OCR errors in personal information
- Supports multiple languages through Tesseract's language capabilities

## Technical Implementation
- Built in Rust for performance and safety
- Uses Tesseract OCR for text recognition
- Implements custom date parsing and validation logic
- Provides clear error handling and reporting

## Dependencies
- Tesseract OCR (v4.0.0+)
- Tesseract language data files (ocrb.traineddata)
- Rust toolchain (stable)

## Installation

1. Install Tesseract OCR:
   - macOS: `brew install tesseract`
   - Linux: `sudo apt-get install tesseract-ocr`

2. Ensure Tesseract data files are available:
   - The system will automatically check common locations including:
     - Homebrew installation directory
     - TESSDATA_PREFIX environment variable
     - Standard system locations (/usr/share/tessdata, /usr/local/share/tessdata)

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

The system processes passport images and validates the extracted MRZ data:

```rust
use std::path::Path;
use luppa::processing::{ImageProcessor, OcrProcessor};
use luppa::verification::MRTDVerifier;

let image_path = Path::new("path/to/passport.jpg");
let processed_image = ImageProcessor::process_image(image_path)?;
let mrz_data = OcrProcessor::extract_mrz(&processed_image)?;
let visual_data = OcrProcessor::extract_visual_data(&processed_image)?;
let result = MRTDVerifier::new().verify(&processed_image, &mrz_data, &visual_data)?;
```

## Known Limitations
- Primarily focused on MRZ data extraction and validation
- Limited support for non-Latin scripts
- No built-in support for PDF documents
- Requires Tesseract OCR to be installed separately
