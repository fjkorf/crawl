//! Screenshot regression tests.
//!
//! # Workflow
//!
//! Generate all baselines:
//!   UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored
//!
//! Run regression tests:
//!   cargo test --test screenshot_test

use std::path::Path;
use std::process::Command;

const CHANNEL_THRESHOLD: u8 = 30;
const MAX_DIFF_PERCENT: f64 = 2.0;

// --- Helpers ---

fn run_static_capture(output_path: &str) {
    let status = Command::new("cargo")
        .args(["run", "--example", "screenshot_test"])
        .env("SCREENSHOT_OUTPUT", output_path)
        .status()
        .expect("failed to run screenshot_test example");
    assert!(status.success(), "static capture failed: {}", status);
}

fn run_walkthrough_capture(output_dir: &str) {
    let status = Command::new("cargo")
        .args(["run", "--example", "screenshot_test"])
        .env("SCREENSHOT_MODE", "walkthrough")
        .env("SCREENSHOT_DIR", output_dir)
        .status()
        .expect("failed to run walkthrough");
    assert!(status.success(), "walkthrough capture failed: {}", status);
}

fn count_different_pixels(a: &image::RgbaImage, b: &image::RgbaImage, threshold: u8) -> u64 {
    assert_eq!(a.dimensions(), b.dimensions(), "image dimensions must match");
    a.pixels()
        .zip(b.pixels())
        .filter(|(pa, pb)| {
            pa.0.iter()
                .zip(pb.0.iter())
                .any(|(ca, cb)| ca.abs_diff(*cb) > threshold)
        })
        .count() as u64
}

fn assert_screenshots_match(baseline_path: &str, new_path: &str) {
    let baseline = Path::new(baseline_path);
    if !baseline.exists() {
        panic!(
            "Baseline not found: '{}'. Generate with: \
             UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored",
            baseline_path
        );
    }

    let baseline_img = image::open(baseline_path).unwrap().to_rgba8();
    let current_img = image::open(new_path).unwrap().to_rgba8();

    assert_eq!(
        baseline_img.dimensions(),
        current_img.dimensions(),
        "dimensions changed for {}",
        baseline_path,
    );

    let diff_count = count_different_pixels(&baseline_img, &current_img, CHANNEL_THRESHOLD);
    let total = baseline_img.width() as u64 * baseline_img.height() as u64;
    let diff_pct = diff_count as f64 / total as f64 * 100.0;

    if diff_pct < MAX_DIFF_PERCENT {
        std::fs::remove_file(new_path).ok();
    } else {
        // Generate diff image for debugging
        let mut diff_img = image::RgbaImage::new(baseline_img.width(), baseline_img.height());
        for (x, y, pixel) in diff_img.enumerate_pixels_mut() {
            let pa = baseline_img.get_pixel(x, y);
            let pb = current_img.get_pixel(x, y);
            let differs = pa.0.iter().zip(pb.0.iter()).any(|(ca, cb)| ca.abs_diff(*cb) > CHANNEL_THRESHOLD);
            *pixel = if differs {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([pa.0[0] / 3, pa.0[1] / 3, pa.0[2] / 3, 255])
            };
        }
        let diff_path = format!("{}.diff.png", new_path.trim_end_matches(".new.png"));
        diff_img.save(&diff_path).ok();

        panic!(
            "{} differs by {:.2}% ({}/{} pixels)\n  new:  {}\n  diff: {}",
            baseline_path, diff_pct, diff_count, total, new_path, diff_path
        );
    }
}

// --- Static screenshot test ---

#[test]
fn dungeon_scene_matches_baseline() {
    let new_path = "tests/snapshots/dungeon_scene.new.png";
    run_static_capture(new_path);
    assert_screenshots_match("tests/snapshots/dungeon_scene.png", new_path);
}

// --- Walkthrough test: verifies all 4 rooms ---

#[test]
fn walkthrough_all_rooms() {
    let dir = "tests/snapshots/walkthrough_new";
    std::fs::create_dir_all(dir).ok();

    run_walkthrough_capture(dir);

    for room in ["room1", "room2", "room3", "room4"] {
        let baseline = format!("tests/snapshots/{}.png", room);
        let new_file = format!("{}/{}.png", dir, room);
        assert!(
            Path::new(&new_file).exists(),
            "walkthrough did not produce {}",
            new_file
        );
        assert_screenshots_match(&baseline, &new_file);
    }

    std::fs::remove_dir_all(dir).ok();
}

// --- Baseline generation (run with UPDATE_SNAPSHOTS=true) ---

#[test]
#[ignore]
fn update_all_baselines() {
    if std::env::var("UPDATE_SNAPSHOTS").is_err() {
        panic!("Set UPDATE_SNAPSHOTS=true to generate/update baselines");
    }

    // Static baseline
    run_static_capture("tests/snapshots/dungeon_scene.png");
    assert!(Path::new("tests/snapshots/dungeon_scene.png").exists());
    println!("Updated: tests/snapshots/dungeon_scene.png");

    // Walkthrough baselines
    run_walkthrough_capture("tests/snapshots");
    for room in ["room1", "room2", "room3", "room4"] {
        let path = format!("tests/snapshots/{}.png", room);
        assert!(Path::new(&path).exists(), "missing {}", path);
        println!("Updated: {}", path);
    }
}
