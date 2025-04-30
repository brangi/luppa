# Passport OCR System Documentation

## Overview
This system is a robust, universal OCR solution for processing passport images, extracting and validating data according to ICAO Doc 9303 standards. It handles multiple languages, common OCR errors, and provides reliable field extraction and validation for developers and AI agents.

## Core Functionalities
- **Language-Agnostic Field Extraction**: Supports English, Spanish, French, German, and others; uses multi-language label detection with fallbacks.
- **Improved Text Extraction**: Includes fuzzy matching, text cleaning, and image preprocessing (grayscale, contrast, adaptive thresholding).
- **Universal Field Detection**: Extracts fields like place of birth using confidence scoring, handles multiple date formats with separator detection, and uses position/label-based methods.
- **Error Handling & Resilience**: Manages missing Tesseract files, provides graceful fallbacks, and merges results from multiple OCR configurations.
- **ML-Enhanced Validation**: Applies confidence scoring and ML-inspired heuristics for field validation.
- **PDF and Image Processing**: Uses the image crate for preprocessing; PDF extraction is stubbed for future implementation.
- **Batch Processing**: Processes multiple images with reporting on extraction quality and field completeness.

## Validations
- **MRZ Validation**: Checks format and consistency of machine-readable zones, including date fields with error normalization (e.g., invalid months/days adjusted to valid ranges).
- **Date Handling**: Improved parsing for birth and expiry dates, handling OCR noise (e.g., 'C'→'0') and determining century based on year values.
- **Security Checks**: Includes PKI, biometric, and feature detection to ensure document authenticity.

## Business Logic
- Ensures compliance with international standards for travel documents, reducing false positives/negatives in validation.
- Supports use cases like identity verification, border control, and data migration, with a focus on reliability and expandability for future features.

## Technical Stack
- Built in Rust using the luppa library for core validations.
- Depends on external crates for image processing and OCR (e.g., Tesseract via TESSDATA_PREFIX environment variable).

## How to Run
1. Ensure dependencies are installed (e.g., Tesseract OCR).
2. Set TESSDATA_PREFIX environment variable.
3. Compile and run the Rust application from the project root.

## Future Improvements
- Address code warnings for unused functions.
- Enhance testing with diverse passport formats and languages.
- Implement full PDF support and advanced ML validation.
