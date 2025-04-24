use std::fs;
use std::path::{Path, PathBuf};
use image::{DynamicImage, ImageBuffer, GenericImageView, Luma};
use tempfile::NamedTempFile;
use crate::utils::PassportError;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn process_image(image_path: &Path) -> Result<Vec<u8>, PassportError> {
        // Load the image
        let img = image::open(image_path)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to open image: {}", e)))?;
            
        println!("Image loaded: {}x{}", img.width(), img.height());
        
        // Convert to grayscale and get the image buffer
        let gray_img = img.to_luma8();
        
        // Increase contrast
        let contrast_img = Self::increase_contrast(&gray_img, 1.5);
        
        // Apply thresholding
        let threshold_img = Self::apply_threshold(&contrast_img, 150);
        
        println!("Image processing completed");
        
        // Create a temporary file to store the processed image
        let temp_file = NamedTempFile::new()
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to create temp file: {}", e)))?;
        let temp_path = temp_file.path().to_path_buf();
        
        // Save the processed image with explicit format (PNG)
        let dynamic_img = DynamicImage::ImageLuma8(threshold_img);
        dynamic_img.save_with_format(&temp_path, image::ImageFormat::Png)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to save processed image: {}", e)))?;
            
        // Read the bytes back
        let image_bytes = fs::read(&temp_path)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to read processed image: {}", e)))?;
            
        Ok(image_bytes)
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
                let pixel = image.get_pixel(x, y);
                let value = if pixel[0] > threshold { 255 } else { 0 };
                output.put_pixel(x, y, Luma([value]));
            }
        }
        
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
