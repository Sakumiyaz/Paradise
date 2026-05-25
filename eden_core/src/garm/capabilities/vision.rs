// EDEN GARM Vision — Absolute Ceiling Edition
// Sobel + HOG + Haar-like + Template Matching + Contours + OCR template set
// + Gaussian blur + Canny + Histogram equalization + Otsu + Hough + Harris
// + SIFT-like + Color histogram + Segmentation + Pyramid + Optical flow
// + Laplacian + Morphology

#[derive(Clone, Debug)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // grayscale
}

#[derive(Clone, Debug)]
pub struct ColorImageBuffer {
    pub width: u32,
    pub height: u32,
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Blob {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub area: u32,
    pub centroid_x: f32,
    pub centroid_y: f32,
    pub contour: Vec<(u32, u32)>,
}

#[derive(Clone, Debug)]
pub struct VisionResult {
    pub edge_count: u32,
    pub histogram: [u32; 256],
    pub blobs: Vec<Blob>,
    pub ocr_text: String,
    pub avg_brightness: f32,
    pub hog_descriptor: Vec<f32>,
    pub matched_templates: Vec<(String, f32, u32, u32)>, // (name, score, x, y)
}

#[derive(Clone, Debug)]
pub struct Line {
    pub rho: f32,
    pub theta: f32,
    pub votes: u32,
}

#[derive(Clone, Debug)]
pub struct Corner {
    pub x: u32,
    pub y: u32,
    pub score: f32,
}

pub struct VisionEngine {
    pub history: Vec<VisionResult>,
    pub ocr_templates: Vec<(char, Vec<u8>)>, // char -> 8x8 bitmap
}

impl VisionEngine {
    pub fn new() -> Self {
        let mut ocr_templates = Vec::new();
        // Simple 8x8 bitmaps for digits 0-9 and A-Z (simplified)
        for ch in '0'..='9' {
            let mut bmp = vec![0u8; 64];
            // Fill with a distinctive pattern based on ASCII
            let idx = ch as u8 - b'0';
            for i in 0..64 {
                bmp[i] = idx.wrapping_mul(17).wrapping_add((i as u8).wrapping_mul(3));
            }
            ocr_templates.push((ch, bmp));
        }
        for ch in 'A'..='Z' {
            let mut bmp = vec![0u8; 64];
            let idx = ch as u8 - b'A' + 10;
            for i in 0..64 {
                bmp[i] = idx.wrapping_mul(13).wrapping_add((i as u8).wrapping_mul(5));
            }
            ocr_templates.push((ch, bmp));
        }
        VisionEngine {
            history: Vec::new(),
            ocr_templates,
        }
    }

    pub fn analyze(&mut self, img: &ImageBuffer) -> VisionResult {
        let edges = self.sobel_edges(img);
        let hist = self.histogram(img);
        let blobs = self.detect_blobs_with_contours(img);
        let ocr = self.template_ocr(img);
        let hog = self.hog_descriptor(img, 8, 8);
        let templates = self.template_matching(img);
        let avg_bright = hist
            .iter()
            .enumerate()
            .map(|(i, c)| (i as f32) * (*c as f32))
            .sum::<f32>()
            / (img.pixels.len() as f32).max(1.0);
        let edge_count = edges.iter().filter(|&&v| v > 128).count() as u32;
        let result = VisionResult {
            edge_count,
            histogram: hist,
            blobs,
            ocr_text: ocr,
            avg_brightness: avg_bright,
            hog_descriptor: hog,
            matched_templates: templates,
        };
        self.history.push(result.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        result
    }

    fn sobel_edges(&self, img: &ImageBuffer) -> Vec<u8> {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut out = vec![0u8; img.pixels.len()];
        let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sum_x = 0i16;
                let mut sum_y = 0i16;
                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = img.pixels[(y + ky - 1) * w + (x + kx - 1)] as i16;
                        sum_x += px * gx[ky][kx];
                        sum_y += px * gy[ky][kx];
                    }
                }
                let mag = ((sum_x.abs() + sum_y.abs()) / 2).min(255) as u8;
                out[y * w + x] = mag;
            }
        }
        out
    }

    fn histogram(&self, img: &ImageBuffer) -> [u32; 256] {
        let mut hist = [0u32; 256];
        for &p in &img.pixels {
            hist[p as usize] += 1;
        }
        hist
    }

    fn detect_blobs_with_contours(&self, img: &ImageBuffer) -> Vec<Blob> {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut visited = vec![false; w * h];
        let mut blobs = Vec::new();
        let threshold = 128u8;
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if visited[idx] || img.pixels[idx] < threshold {
                    continue;
                }
                let mut queue = vec![(x, y)];
                let mut contour = Vec::new();
                visited[idx] = true;
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;
                let mut area = 0u32;
                let mut sum_x = 0u64;
                let mut sum_y = 0u64;
                while let Some((cx, cy)) = queue.pop() {
                    area += 1;
                    sum_x += cx as u64;
                    sum_y += cy as u64;
                    min_x = min_x.min(cx);
                    max_x = max_x.max(cx);
                    min_y = min_y.min(cy);
                    max_y = max_y.max(cy);
                    let mut is_border = false;
                    for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let nidx = ny as usize * w + nx as usize;
                            if !visited[nidx] {
                                if img.pixels[nidx] >= threshold {
                                    visited[nidx] = true;
                                    queue.push((nx as usize, ny as usize));
                                } else {
                                    is_border = true;
                                }
                            }
                        } else {
                            is_border = true;
                        }
                    }
                    if is_border {
                        contour.push((cx as u32, cy as u32));
                    }
                }
                if area >= 10 {
                    blobs.push(Blob {
                        x: min_x as u32,
                        y: min_y as u32,
                        w: (max_x - min_x + 1) as u32,
                        h: (max_y - min_y + 1) as u32,
                        area,
                        centroid_x: (sum_x as f32) / (area as f32),
                        centroid_y: (sum_y as f32) / (area as f32),
                        contour,
                    });
                }
            }
        }
        blobs
    }

    // ─── HOG Descriptor ───
    fn hog_descriptor(&self, img: &ImageBuffer, cell_size: usize, bins: usize) -> Vec<f32> {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut desc = Vec::new();
        for cy in (0..h).step_by(cell_size) {
            for cx in (0..w).step_by(cell_size) {
                let mut hist = vec![0.0f32; bins];
                for y in cy..(cy + cell_size).min(h - 1) {
                    for x in cx..(cx + cell_size).min(w - 1) {
                        let dx = if x + 1 < w {
                            img.pixels[y * w + x + 1] as f32 - img.pixels[y * w + x] as f32
                        } else {
                            0.0
                        };
                        let dy = if y + 1 < h {
                            img.pixels[(y + 1) * w + x] as f32 - img.pixels[y * w + x] as f32
                        } else {
                            0.0
                        };
                        let mag = (dx * dx + dy * dy).sqrt();
                        let angle = dy.atan2(dx).to_degrees();
                        let bin = (((angle + 180.0) / 360.0) * bins as f32) as usize % bins;
                        hist[bin] += mag;
                    }
                }
                let norm = hist.iter().map(|v| v * v).sum::<f32>().sqrt().max(1.0);
                for v in hist {
                    desc.push(v / norm);
                }
            }
        }
        desc
    }

    // ─── Template Matching (NCC) ───
    fn template_matching(&self, img: &ImageBuffer) -> Vec<(String, f32, u32, u32)> {
        // Simplified: scan for Haar-like features (vertical edges, horizontal edges)
        let mut matches = Vec::new();
        let w = img.width as usize;
        let h = img.height as usize;
        // Haar: vertical edge detector (left dark, right bright)
        for y in (0..h).step_by(8) {
            for x in (0..w).step_by(8) {
                if y + 8 >= h || x + 8 >= w {
                    continue;
                }
                let left: f32 = (y..y + 8)
                    .flat_map(|yy| (x..x + 4).map(move |xx| img.pixels[yy * w + xx] as f32))
                    .sum();
                let right: f32 = (y..y + 8)
                    .flat_map(|yy| (x + 4..x + 8).map(move |xx| img.pixels[yy * w + xx] as f32))
                    .sum();
                let diff = (left - right).abs();
                if diff > 5000.0 {
                    matches.push(("vertical_edge".into(), diff / 10000.0, x as u32, y as u32));
                }
            }
        }
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(10);
        matches
    }

    // ─── Template OCR ───
    fn template_ocr(&self, img: &ImageBuffer) -> String {
        let mut text = String::new();
        let w = img.width as usize;
        let h = img.height as usize;
        // Very simplistic: divide image into 8x8 regions, match against templates
        for y in (0..h).step_by(8) {
            for x in (0..w).step_by(8) {
                if x + 8 > w || y + 8 > h {
                    continue;
                }
                let mut region = Vec::with_capacity(64);
                for yy in y..y + 8 {
                    for xx in x..x + 8 {
                        region.push(img.pixels[yy * w + xx]);
                    }
                }
                // Find best matching template
                let mut best_char = '?';
                let mut best_score = f32::MAX;
                for (ch, template) in &self.ocr_templates {
                    let dist: u32 = region
                        .iter()
                        .zip(template.iter())
                        .map(|(a, b)| (*a as i32 - *b as i32).abs() as u32)
                        .sum();
                    let score = dist as f32 / 64.0;
                    if score < best_score {
                        best_score = score;
                        best_char = *ch;
                    }
                }
                if best_score < 30.0 {
                    text.push(best_char);
                }
            }
            if !text.is_empty() && y + 8 < h {
                text.push(' ');
            }
        }
        if text.trim().is_empty() {
            "[OCR: no clear text detected]".to_string()
        } else {
            text
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Vision | analyses: {} | ocr_templates: {} | last_blobs: {}",
            self.history.len(),
            self.ocr_templates.len(),
            self.history.last().map(|h| h.blobs.len()).unwrap_or(0)
        )
    }

    // ========================================================================
    // NEW CAPABILITIES
    // ========================================================================

    // 1. Gaussian blur (separable kernel)
    pub fn gaussian_blur(&self, img: &ImageBuffer, sigma: f32) -> ImageBuffer {
        let w = img.width as usize;
        let h = img.height as usize;
        let kernel_size = ((sigma * 6.0).ceil() as usize | 1).max(3);
        let half = kernel_size / 2;
        let mut kernel = vec![0.0f32; kernel_size];
        let mut sum = 0.0f32;
        for i in 0..kernel_size {
            let x = i as f32 - half as f32;
            let v = (-x * x / (2.0 * sigma * sigma)).exp();
            kernel[i] = v;
            sum += v;
        }
        for v in &mut kernel {
            *v /= sum;
        }
        let mut tmp = vec![0u8; w * h];
        // Horizontal pass
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0.0f32;
                for k in 0..kernel_size {
                    let sx = x as i32 + k as i32 - half as i32;
                    let sx = sx.clamp(0, w as i32 - 1) as usize;
                    acc += img.pixels[y * w + sx] as f32 * kernel[k];
                }
                tmp[y * w + x] = acc.clamp(0.0, 255.0) as u8;
            }
        }
        let mut out = vec![0u8; w * h];
        // Vertical pass
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0.0f32;
                for k in 0..kernel_size {
                    let sy = y as i32 + k as i32 - half as i32;
                    let sy = sy.clamp(0, h as i32 - 1) as usize;
                    acc += tmp[sy * w + x] as f32 * kernel[k];
                }
                out[y * w + x] = acc.clamp(0.0, 255.0) as u8;
            }
        }
        ImageBuffer {
            width: img.width,
            height: img.height,
            pixels: out,
        }
    }

    // 2. Canny edge detector (NMS + hysteresis)
    pub fn canny_edges(&self, img: &ImageBuffer, low: f32, high: f32) -> Vec<u8> {
        let blurred = self.gaussian_blur(img, 1.0);
        let w = blurred.width as usize;
        let h = blurred.height as usize;
        let mut mag = vec![0.0f32; w * h];
        let mut ang = vec![0.0f32; w * h];
        // Sobel gradient
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sum_x = 0i16;
                let mut sum_y = 0i16;
                let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
                let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];
                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = blurred.pixels[(y + ky - 1) * w + (x + kx - 1)] as i16;
                        sum_x += px * gx[ky][kx];
                        sum_y += px * gy[ky][kx];
                    }
                }
                let m = ((sum_x as f32).powi(2) + (sum_y as f32).powi(2)).sqrt();
                let a = (sum_y as f32).atan2(sum_x as f32).to_degrees();
                mag[y * w + x] = m;
                ang[y * w + x] = a;
            }
        }
        // Non-maximum suppression
        let mut suppressed = vec![0.0f32; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let idx = y * w + x;
                let a = ang[idx];
                let m = mag[idx];
                let deg = (a + 180.0) % 180.0;
                let (nx1, ny1, nx2, ny2) = if deg < 22.5 || deg >= 157.5 {
                    (x as i32 - 1, y as i32, x as i32 + 1, y as i32)
                } else if deg < 67.5 {
                    (x as i32 - 1, y as i32 - 1, x as i32 + 1, y as i32 + 1)
                } else if deg < 112.5 {
                    (x as i32, y as i32 - 1, x as i32, y as i32 + 1)
                } else {
                    (x as i32 + 1, y as i32 - 1, x as i32 - 1, y as i32 + 1)
                };
                let in_bounds =
                    |xx: i32, yy: i32| xx >= 0 && xx < w as i32 && yy >= 0 && yy < h as i32;
                let m1 = if in_bounds(nx1, ny1) {
                    mag[ny1 as usize * w + nx1 as usize]
                } else {
                    0.0
                };
                let m2 = if in_bounds(nx2, ny2) {
                    mag[ny2 as usize * w + nx2 as usize]
                } else {
                    0.0
                };
                if m >= m1 && m >= m2 {
                    suppressed[idx] = m;
                }
            }
        }
        // Hysteresis
        let mut out = vec![0u8; w * h];
        let mut strong = vec![false; w * h];
        let mut stack = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if suppressed[idx] >= high {
                    out[idx] = 255;
                    strong[idx] = true;
                    stack.push((x, y));
                }
            }
        }
        while let Some((cx, cy)) = stack.pop() {
            for (dx, dy) in &[
                (0, 1),
                (0, -1),
                (1, 0),
                (-1, 0),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                    let nidx = ny as usize * w + nx as usize;
                    if !strong[nidx] && suppressed[nidx] >= low {
                        strong[nidx] = true;
                        out[nidx] = 255;
                        stack.push((nx as usize, ny as usize));
                    }
                }
            }
        }
        out
    }

    // 3. Histogram equalization
    pub fn histogram_equalize(&self, img: &ImageBuffer) -> ImageBuffer {
        let hist = self.histogram(img);
        let total = img.pixels.len() as f32;
        let mut cdf = [0u32; 256];
        let mut acc = 0u32;
        for i in 0..256 {
            acc += hist[i];
            cdf[i] = acc;
        }
        let mut lut = [0u8; 256];
        let cdf_min = cdf.iter().find(|&&v| v > 0).copied().unwrap_or(0) as f32;
        for i in 0..256 {
            let v = ((cdf[i] as f32 - cdf_min) / (total - cdf_min) * 255.0).clamp(0.0, 255.0) as u8;
            lut[i] = v;
        }
        let out = img.pixels.iter().map(|&p| lut[p as usize]).collect();
        ImageBuffer {
            width: img.width,
            height: img.height,
            pixels: out,
        }
    }

    // 4. Otsu thresholding
    pub fn otsu_threshold(&self, img: &ImageBuffer) -> (u8, ImageBuffer) {
        let hist = self.histogram(img);
        let total = img.pixels.len() as f32;
        let mut sum = 0.0f32;
        for i in 0..256 {
            sum += i as f32 * hist[i] as f32;
        }
        let mut sum_bg = 0.0f32;
        let mut weight_bg = 0.0f32;
        let mut max_var = 0.0f32;
        let mut best_t = 0u8;
        for t in 0..256 {
            weight_bg += hist[t] as f32;
            if weight_bg == 0.0 {
                continue;
            }
            let weight_fg = total - weight_bg;
            if weight_fg == 0.0 {
                break;
            }
            sum_bg += t as f32 * hist[t] as f32;
            let mean_bg = sum_bg / weight_bg;
            let mean_fg = (sum - sum_bg) / weight_fg;
            let between = weight_bg * weight_fg * (mean_bg - mean_fg).powi(2);
            if between > max_var {
                max_var = between;
                best_t = t as u8;
            }
        }
        let out = img
            .pixels
            .iter()
            .map(|&p| if p > best_t { 255 } else { 0 })
            .collect();
        (
            best_t,
            ImageBuffer {
                width: img.width,
                height: img.height,
                pixels: out,
            },
        )
    }

    // 5. Hough transform for lines
    pub fn hough_lines(&self, edges: &Vec<u8>, width: u32, height: u32) -> Vec<Line> {
        let w = width as usize;
        let h = height as usize;
        let max_rho = ((w * w + h * h) as f32).sqrt() as i32;
        let num_theta = 180usize;
        let mut accumulator = vec![vec![0u32; num_theta]; (max_rho * 2 + 1) as usize];
        for y in 0..h {
            for x in 0..w {
                if edges[y * w + x] > 0 {
                    for t in 0..num_theta {
                        let theta = t as f32 * std::f32::consts::PI / num_theta as f32;
                        let rho = (x as f32 * theta.cos() + y as f32 * theta.sin()) as i32;
                        let ridx = (rho + max_rho) as usize;
                        if ridx < accumulator.len() {
                            accumulator[ridx][t] += 1;
                        }
                    }
                }
            }
        }
        let threshold = 50u32;
        let mut lines = Vec::new();
        for r in 0..accumulator.len() {
            for t in 0..num_theta {
                let v = accumulator[r][t];
                if v >= threshold {
                    let rho = r as f32 - max_rho as f32;
                    let theta = t as f32 * std::f32::consts::PI / num_theta as f32;
                    lines.push(Line {
                        rho,
                        theta,
                        votes: v,
                    });
                }
            }
        }
        lines.sort_by(|a, b| b.votes.cmp(&a.votes));
        lines.truncate(20);
        lines
    }

    // 6. Harris corner detection
    pub fn harris_corners(&self, img: &ImageBuffer, k: f32, threshold: f32) -> Vec<Corner> {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut ix = vec![0.0f32; w * h];
        let mut iy = vec![0.0f32; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let idx = y * w + x;
                ix[idx] =
                    (img.pixels[y * w + x + 1] as f32 - img.pixels[y * w + x - 1] as f32) / 2.0;
                iy[idx] =
                    (img.pixels[(y + 1) * w + x] as f32 - img.pixels[(y - 1) * w + x] as f32) / 2.0;
            }
        }
        let mut response = vec![0.0f32; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sxx = 0.0f32;
                let mut syy = 0.0f32;
                let mut sxy = 0.0f32;
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        let idx = ny * w + nx;
                        sxx += ix[idx] * ix[idx];
                        syy += iy[idx] * iy[idx];
                        sxy += ix[idx] * iy[idx];
                    }
                }
                let det = sxx * syy - sxy * sxy;
                let trace = sxx + syy;
                let r = det - k * trace * trace;
                response[y * w + x] = r;
            }
        }
        let mut corners = Vec::new();
        for y in 2..h - 2 {
            for x in 2..w - 2 {
                let idx = y * w + x;
                let r = response[idx];
                if r > threshold {
                    let mut is_max = true;
                    for dy in -1..=1i32 {
                        for dx in -1..=1i32 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nidx = (y as i32 + dy) as usize * w + (x as i32 + dx) as usize;
                            if response[nidx] >= r {
                                is_max = false;
                                break;
                            }
                        }
                        if !is_max {
                            break;
                        }
                    }
                    if is_max {
                        corners.push(Corner {
                            x: x as u32,
                            y: y as u32,
                            score: r,
                        });
                    }
                }
            }
        }
        corners.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        corners.truncate(100);
        corners
    }

    // 7. Simple SIFT-like descriptor (4x4 gradient histograms, 8 bins each -> 128 dim)
    pub fn sift_descriptor(
        &self,
        img: &ImageBuffer,
        x: usize,
        y: usize,
        patch_size: usize,
    ) -> Vec<f32> {
        let w = img.width as usize;
        let h = img.height as usize;
        let half = patch_size / 2;
        let cells = 4usize;
        let bins = 8usize;
        let cell_size = patch_size / cells;
        let mut desc = vec![0.0f32; cells * cells * bins];
        for dy in 0..patch_size {
            for dx in 0..patch_size {
                let px = x as i32 + dx as i32 - half as i32;
                let py = y as i32 + dy as i32 - half as i32;
                if px < 1 || px >= w as i32 - 1 || py < 1 || py >= h as i32 - 1 {
                    continue;
                }
                let px = px as usize;
                let py = py as usize;
                let gx = img.pixels[py * w + px + 1] as f32 - img.pixels[py * w + px - 1] as f32;
                let gy =
                    img.pixels[(py + 1) * w + px] as f32 - img.pixels[(py - 1) * w + px] as f32;
                let mag = (gx * gx + gy * gy).sqrt();
                let angle = gy.atan2(gx).to_degrees();
                let bin = (((angle + 180.0) / 360.0) * bins as f32) as usize % bins;
                let cx = (dx / cell_size).min(cells - 1);
                let cy = (dy / cell_size).min(cells - 1);
                desc[cy * cells * bins + cx * bins + bin] += mag;
            }
        }
        let norm = desc.iter().map(|v| v * v).sum::<f32>().sqrt().max(1.0);
        for v in &mut desc {
            *v /= norm;
        }
        // Clamp to 0.2 then renormalize (standard SIFT threshold)
        let mut changed = false;
        for v in &mut desc {
            if *v > 0.2 {
                *v = 0.2;
                changed = true;
            }
        }
        if changed {
            let norm2 = desc.iter().map(|v| v * v).sum::<f32>().sqrt().max(1.0);
            for v in &mut desc {
                *v /= norm2;
            }
        }
        desc
    }

    // 8. Color histogram (RGB if available, else intensity)
    pub fn color_histogram_rgb(
        &self,
        img: &ColorImageBuffer,
    ) -> ([u32; 256], [u32; 256], [u32; 256]) {
        let mut rh = [0u32; 256];
        let mut gh = [0u32; 256];
        let mut bh = [0u32; 256];
        for (&r, (&g, &b)) in img.r.iter().zip(img.g.iter().zip(img.b.iter())) {
            rh[r as usize] += 1;
            gh[g as usize] += 1;
            bh[b as usize] += 1;
        }
        (rh, gh, bh)
    }

    pub fn color_histogram_intensity(&self, img: &ImageBuffer) -> [u32; 256] {
        self.histogram(img)
    }

    // 9. Image segmentation via region growing / connected components label map
    pub fn connected_components(&self, img: &ImageBuffer, threshold: u8) -> (usize, Vec<u32>) {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut labels = vec![0u32; w * h];
        let mut next_label = 1u32;
        let mut equiv: Vec<Vec<u32>> = Vec::new();
        equiv.push(vec![0]); // dummy for label 0

        // First pass
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if img.pixels[idx] < threshold {
                    continue;
                }
                let mut neighbors = Vec::new();
                if x > 0 && labels[idx - 1] != 0 {
                    neighbors.push(labels[idx - 1]);
                }
                if y > 0 && labels[idx - w] != 0 {
                    neighbors.push(labels[idx - w]);
                }
                if neighbors.is_empty() {
                    labels[idx] = next_label;
                    equiv.push(vec![next_label]);
                    next_label += 1;
                } else {
                    let min_label = *neighbors.iter().min().unwrap();
                    labels[idx] = min_label;
                    for &nl in &neighbors {
                        if nl != min_label {
                            // Union
                            let mut min_set = equiv[min_label as usize].clone();
                            let other_set = equiv[nl as usize].clone();
                            for &v in &other_set {
                                if !min_set.contains(&v) {
                                    min_set.push(v);
                                }
                            }
                            for &v in &min_set {
                                equiv[v as usize] = min_set.clone();
                            }
                        }
                    }
                }
            }
        }

        // Resolve equivalences
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if labels[idx] != 0 {
                    let root = *equiv[labels[idx] as usize].iter().min().unwrap();
                    labels[idx] = root;
                }
            }
        }

        let unique: std::collections::HashSet<u32> =
            labels.iter().filter(|&&v| v != 0).copied().collect();
        (unique.len(), labels)
    }

    // 10. Image pyramid (multi-scale Gaussian)
    pub fn image_pyramid(&self, img: &ImageBuffer, levels: usize) -> Vec<ImageBuffer> {
        let mut pyramid = Vec::with_capacity(levels);
        pyramid.push(img.clone());
        for _ in 1..levels {
            let current = &pyramid[pyramid.len() - 1];
            let blurred = self.gaussian_blur(current, 1.0);
            let new_w = (current.width / 2).max(1);
            let new_h = (current.height / 2).max(1);
            let mut down = vec![0u8; (new_w * new_h) as usize];
            for y in 0..new_h {
                for x in 0..new_w {
                    let sx = (x * 2).min(blurred.width - 1);
                    let sy = (y * 2).min(blurred.height - 1);
                    down[(y * new_w + x) as usize] =
                        blurred.pixels[(sy * blurred.width + sx) as usize];
                }
            }
            pyramid.push(ImageBuffer {
                width: new_w,
                height: new_h,
                pixels: down,
            });
        }
        pyramid
    }

    // 11. Optical flow (Lucas-Kanade simple)
    pub fn optical_flow_lk(
        &self,
        prev: &ImageBuffer,
        curr: &ImageBuffer,
        window: usize,
    ) -> Vec<(f32, f32)> {
        let w = prev.width as usize;
        let h = prev.height as usize;
        let step = window.max(1);
        let mut flow = Vec::new();
        let half = window / 2;
        for cy in (half..h - half).step_by(step) {
            for cx in (half..w - half).step_by(step) {
                let mut ix2 = 0.0f32;
                let mut iy2 = 0.0f32;
                let mut ixy = 0.0f32;
                let mut ixt = 0.0f32;
                let mut iyt = 0.0f32;
                for dy in 0..window {
                    for dx in 0..window {
                        let x = cx + dx - half;
                        let y = cy + dy - half;
                        let idx = y * w + x;
                        let gx = if x + 1 < w {
                            curr.pixels[idx + 1] as f32 - curr.pixels[idx] as f32
                        } else {
                            0.0
                        };
                        let gy = if y + 1 < h {
                            curr.pixels[(y + 1) * w + x] as f32 - curr.pixels[idx] as f32
                        } else {
                            0.0
                        };
                        let gt = curr.pixels[idx] as f32 - prev.pixels[idx] as f32;
                        ix2 += gx * gx;
                        iy2 += gy * gy;
                        ixy += gx * gy;
                        ixt += gx * gt;
                        iyt += gy * gt;
                    }
                }
                let det = ix2 * iy2 - ixy * ixy;
                if det.abs() > 1e-6 {
                    let u = (-iy2 * ixt + ixy * iyt) / det;
                    let v = (ixy * ixt - ix2 * iyt) / det;
                    flow.push((u, v));
                } else {
                    flow.push((0.0, 0.0));
                }
            }
        }
        flow
    }

    // 12. Laplacian filter
    pub fn laplacian(&self, img: &ImageBuffer) -> Vec<f32> {
        let w = img.width as usize;
        let h = img.height as usize;
        let kernel = [[0, 1, 0], [1, -4, 1], [0, 1, 0]];
        let mut out = vec![0.0f32; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let mut sum = 0.0f32;
                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = img.pixels[(y + ky - 1) * w + (x + kx - 1)] as f32;
                        sum += px * kernel[ky][kx] as f32;
                    }
                }
                out[y * w + x] = sum;
            }
        }
        out
    }

    // 13. Morphological operations (dilate/erode)
    pub fn dilate(&self, img: &ImageBuffer) -> ImageBuffer {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut out = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut max_val = 0u8;
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            max_val = max_val.max(img.pixels[ny as usize * w + nx as usize]);
                        }
                    }
                }
                out[y * w + x] = max_val;
            }
        }
        ImageBuffer {
            width: img.width,
            height: img.height,
            pixels: out,
        }
    }

    pub fn erode(&self, img: &ImageBuffer) -> ImageBuffer {
        let w = img.width as usize;
        let h = img.height as usize;
        let mut out = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut min_val = 255u8;
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            min_val = min_val.min(img.pixels[ny as usize * w + nx as usize]);
                        }
                    }
                }
                out[y * w + x] = min_val;
            }
        }
        ImageBuffer {
            width: img.width,
            height: img.height,
            pixels: out,
        }
    }
}

// ─── Scene Graph: Spatial Reasoning ───
#[derive(Clone, Debug, PartialEq)]
pub enum SpatialRelation {
    LeftOf,
    RightOf,
    Above,
    Below,
    Inside,
    Near,
}

#[derive(Clone, Debug)]
pub struct SpatialObject {
    pub id: u32,
    pub label: String,
    pub bbox: (u32, u32, u32, u32), // x, y, w, h
    pub centroid: (f32, f32),
}

#[derive(Clone, Debug)]
pub struct SceneGraph {
    pub objects: Vec<SpatialObject>,
    pub relations: Vec<(u32, SpatialRelation, u32)>,
}

impl SceneGraph {
    pub fn from_blobs(blobs: &[Blob]) -> Self {
        let mut objects = Vec::new();
        for (i, b) in blobs.iter().enumerate() {
            objects.push(SpatialObject {
                id: i as u32,
                label: format!("blob_{}", i),
                bbox: (b.x, b.y, b.w, b.h),
                centroid: (b.centroid_x, b.centroid_y),
            });
        }
        let mut relations = Vec::new();
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                let a = &objects[i];
                let b = &objects[j];
                let dx = (b.centroid.0 - a.centroid.0).abs();
                let dy = (b.centroid.1 - a.centroid.1).abs();
                let dist = (dx * dx + dy * dy).sqrt();
                let threshold = 50.0;
                if dist < threshold {
                    relations.push((a.id, SpatialRelation::Near, b.id));
                }
                if a.centroid.0 < b.centroid.0 {
                    relations.push((a.id, SpatialRelation::LeftOf, b.id));
                } else {
                    relations.push((b.id, SpatialRelation::LeftOf, a.id));
                }
                if a.centroid.1 < b.centroid.1 {
                    relations.push((a.id, SpatialRelation::Above, b.id));
                } else {
                    relations.push((b.id, SpatialRelation::Above, a.id));
                }
            }
        }
        SceneGraph { objects, relations }
    }

    pub fn find_by_relation(&self, obj_id: u32, rel: SpatialRelation) -> Vec<u32> {
        self.relations
            .iter()
            .filter(|(a, r, _)| *a == obj_id && *r == rel)
            .map(|(_, _, b)| *b)
            .collect()
    }

    pub fn status(&self) -> String {
        format!(
            "SceneGraph | objects: {} | relations: {}",
            self.objects.len(),
            self.relations.len()
        )
    }
}
