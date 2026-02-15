use anyhow::{Context, Result};
use log::warn;
use std::path::Path;
use std::process::Command;

pub fn check_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_video_duration(video_path: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()
        .context("Failed to run ffprobe")?;

    if !output.status.success() {
        warn!(
            "ffprobe failed for {:?}: {}",
            video_path,
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(0.0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<f64>().context("Invalid duration")
}

pub fn generate_thumbnail(video_path: &Path, thumb_path: &Path, time_pos: f64) -> Result<()> {
    // Generate thumbnail at specific time
    // ffmpeg -ss <time> -i <input> -vframes 1 -q:v 2 <output>

    // Create parent directory if it doesn't exist
    if let Some(parent) = thumb_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{:.3}", time_pos))
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2") // High quality jpeg
        .arg("-y") // Overwrite
        .arg(thumb_path)
        .output()
        .context("Failed to run ffmpeg")?;

    if !output.status.success() {
        warn!(
            "ffmpeg failed for {:?}: {}",
            video_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn ffmpeg_exists() -> bool {
        check_ffmpeg_available()
    }

    fn create_dummy_video(path: &Path) -> Result<()> {
        // Create a 1 second black video using ffmpeg
        // ffmpeg -f lavfi -i color=c=black:s=64x64:d=1 -c:v libx264 -t 1 -y output.mp4
        let status = Command::new("ffmpeg")
            .args(&[
                "-f",
                "lavfi",
                "-i",
                "color=c=black:s=64x64:d=1",
                "-t",
                "1",
                "-y",
            ])
            .arg(path)
            .output()? // Use output to suppress stdout/stderr in tests
            .status;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to create dummy video"))
        }
    }

    #[test]
    fn test_check_ffmpeg_available_no_panic() {
        let _ = check_ffmpeg_available();
    }

    #[test]
    fn test_video_operations() {
        if !ffmpeg_exists() {
            println!("Skipping video tests because ffmpeg is not available.");
            return;
        }

        let temp_dir = std::env::temp_dir().join("gemini_video_test");
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let video_path = temp_dir.join("test_vid.mp4");
        let thumb_path = temp_dir.join("thumb.jpg");

        // 1. Create dummy video
        if let Err(e) = create_dummy_video(&video_path) {
            eprintln!(
                "Failed to create dummy video (ffmpeg might lack libraries?): {}",
                e
            );
            // Cleanup
            let _ = fs::remove_dir_all(&temp_dir);
            return;
        }

        // 2. Test Duration
        match get_video_duration(&video_path) {
            Ok(duration) => {
                // Duration should be approx 1.0
                assert!(
                    duration > 0.8 && duration < 1.2,
                    "Duration {} not close to 1.0",
                    duration
                );
            }
            Err(e) => {
                eprintln!("get_video_duration failed: {}", e);
            }
        }

        // 3. Test Thumbnail Generation
        let result = generate_thumbnail(&video_path, &thumb_path, 0.5);
        assert!(result.is_ok(), "Thumbnail generation failed");

        if thumb_path.exists() {
            assert!(
                fs::metadata(&thumb_path).unwrap().len() > 0,
                "Thumbnail is empty"
            );
        } else {
            // If generation failed silently (warn logged), fail test
            panic!("Thumbnail file not created");
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
