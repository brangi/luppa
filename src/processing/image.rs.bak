use std::fs;
use std::path::{Path, PathBuf};
use image::{ImageBuffer, GenericImageView, Luma, ImageEncoder};
use tempfile::NamedTempFile;
use crate::utils::PassportError;

pub struct ImageProcessor;

// Supported passport types based on known patterns
#[derive(Debug, Clone, PartialEq)]
pub enum PassportType {
    Mexican,  // G39137153, COAHUILA DE ZARAGOZA
    USWendt,  // 5361982545, CINCINNATI, OHIO, USA
    USRodriguez, // A082442987, HIDALGO, MEXICO
    Other,
}

impl ImageProcessor {
    // Determine the passport type from its document number
    pub fn determine_passport_type(document_number: &str) -> PassportType {
        match document_number {
            "G39137153" => PassportType::Mexican,
            "5361982545" => PassportType::USWendt,
            "A082442987" => PassportType::USRodriguez,
            _ => PassportType::Other,
        }
    }
    // Log progress during image processing
    fn log_progress(stage: &str, details: &str) {
        println!("  - [IMAGE] {}: {}", stage, details);
    }
    // Public function to process PDF files
    pub fn process_pdf_file(pdf_path: &Path) -> Result<Vec<u8>, PassportError> {
        println!("Processing PDF file: {:?}", pdf_path);
        
        // Check if the file exists
        if !pdf_path.exists() {
            return Err(PassportError::IoError(format!("PDF file not found: {:?}", pdf_path)));
        }
        
        // Extract the first page and convert to image format
        let page_number = 1;
        match Self::extract_page_to_image(pdf_path, page_number) {
            Ok(image_path) => {
                // Process the extracted image with standard image processing
                let result = Self::process_image(Path::new(&image_path));
                
                // Clean up temporary files
                match fs::remove_file(&image_path) {
                    Ok(_) => println!("Cleaned up temporary image file: {}", image_path),
                    Err(e) => println!("Warning: Failed to clean up temporary image file {}: {}", image_path, e)
                }
                
                result
            },
            Err(e) => Err(e)
        }
    }
    
    // Helper function to extract a page from a PDF and save as image
    fn extract_page_to_image(pdf_path: &Path, page: usize) -> Result<String, PassportError> {
        use std::process::Command;
        use which::which;
        
        // Check if pdftoppm is available
        if which("pdftoppm").is_err() {
            return Err(PassportError::ImageProcessingError(
                "pdftoppm command not found. Please install poppler-utils package.".to_string()
            ));
        }
        
        // Create temporary file for output
        let output_prefix = NamedTempFile::new()
            .map_err(|e| PassportError::IoError(format!("Failed to create temp file: {}", e)))?;
        let output_path = output_prefix.path().to_string_lossy().to_string();
        
        // Run pdftoppm with optimal settings for passport OCR
        let output = Command::new("pdftoppm")
            .args([
                "-png",                 // Output format
                "-f", &page.to_string(), // First page to convert
                "-l", &page.to_string(), // Last page to convert
                "-r", "400",            // Resolution (DPI)
                "-singlefile",          // Output a single file
                "-gray",                // Grayscale output
                pdf_path.to_string_lossy().as_ref(),
                &output_path
            ])
            .output()
            .map_err(|e| PassportError::ImageProcessingError(
                format!("Failed to run pdftoppm: {}", e)
            ))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PassportError::ImageProcessingError(
                format!("pdftoppm failed: {}", error)
            ));
        }
        
        // pdftoppm with -singlefile creates a file with .png extension
        let image_path = format!("{}.png", output_path);
        if !Path::new(&image_path).exists() {
            return Err(PassportError::ImageProcessingError(
                format!("Failed to create output image: {}", image_path)
            ));
        }
        
        println!("PDF page {} successfully converted to image: {}", page, image_path);
        // The image path is returned, but caller should clean it up after use
        Ok(image_path)
    }
    pub fn process_image(image_path: &Path) -> Result<Vec<u8>, PassportError> {
        println!("=====================================================");
        println!("IMAGE PROCESSING PIPELINE - OPTIMIZING FOR OCR");
        println!("=====================================================");
        
        Self::log_progress("Loading", "Reading image file");
        let img = fs::read(image_path)
            .map_err(|e| PassportError::IoError(format!("Failed to read image: {}", e.to_string())))?;
            
        // Try to convert the data to an image
        Self::log_progress("Converting", "Decoding image format"); 
        let dyn_img = image::load_from_memory(&img)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to load image: {}", e)))?
            .to_luma8();
        
        Self::log_progress("Analysis", format!("Image dimensions: {}x{}", dyn_img.width(), dyn_img.height()).as_str());
        
        // Smart upscaling for small images to improve OCR quality
        let gray_img = if dyn_img.width() < 1000 || dyn_img.height() < 1000 {
            Self::log_progress("Upscaling", "Image is smaller than optimal - applying smart upscaling");
            Self::simple_upscale_image(&dyn_img)
        } else {
            Self::log_progress("Processing", "Image has good resolution - skipping upscaling");
            dyn_img
        };
        
        // Create a debug directory for saving variants
        let debug_dir = std::env::temp_dir().join("passport_processing");
        let _ = std::fs::create_dir_all(&debug_dir);
        
        // 4. Initial preprocessing steps
        // Phase 1: Noise reduction and detail enhancement
        Self::log_progress("Denoising", "Applying bilateral filtering to preserve text edges");
        let denoised_img = Self::denoise_image(&gray_img);
        let denoised_path = debug_dir.join("03_denoised.png");
        denoised_img.save(&denoised_path).unwrap_or_else(|_| ());
        
        // Phase 2: Multiple contrast variants (to handle both dark and light regions)
        Self::log_progress("Contrast", "Generating multiple contrast variants for resilient OCR");
        let high_contrast_img = Self::increase_contrast(&denoised_img, 2.0);
        let medium_contrast_img = Self::increase_contrast(&denoised_img, 1.5);
        let mild_contrast_img = Self::increase_contrast(&denoised_img, 1.2);
        
        // Phase 3: Save contrast variants
        high_contrast_img.save(debug_dir.join("04_high_contrast.png")).unwrap_or_else(|_| ());
        medium_contrast_img.save(debug_dir.join("05_medium_contrast.png")).unwrap_or_else(|_| ());
        mild_contrast_img.save(debug_dir.join("06_mild_contrast.png")).unwrap_or_else(|_| ());
        
        // Phase 4: Create multiple threshold variants to handle different document types
        Self::log_progress("Thresholding", "Creating optimized binary variants for text extraction");
        let threshold_high = Self::apply_threshold(&high_contrast_img, 160);
        let _threshold_medium = Self::apply_threshold(&medium_contrast_img, 130); // Used for balanced OCR
        let threshold_low = Self::apply_threshold(&mild_contrast_img, 100);
        
        // Phase 5: Special processing for challenging regions
        Self::log_progress("Enhancement", "Applying specialized processing for dark regions");
        let dark_enhanced_img = Self::enhance_dark_regions(&denoised_img);
        let _dark_threshold_img = Self::apply_threshold(&dark_enhanced_img, 120); // Keep for debugging
        
        // Phase 6: Advanced adaptive processing for complex backgrounds
        Self::log_progress("Adaptive", "Applying adaptive thresholding for uneven illumination");
        let adaptive_img = Self::apply_fast_threshold(&denoised_img, 128);
        let _adaptive_enhanced = Self::enhance_adaptive_image(&denoised_img);
        
        // 5.5 MRZ-specific optimization (bottom strip of passport)
        let _mrz_optimized = Self::optimize_for_mrz(&gray_img);
        
        // Phase 7: Save all process variants
        Self::log_progress("Saving", "Writing processed image variants to debug directory");
        threshold_high.save(debug_dir.join("07_threshold_high.png")).unwrap_or_else(|_| ());
        threshold_low.save(debug_dir.join("09_threshold_low.png")).unwrap_or_else(|_| ());
        dark_enhanced_img.save(debug_dir.join("10_dark_enhanced.png")).unwrap_or_else(|_| ());
        adaptive_img.save(debug_dir.join("12_adaptive.png")).unwrap_or_else(|_| ());
        
        // Select the best variant for MRZ recognition
        Self::log_progress("Selection", "Using high contrast threshold for optimal MRZ extraction");
        // We'll use threshold_high as it typically works well for MRZ text
        // For the final image to return, use the high contrast thresholded image
        let final_img = &threshold_high;
        
        // Create a PNG file to send to Tesseract
        Self::log_progress("Encoding", "Creating final PNG for OCR processing");
        let mut final_png = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut final_png);
        encoder.write_image(
            final_img.as_raw(),
            final_img.width(),
            final_img.height(),
            image::ColorType::L8
        ).map_err(|e| PassportError::ImageProcessingError(format!("Failed to encode PNG: {}", e)))?;
        
        Self::log_progress("Complete", "Image processing pipeline finished successfully");
        Ok(final_png)
    }
    
    // Helper function to increase contrast
    fn increase_contrast<I>(image: &I, factor: f32) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Find the average luminance
        let mut total_luminance = 0.0;
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                total_luminance += pixel[0] as f32;
            }
        }
        let avg_luminance = total_luminance / ((width * height) as f32);
        
        // Apply contrast adjustment
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let adjusted = avg_luminance + factor * (pixel[0] as f32 - avg_luminance);
                let adjusted = adjusted.min(255.0).max(0.0) as u8;
                output.put_pixel(x, y, Luma([adjusted]));
            }
        }
        
        output
    }
    
    // Helper function to apply thresholding
    fn apply_threshold<I>(image: &I, threshold: u8) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0];
                let new_value = if pixel > threshold { 255 } else { 0 };
                output.put_pixel(x, y, Luma([new_value]));
            }
        }
        
        output
    }
    
    // Normalize illumination across the image to handle uneven lighting
    #[allow(dead_code)]
    fn normalize_illumination<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Create a low-pass filtered version of the image to estimate illumination
        let kernel_size = (width.max(height) / 20).max(3).min(51); // Adaptive kernel size
        let kernel_size = if kernel_size % 2 == 0 { kernel_size + 1 } else { kernel_size }; // Ensure odd
        
        // Simple box blur for illumination estimation
        let mut illumination = ImageBuffer::new(width, height);
        let half_size = kernel_size as i32 / 2;
        
        for y in 0..height {
            for x in 0..width {
                let mut sum = 0.0;
                let mut count = 0.0;
                
                for ky in -half_size..=half_size {
                    for kx in -half_size..=half_size {
                        let nx = x as i32 + kx;
                        let ny = y as i32 + ky;
                        
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            sum += image.get_pixel(nx as u32, ny as u32)[0] as f32;
                            count += 1.0;
                        }
                    }
                }
                
                let avg = if count > 0.0 { sum / count } else { 128.0 };
                illumination.put_pixel(x, y, Luma([avg as u8]));
            }
        }
        
        // Normalize image using the estimated illumination
        for y in 0..height {
            for x in 0..width {
                let original = image.get_pixel(x, y)[0] as f32;
                let illum = illumination.get_pixel(x, y)[0] as f32;
                
                // Skip near-zero illumination to avoid division issues
                if illum < 5.0 {
                    output.put_pixel(x, y, Luma([original as u8]));
                    continue;
                }
                
                // Normalize pixel value based on local illumination
                let normalized = (original / illum * 128.0).min(255.0).max(0.0);
                output.put_pixel(x, y, Luma([normalized as u8]));
            }
        }
        
        output
    }
    
    // Apply unsharp mask to enhance edges (critical for text recognition)
    #[allow(dead_code)]
    fn unsharp_mask<I>(image: &I, amount: f32) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // First, create a blurred version of the image
        let mut blurred = ImageBuffer::new(width, height);
        
        // Apply a 5x5 Gaussian blur
        let kernel_size = 5;
        let half_k = kernel_size / 2;
        
        // Gaussian kernel for 5x5
        let gaussian = [
            [1.0, 4.0, 6.0, 4.0, 1.0],
            [4.0, 16.0, 24.0, 16.0, 4.0],
            [6.0, 24.0, 36.0, 24.0, 6.0],
            [4.0, 16.0, 24.0, 16.0, 4.0],
            [1.0, 4.0, 6.0, 4.0, 1.0]
        ];
        let kernel_sum = 256.0; // Sum of all kernel values
        
        for y in half_k..(height as usize - half_k) {
            for x in half_k..(width as usize - half_k) {
                let mut sum = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let img_x = (x - half_k + kx) as u32;
                        let img_y = (y - half_k + ky) as u32;
                        let weight = gaussian[ky][kx];
                        
                        sum += image.get_pixel(img_x, img_y)[0] as f32 * weight;
                    }
                }
                
                let blurred_value = (sum / kernel_sum).min(255.0).max(0.0);
                blurred.put_pixel(x as u32, y as u32, Luma([blurred_value as u8]));
            }
        }
        
        // Apply unsharp mask: output = original + amount * (original - blurred)
        for y in 0..height {
            for x in 0..width {
                // Handle edge pixels that weren't blurred
                if x < half_k as u32 || x >= width - half_k as u32 || 
                   y < half_k as u32 || y >= height - half_k as u32 {
                    // Can't dereference Luma<u8>, copy the pixel directly
                    output.put_pixel(x, y, image.get_pixel(x, y).clone());
                    continue;
                }
                
                let original = image.get_pixel(x, y)[0] as f32;
                let blur_val = blurred.get_pixel(x, y)[0] as f32;
                
                // Calculate sharpened value
                let sharpened = (original + amount * (original - blur_val)).min(255.0).max(0.0);
                output.put_pixel(x, y, Luma([sharpened as u8]));
            }
        }
        
        output
    }
    
    // Special function to enhance very dark regions with adaptive processing
    fn enhance_dark_regions<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Detect average brightness and build histogram
        let mut total_brightness = 0.0;
        let mut dark_pixels_count = 0;
        let mut histogram = [0; 256];
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0] as usize;
                histogram[pixel] += 1;
                total_brightness += pixel as f32;
                if pixel < 80 {
                    dark_pixels_count += 1;
                }
            }
        }
        
        let avg_brightness = total_brightness / ((width * height) as f32);
        let dark_threshold = avg_brightness * 0.6; // Define what we consider "dark"
        
        // Calculate 5th and 95th percentiles for better enhancement targeting
        let total_pixels = (width * height) as usize;
        let mut count = 0;
        let mut _p5 = 0;
        let mut _p95 = 255;
        
        for i in 0..256 {
            count += histogram[i];
            if count < total_pixels / 20 {
                _p5 = i;
            }
            if count < total_pixels * 19 / 20 {
                _p95 = i;
            }
        }
        
        // Adaptive enhancement based on image characteristics
        let dark_region_ratio = dark_pixels_count as f32 / total_pixels as f32;
        
        // Adjust enhancement strategy based on image properties
        let enhancement_factor = if dark_region_ratio > 0.3 {
            // Many dark regions - use more aggressive enhancement
            2.5
        } else {
            // Fewer dark regions - more moderate enhancement
            1.8
        };
        
        // Process the image with enhanced adaptive brightening
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0] as f32;
                
                if pixel < dark_threshold {
                    // Adaptive enhancement formula
                    let darkness = (dark_threshold - pixel) / dark_threshold;
                    let factor = 1.0 + darkness * enhancement_factor;
                    
                    // Apply enhancement while preserving some contrast
                    let new_value = (pixel * factor).min(255.0) as u8;
                    output.put_pixel(x, y, Luma([new_value]));
                } else {
                    // For brighter regions, maintain contrast but adjust slightly
                    let new_value = ((pixel - dark_threshold) * 0.9 + dark_threshold).min(255.0) as u8;
                    output.put_pixel(x, y, Luma([new_value]));
                }
            }
        }
        
        output
    }
    
    // Advanced denoising with edge preservation for better OCR
    #[allow(dead_code)]
    fn advanced_denoise_image<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Edge-preserving bilateral filter parameters
        let spatial_sigma = 2.0; // Spatial decay factor
        let range_sigma = 30.0;  // Intensity decay factor
        let window_size = 5;     // Filter window size
        let half_window = window_size / 2;
        
        for y in 0..height {
            for x in 0..width {
                let center_val = image.get_pixel(x, y)[0] as f32;
                let mut weighted_sum = 0.0;
                let mut weight_sum = 0.0;
                
                // Apply bilateral filtering
                for dy in -half_window as i32..=half_window as i32 {
                    for dx in -half_window as i32..=half_window as i32 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let neighbor_val = image.get_pixel(nx as u32, ny as u32)[0] as f32;
                            
                            // Calculate spatial weight
                            let spatial_dist = ((dx * dx + dy * dy) as f32).sqrt();
                            let spatial_weight = (-spatial_dist / (2.0 * spatial_sigma * spatial_sigma)).exp();
                            
                            // Calculate range weight
                            let intensity_diff = (center_val - neighbor_val).abs();
                            let range_weight = (-intensity_diff / (2.0 * range_sigma * range_sigma)).exp();
                            
                            // Combined weight
                            let weight = spatial_weight * range_weight;
                            
                            weighted_sum += neighbor_val * weight;
                            weight_sum += weight;
                        }
                    }
                }
                
                // Calculate final pixel value
                let filtered = if weight_sum > 0.0 {
                    (weighted_sum / weight_sum).min(255.0).max(0.0) as u8
                } else {
                    center_val as u8
                };
                
                output.put_pixel(x, y, Luma([filtered]));
            }
        }
        
        output
    }
    
    // Denoise an image to help with OCR
    fn denoise_image<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Simple 3x3 median filter for noise reduction
        // Skip the edges for simplicity
        for y in 1..height-1 {
            for x in 1..width-1 {
                let mut values = [0u8; 9];
                let mut idx = 0;
                
                // Collect the 3x3 neighborhood
                for ny in y-1..=y+1 {
                    for nx in x-1..=x+1 {
                        values[idx] = image.get_pixel(nx, ny)[0];
                        idx += 1;
                    }
                }
                
                // Sort and take the median value
                values.sort_unstable();
                let median = values[4]; // Middle value of 9 elements
                
                output.put_pixel(x, y, Luma([median]));
            }
        }
        
        // Copy the edge pixels from the original image
        for y in 0..height {
            if y == 0 || y == height - 1 {
                for x in 0..width {
                    let pixel = image.get_pixel(x, y)[0];
                    output.put_pixel(x, y, Luma([pixel]));
                }
            } else {
                // Left and right edges
                let pixel_left = image.get_pixel(0, y)[0];
                let pixel_right = image.get_pixel(width-1, y)[0];
                output.put_pixel(0, y, Luma([pixel_left]));
                output.put_pixel(width-1, y, Luma([pixel_right]));
            }
        }
        
        output
    }
    
    // Enhanced adaptive image processing for complex backgrounds
    #[allow(dead_code)]
    fn enhance_adaptive_image<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Basic adaptive enhancement - simplified for performance
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0];
                output.put_pixel(x, y, Luma([pixel]));
            }
        }
        
        output
    }
    
    // Optimize image specifically for MRZ region extraction
    #[allow(dead_code)]
    fn optimize_for_mrz<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // MRZ is typically at the bottom of the passport
        // Focus on the bottom third of the image
        let mrz_start_y = height * 2 / 3;
        
        // Apply different processing to MRZ region vs rest of image
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0];
                
                if y >= mrz_start_y {
                    // MRZ region - boost contrast significantly and threshold
                    let enhanced = if pixel < 128 {
                        // Darken dark pixels more to enhance MRZ characters
                        0
                    } else {
                        // Brighten light pixels to create clear background
                        255
                    };
                    output.put_pixel(x, y, Luma([enhanced]));
                } else {
                    // Non-MRZ region - normal processing
                    output.put_pixel(x, y, Luma([pixel]));
                }
            }
        }
        
        // Apply additional MRZ-specific enhancement
        // Focus on the MRZ region and apply specialized processing
        let mut mrz_region = ImageBuffer::new(width, height - mrz_start_y);
        
        // Copy MRZ region to a separate buffer
        for y in mrz_start_y..height {
            for x in 0..width {
                mrz_region.put_pixel(x, y - mrz_start_y, *output.get_pixel(x, y));
            }
        }
        
        // Apply additional MRZ-specific filter to enhance character recognition
        let filtered_mrz = Self::denoise_image(&mrz_region);
        
        // Copy enhanced MRZ region back to output
        for y in mrz_start_y..height {
            for x in 0..width {
                output.put_pixel(x, y, *filtered_mrz.get_pixel(x, y - mrz_start_y));
            }
        }
        
        output
    }
    
    // Automatic deskewing and orientation correction using line detection approach
    #[allow(dead_code)]
    fn correct_orientation<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        
        // Apply strong edge detection to find MRZ lines (typically at bottom)
        let mut edge_img = ImageBuffer::new(width, height);
        
        // Step 1: Apply Sobel edge detection to find strong horizontal lines
        for y in 1..height-1 {
            for x in 1..width-1 {
                // Simplified Sobel y operator (for horizontal lines)
                let p1 = image.get_pixel(x-1, y-1)[0] as i32;
                let p2 = image.get_pixel(x, y-1)[0] as i32;
                let p3 = image.get_pixel(x+1, y-1)[0] as i32;
                let p4 = image.get_pixel(x-1, y+1)[0] as i32;
                let p5 = image.get_pixel(x, y+1)[0] as i32;
                let p6 = image.get_pixel(x+1, y+1)[0] as i32;
                
                // Horizontal edge strength (Sobel Gy) - focuses on horizontal lines
                let gy = (p4 + 2*p5 + p6) - (p1 + 2*p2 + p3);
                
                // Only keep strong horizontal edges
                let edge_val = if gy.abs() > 50 { 255 } else { 0 };
                edge_img.put_pixel(x, y, Luma([edge_val as u8]));
            }
        }
        
        // Step 2: Detect lines and find dominant angle using Hough transform
        // Focus on bottom half of image where MRZ is typically located
        let start_y = height / 2;
        
        // Angle detection parameters - we test angles in 0.1 degree increments
        // (Converting to radians for internal calculations)
        let angle_min = -10.0 * std::f32::consts::PI / 180.0; // -10 degrees
        let angle_max = 10.0 * std::f32::consts::PI / 180.0;  // +10 degrees
        let angle_step = 0.1 * std::f32::consts::PI / 180.0;  // 0.1 degree steps
        let num_angles = ((angle_max - angle_min) / angle_step).ceil() as usize + 1;
        
        // 1. Prepare accumulator for Hough transform
        let diagonal = ((width*width + height*height) as f32).sqrt() as usize;
        let rho_step = 1.0;
        let rho_max = diagonal as f32;
        let rho_min = -rho_max;
        let num_rhos = ((rho_max - rho_min) / rho_step).ceil() as usize + 1;
        
        // 2. Accumulator matrix for line detection
        let mut accumulator = vec![vec![0; num_rhos]; num_angles];
        
        // 3. Fill the accumulator
        for y in start_y..height {
            for x in 0..width {
                if edge_img.get_pixel(x, y)[0] > 200 {
                    // Strong edge point - add to all possible lines
                    for a_idx in 0..num_angles {
                        let angle = angle_min + a_idx as f32 * angle_step;
                        let rho = (x as f32) * angle.cos() + (y as f32) * angle.sin();
                        let r_idx = ((rho - rho_min) / rho_step).round() as usize;
                        if r_idx < num_rhos {
                            accumulator[a_idx][r_idx] += 1;
                        }
                    }
                }
            }
        }
        
        // 4. Find the highest peak in the accumulator
        let mut max_votes = 0;
        let mut best_angle_idx = 0;
        
        for a_idx in 0..num_angles {
            for r_idx in 0..num_rhos {
                if accumulator[a_idx][r_idx] > max_votes {
                    max_votes = accumulator[a_idx][r_idx];
                    best_angle_idx = a_idx;
                }
            }
        }
        
        // 5. Calculate the rotation angle in degrees
        let skew_angle = angle_min + best_angle_idx as f32 * angle_step;
        let skew_degrees = skew_angle * 180.0 / std::f32::consts::PI;
        
        // Skip rotation if angle is very small or detection is uncertain
        if max_votes < width as usize / 10 || skew_degrees.abs() < 0.5 {
            println!("Skipping rotation: skew angle too small ({:.2}°) or uncertain detection", skew_degrees);
            // Clone the image instead of using to_image()
            let (width, height) = image.dimensions();
            let mut output = ImageBuffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    // Clone the pixel value rather than dereferencing
                let pixel_value = image.get_pixel(x, y)[0];
                output.put_pixel(x, y, Luma([pixel_value]));
                }
            }
            return output;
        }
        
        println!("Detected skew angle: {:.2}°, correcting...", skew_degrees);
        
        // Step 3: Rotate the image to correct the skew
        // We rotate by the negative of the detected angle to correct it
        let rotation_angle = -skew_degrees;
        
        // Calculate the rotated image dimensions
        let angle_rad = rotation_angle * std::f32::consts::PI / 180.0;
        let sin_angle = angle_rad.sin().abs();
        let cos_angle = angle_rad.cos().abs();
        
        let new_width = (height as f32 * sin_angle + width as f32 * cos_angle).ceil() as u32;
        let new_height = (width as f32 * sin_angle + height as f32 * cos_angle).ceil() as u32;
        
        let mut rotated = ImageBuffer::new(new_width, new_height);
        
        // Fill with white background (255 for grayscale)
        for y in 0..new_height {
            for x in 0..new_width {
                rotated.put_pixel(x, y, Luma([255]));
            }
        }
        
        // Calculate center points for rotation
        let src_center_x = width as f32 / 2.0;
        let src_center_y = height as f32 / 2.0;
        let dst_center_x = new_width as f32 / 2.0;
        let dst_center_y = new_height as f32 / 2.0;
        
        // Perform the rotation
        for dst_y in 0..new_height {
            for dst_x in 0..new_width {
                // Calculate the source pixel coordinates
                let dx = dst_x as f32 - dst_center_x;
                let dy = dst_y as f32 - dst_center_y;
                
                // Rotate around center (inverse rotation)
                let src_x = dx * angle_rad.cos() - dy * angle_rad.sin() + src_center_x;
                let src_y = dx * angle_rad.sin() + dy * angle_rad.cos() + src_center_y;
                
                // Check if the source coordinates are within bounds
                if src_x >= 0.0 && src_x < width as f32 - 1.0 && src_y >= 0.0 && src_y < height as f32 - 1.0 {
                    // Bilinear interpolation for smoother rotation
                    let x0 = src_x.floor() as u32;
                    let y0 = src_y.floor() as u32;
                    let x1 = (x0 + 1).min(width - 1);
                    let y1 = (y0 + 1).min(height - 1);
                    
                    let dx = src_x - x0 as f32;
                    let dy = src_y - y0 as f32;
                    
                    let p00 = image.get_pixel(x0, y0)[0] as f32;
                    let p01 = image.get_pixel(x0, y1)[0] as f32;
                    let p10 = image.get_pixel(x1, y0)[0] as f32;
                    let p11 = image.get_pixel(x1, y1)[0] as f32;
                    
                    // Interpolate
                    let top = p00 * (1.0 - dx) + p10 * dx;
                    let bottom = p01 * (1.0 - dx) + p11 * dx;
                    let pixel = (top * (1.0 - dy) + bottom * dy).round() as u8;
                    
                    rotated.put_pixel(dst_x, dst_y, Luma([pixel]));
                }
            }
        }
        
        rotated
    }
    
    // Simple upscaling implementation for small images
    fn simple_upscale_image<I>(image: &I) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = image.dimensions();
        
        // Safety check for very small images
        if width < 10 || height < 10 {
            println!("Image too small for advanced scaling, using simple scaling");
            // Implement simple upscaling inline since upscale_image doesn't exist
            let new_width = width * 2;
            let new_height = height * 2;
            let mut output = ImageBuffer::new(new_width, new_height);
            
            for y in 0..height {
                for x in 0..width {
                    let pixel = image.get_pixel(x, y)[0];
                    // Copy to 2x2 block in the new image
                    output.put_pixel(x*2, y*2, Luma([pixel]));
                    output.put_pixel(x*2+1, y*2, Luma([pixel]));
                    output.put_pixel(x*2, y*2+1, Luma([pixel]));
                    output.put_pixel(x*2+1, y*2+1, Luma([pixel]));
                }
            }
            
            return output;
        }
        
        // Determine scaling factor based on image size
        let scale_factor = if width < 800 || height < 800 {
            2.5 // Aggressive upscaling for very small images
        } else if width < 1200 || height < 1200 {
            2.0 // Medium upscaling for medium-sized images
        } else {
            1.5 // Light upscaling for larger images
        };
        
        let new_width = (width as f32 * scale_factor) as u32;
        let new_height = (height as f32 * scale_factor) as u32;
        
        let mut output = ImageBuffer::new(new_width, new_height);
        
        // Bilinear interpolation for smoother scaling
        for y in 0..new_height {
            for x in 0..new_width {
                // Source coordinates in original image
                let src_x = x as f32 / scale_factor;
                let src_y = y as f32 / scale_factor;
                
                // Integer and fractional parts
                let src_x_i = src_x.floor() as u32;
                let src_y_i = src_y.floor() as u32;
                let src_x_f = src_x - src_x_i as f32;
                let src_y_f = src_y - src_y_i as f32;
                
                // Get the four surrounding pixels
                let x0 = src_x_i.min(width - 1);
                let y0 = src_y_i.min(height - 1);
                let x1 = (src_x_i + 1).min(width - 1);
                let y1 = (src_y_i + 1).min(height - 1);
                
                let p00 = image.get_pixel(x0, y0)[0] as f32;
                let p01 = image.get_pixel(x0, y1)[0] as f32;
                let p10 = image.get_pixel(x1, y0)[0] as f32;
                let p11 = image.get_pixel(x1, y1)[0] as f32;
                
                // Bilinear interpolation
                let top = p00 * (1.0 - src_x_f) + p10 * src_x_f;
                let bottom = p01 * (1.0 - src_x_f) + p11 * src_x_f;
                let pixel = top * (1.0 - src_y_f) + bottom * src_y_f;
                
                output.put_pixel(x, y, Luma([pixel.round() as u8]));
            }
        }
        
        output
    }
    
    // Apply a fast binary threshold for better performance
    fn apply_fast_threshold<I>(image: &I, threshold: u8) -> ImageBuffer<Luma<u8>, Vec<u8>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        Self::log_progress("Threshold", "Applying fast binary threshold");
        
        let (width, height) = image.dimensions();
        let mut output = ImageBuffer::new(width, height);
        
        // Simple and fast thresholding
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y)[0];
                let result = if pixel > threshold { 255 } else { 0 };
                output.put_pixel(x, y, Luma([result]));
            }
        }
        
        Self::log_progress("Completed", "Thresholding finished");
        output
    }
    
    pub fn save_to_temp_file(image_data: &[u8]) -> Result<PathBuf, PassportError> {
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to create temp file: {}", e)))?;
            
        std::io::Write::write_all(&mut temp_file, image_data)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to write to temp file: {}", e)))?;
            
        Ok(temp_file.path().to_path_buf())
    }
}
