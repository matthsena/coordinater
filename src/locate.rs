use image::{DynamicImage, GenericImageView, Rgba};
use xcap::Monitor;

pub struct LocateResult {
    pub x: i32,
    pub y: i32,
}

pub fn find_on_screen(
    monitor_id: u32,
    template_path: &str,
    threshold: f64,
) -> Result<LocateResult, String> {
    let monitors = Monitor::all()
        .map_err(|e| format!("failed to list monitors: {}", e))?;

    let monitor = monitors
        .into_iter()
        .find(|m| m.id().unwrap_or(0) == monitor_id)
        .ok_or_else(|| format!("monitor {} not found", monitor_id))?;

    let screenshot = monitor
        .capture_image()
        .map_err(|e| format!("failed to capture screen: {}", e))?;

    let screen = DynamicImage::ImageRgba8(screenshot);
    let template = image::open(template_path)
        .map_err(|e| format!("failed to open template image: {}", e))?;

    find_template(&screen, &template, threshold)
}

fn find_template(
    screen: &DynamicImage,
    template: &DynamicImage,
    threshold: f64,
) -> Result<LocateResult, String> {
    let (sw, sh) = screen.dimensions();
    let (tw, th) = template.dimensions();

    if tw > sw || th > sh {
        return Err("template is larger than screen".to_string());
    }

    let mut best_score = f64::MAX;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    let max_possible_diff = (tw * th) as f64 * 255.0 * 3.0;
    let max_allowed_diff = (1.0 - threshold) * max_possible_diff;

    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let mut diff_sum: f64 = 0.0;

            for ty in 0..th {
                for tx in 0..tw {
                    let Rgba(sp) = screen.get_pixel(x + tx, y + ty);
                    let Rgba(tp) = template.get_pixel(tx, ty);

                    diff_sum += (sp[0] as f64 - tp[0] as f64).abs();
                    diff_sum += (sp[1] as f64 - tp[1] as f64).abs();
                    diff_sum += (sp[2] as f64 - tp[2] as f64).abs();

                    if diff_sum > max_allowed_diff {
                        break;
                    }
                }
                if diff_sum > max_allowed_diff {
                    break;
                }
            }

            let similarity = 1.0 - (diff_sum / max_possible_diff);

            if similarity > threshold && diff_sum < best_score {
                best_score = diff_sum;
                best_x = x;
                best_y = y;

                if diff_sum == 0.0 {
                    return Ok(LocateResult {
                        x: (best_x + tw / 2) as i32,
                        y: (best_y + th / 2) as i32,
                    });
                }
            }
        }
    }

    if best_score == f64::MAX {
        return Err(format!(
            "could not find image on screen (threshold: {})",
            threshold
        ));
    }

    Ok(LocateResult {
        x: (best_x + tw / 2) as i32,
        y: (best_y + th / 2) as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage, Rgba};

    #[test]
    fn test_find_template_exact_match() {
        let mut screen = RgbaImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                screen.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        // Place a 10x10 red square at (30, 40)
        for y in 40..50 {
            for x in 30..40 {
                screen.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let mut template = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let screen = DynamicImage::ImageRgba8(screen);
        let template = DynamicImage::ImageRgba8(template);

        let result = find_template(&screen, &template, 0.8).unwrap();
        assert_eq!(result.x, 35); // center of 30..40
        assert_eq!(result.y, 45); // center of 40..50
    }

    #[test]
    fn test_find_template_not_found() {
        let screen = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let mut template = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                template.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let template = DynamicImage::ImageRgba8(template);

        let result = find_template(&screen, &template, 0.8);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_larger_than_screen() {
        let screen = DynamicImage::ImageRgba8(RgbaImage::new(5, 5));
        let template = DynamicImage::ImageRgba8(RgbaImage::new(10, 10));

        let result = find_template(&screen, &template, 0.8);
        assert!(result.is_err());
    }
}
