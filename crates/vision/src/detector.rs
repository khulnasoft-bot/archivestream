use image::{DynamicImage, GenericImageView};
use img_hash::{HasherConfig, ImageHash};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDiff {
    pub similarity_score: f64,
    pub perceptual_hash_distance: u32,
    pub changed_regions: Vec<Region>,
    pub layout_shift_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub change_intensity: f64,
}

pub struct VisualChangeDetector {
    hasher: HasherConfig,
}

impl VisualChangeDetector {
    pub fn new() -> Self {
        Self {
            hasher: HasherConfig::new().hash_size(16, 16),
        }
    }

    /// Detect visual changes between two screenshots
    pub fn detect_changes(
        &self,
        img1: &DynamicImage,
        img2: &DynamicImage,
    ) -> anyhow::Result<VisualDiff> {
        // Compute perceptual hashes
        let hash1 = self.hasher.hash_image(img1);
        let hash2 = self.hasher.hash_image(img2);
        
        let hash_distance = hash1.dist(&hash2);
        
        // Calculate pixel-level similarity
        let similarity = self.calculate_similarity(img1, img2)?;
        
        // Detect changed regions
        let changed_regions = self.find_changed_regions(img1, img2)?;
        
        // Detect layout shifts (significant position changes)
        let layout_shift = self.detect_layout_shift(img1, img2)?;
        
        Ok(VisualDiff {
            similarity_score: similarity,
            perceptual_hash_distance: hash_distance,
            changed_regions,
            layout_shift_detected: layout_shift,
        })
    }

    /// Calculate overall similarity score (0.0 = identical, 1.0 = completely different)
    fn calculate_similarity(
        &self,
        img1: &DynamicImage,
        img2: &DynamicImage,
    ) -> anyhow::Result<f64> {
        if img1.dimensions() != img2.dimensions() {
            return Ok(1.0); // Completely different if dimensions don't match
        }

        let (width, height) = img1.dimensions();
        let mut diff_pixels = 0u64;
        let total_pixels = (width * height) as u64;

        for y in 0..height {
            for x in 0..width {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);
                
                // Simple RGB difference
                let diff = ((p1[0] as i32 - p2[0] as i32).abs()
                    + (p1[1] as i32 - p2[1] as i32).abs()
                    + (p1[2] as i32 - p2[2] as i32).abs()) as u32;
                
                if diff > 30 {
                    // Threshold for "different" pixel
                    diff_pixels += 1;
                }
            }
        }

        Ok(diff_pixels as f64 / total_pixels as f64)
    }

    /// Find rectangular regions with significant changes
    fn find_changed_regions(
        &self,
        img1: &DynamicImage,
        img2: &DynamicImage,
    ) -> anyhow::Result<Vec<Region>> {
        // Simplified implementation - divide into grid and detect changes
        let (width, height) = img1.dimensions();
        let grid_size = 50; // 50x50 pixel blocks
        let mut regions = Vec::new();

        for y in (0..height).step_by(grid_size) {
            for x in (0..width).step_by(grid_size) {
                let block_width = grid_size.min((width - x) as usize) as u32;
                let block_height = grid_size.min((height - y) as usize) as u32;
                
                let intensity = self.calculate_block_change(
                    img1, img2, x, y, block_width, block_height
                )?;
                
                if intensity > 0.1 {
                    // Significant change detected
                    regions.push(Region {
                        x,
                        y,
                        width: block_width,
                        height: block_height,
                        change_intensity: intensity,
                    });
                }
            }
        }

        Ok(regions)
    }

    fn calculate_block_change(
        &self,
        img1: &DynamicImage,
        img2: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<f64> {
        let mut diff_pixels = 0u64;
        let total_pixels = (width * height) as u64;

        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx;
                let py = y + dy;
                
                let p1 = img1.get_pixel(px, py);
                let p2 = img2.get_pixel(px, py);
                
                let diff = ((p1[0] as i32 - p2[0] as i32).abs()
                    + (p1[1] as i32 - p2[1] as i32).abs()
                    + (p1[2] as i32 - p2[2] as i32).abs()) as u32;
                
                if diff > 30 {
                    diff_pixels += 1;
                }
            }
        }

        Ok(diff_pixels as f64 / total_pixels as f64)
    }

    /// Detect significant layout shifts (e.g., elements moving position)
    fn detect_layout_shift(
        &self,
        img1: &DynamicImage,
        img2: &DynamicImage,
    ) -> anyhow::Result<bool> {
        // Simplified: Check if large blocks have moved
        // In production, use edge detection + feature matching
        let hash_distance = self.hasher.hash_image(img1).dist(&self.hasher.hash_image(img2));
        
        // If perceptual hash is similar but pixel diff is high, likely a layout shift
        Ok(hash_distance < 10 && self.calculate_similarity(img1, img2)? > 0.3)
    }
}

impl Default for VisualChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_identical_images() {
        let detector = VisualChangeDetector::new();
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        
        let diff = detector.detect_changes(&img, &img).unwrap();
        assert_eq!(diff.similarity_score, 0.0);
        assert_eq!(diff.perceptual_hash_distance, 0);
    }

    #[test]
    fn test_different_images() {
        let detector = VisualChangeDetector::new();
        let img1 = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let mut img2 = RgbaImage::new(100, 100);
        
        // Fill with white
        for pixel in img2.pixels_mut() {
            *pixel = image::Rgba([255, 255, 255, 255]);
        }
        let img2 = DynamicImage::ImageRgba8(img2);
        
        let diff = detector.detect_changes(&img1, &img2).unwrap();
        assert!(diff.similarity_score > 0.5);
    }
}
