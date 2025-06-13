//! This is the simplest image resizer I could make that met my needs.
//! It ONLY exists because imagemagik and such wouldn't take my bigger 100MB images
//!
use anyhow::{Context, Result};
use argh::FromArgs;
use image::imageops::FilterType;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Simple image resizer
#[derive(FromArgs)]
struct Args {
    /// input image path or directory
    #[argh(positional)]
    input: Vec<PathBuf>,

    /// resize dimensions (e.g., "500x400" or "20%")
    #[argh(option)]
    resize: String,

    /// output path (optional)
    #[argh(option, short = 'o')]
    output: Option<PathBuf>,

    /// overwrite files without prompting
    #[argh(switch, short = 'f')]
    force: bool,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    if args.input.is_empty() {
        return Err(anyhow::anyhow!("No input files specified"));
    }

    let mut image_files = Vec::new();
    for input in &args.input {
        if input.is_dir() {
            for entry in WalkDir::new(input)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if is_image_file(entry.path()) {
                    image_files.push(entry.path().to_path_buf());
                }
            }
        } else if input.to_str().unwrap_or("").contains('*') {
            for entry in glob::glob(input.to_str().unwrap())? {
                let path = entry?;
                if is_image_file(&path) {
                    image_files.push(path);
                }
            }
        } else if is_image_file(input) {
            image_files.push(input.clone());
        }
    }

    if image_files.is_empty() {
        return Err(anyhow::anyhow!("No valid image files found"));
    }

    for input_path in image_files {
        let output_path = match &args.output {
            Some(o) if o.is_dir() => o.join(input_path.file_name().unwrap()),
            Some(o) => o.clone(),
            None => input_path.clone(),
        };

        if output_path.exists() && !args.force {
            if !confirm_overwrite(&output_path)? {
                continue;
            }
        }

        resize_image(&input_path, &output_path, &args.resize)?;
        println!(
            "Processed: {} -> {}",
            input_path.display(),
            output_path.display()
        );
    }

    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp"
    )
}

fn confirm_overwrite(path: &Path) -> Result<bool> {
    println!(
        "Output file {} already exists. Overwrite? [y/N]",
        path.display()
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn resize_image(input_path: &Path, output_path: &Path, resize_arg: &str) -> Result<()> {
    // Load image
    let img = image::open(input_path)
        .with_context(|| format!("Failed to open image: {}", input_path.display()))?;

    // Parse resize argument
    let (width, height) = if resize_arg.ends_with('%') {
        // Percentage scaling
        let percent = resize_arg
            .trim_end_matches('%')
            .parse::<f32>()
            .with_context(|| format!("Invalid percentage: {}", resize_arg))?
            / 100.0;
        let (w, h) = (img.width() as f32 * percent, img.height() as f32 * percent);
        (w.round() as u32, h.round() as u32)
    } else {
        // Exact dimensions (format "WxH")
        let dims: Vec<&str> = resize_arg.split('x').collect();
        if dims.len() != 2 {
            return Err(anyhow::anyhow!(
                "Resize format must be either 'WxH' or 'N%'"
            ));
        }
        (
            dims[0]
                .parse()
                .with_context(|| format!("Invalid width: {}", dims[0]))?,
            dims[1]
                .parse()
                .with_context(|| format!("Invalid height: {}", dims[1]))?,
        )
    };

    // Resize image (Lanczos3 is high reasonably high quality)
    let resized = img.resize(width, height, FilterType::Lanczos3);

    resized
        .save(output_path)
        .with_context(|| format!("Failed to save image: {}", output_path.display()))?;

    Ok(())
}
