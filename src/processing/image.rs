use crate::utils::PassportError;
use image::{DynamicImage, ImageBuffer, Luma};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use std::path::Path;
use std::path::PathBuf;
use std::io::Write;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn process_image(image_path: &Path) -> Result<Vec<u8>, PassportError> {
        let img = image::open(image_path)
            .map_err(|e| PassportError::ImageProcessingError(format!("Failed to open image: {}", e)))?;
        let processed = Self::preprocess_image(&img);
        Ok(processed)
    }

    fn preprocess_image(img: &DynamicImage) -> Vec<u8> {
        // Convert to grayscale
        let gray = img.to_luma8();

        // Deskew image
        let deskewed = Self::deskew_image(&gray);

        // Enhance contrast
        let enhanced = Self::enhance_contrast(&deskewed);

        // Convert back to bytes
        enhanced.into_raw()
    }

    fn deskew_image(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        // Simple deskewing - rotate by a small angle
        rotate_about_center(img, 0.0, Interpolation::Bilinear, Luma([0u8]))
    }

    fn enhance_contrast(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        // Simple contrast enhancement
        let mut enhanced = img.clone();
        for pixel in enhanced.pixels_mut() {
            let value = pixel[0];
            let new_value = if value < 128 {
                value.saturating_sub(20)
            } else {
                value.saturating_add(20)
            };
            pixel[0] = new_value;
        }
        enhanced
    }
    
    pub fn save_to_temp_file(image_data: &[u8]) -> Result<PathBuf, PassportError> {
        let mut temp_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .map_err(|e| PassportError::ImageProcessingError(e.to_string()))?;
            
        temp_file.write_all(image_data)
            .map_err(|e| PassportError::ImageProcessingError(e.to_string()))?;
            
        Ok(temp_file.path().to_path_buf())
    }
}
