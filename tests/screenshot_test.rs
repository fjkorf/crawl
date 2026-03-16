//! Screenshot regression test: captures a screenshot of the dungeon scene
//! and compares it against a baseline PNG.
//!
//! # Workflow
//!
//! Generate baseline (first time or after intentional visual changes):
//!   UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored
//!
//! Run the comparison test:
//!   cargo test --test screenshot_test
//!
//! Manual capture for inspection:
//!   cargo run --example screenshot_test

use std::path::Path;
use std::process::Command;

const BASELINE_PATH: &str = "tests/snapshots/dungeon_scene.png";
const NEW_PATH: &str = "tests/snapshots/dungeon_scene.new.png";

/// Per-channel threshold: two pixels are "same" if all RGBA channels
/// differ by at most this value (0–255 scale).
const CHANNEL_THRESHOLD: u8 = 30;

/// Maximum percentage of pixels allowed to differ before the test fails.
const MAX_DIFF_PERCENT: f64 = 2.0;

fn run_screenshot_capture(output_path: &str) {
    let status = Command::new("cargo")
        .args(["run", "--example", "screenshot_test"])
        .env("SCREENSHOT_OUTPUT", output_path)
        .status()
        .expect("failed to run screenshot_test example");
    assert!(
        status.success(),
        "screenshot_test example exited with: {}",
        status
    );
}

fn count_different_pixels(
    a: &image::RgbaImage,
    b: &image::RgbaImage,
    threshold: u8,
) -> u64 {
    assert_eq!(a.dimensions(), b.dimensions(), "image dimensions must match");
    let mut diff = 0u64;
    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let channel_diff = pa
            .0
            .iter()
            .zip(pb.0.iter())
            .any(|(ca, cb)| ca.abs_diff(*cb) > threshold);
        if channel_diff {
            diff += 1;
        }
    }
    diff
}

#[test]
fn dungeon_scene_matches_baseline() {
    let baseline = Path::new(BASELINE_PATH);
    if !baseline.exists() {
        panic!(
            "Baseline screenshot not found at '{}'. \
             Generate it with: UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored",
            BASELINE_PATH
        );
    }

    // Capture new screenshot
    run_screenshot_capture(NEW_PATH);

    let baseline_img = image::open(BASELINE_PATH)
        .expect("failed to open baseline")
        .to_rgba8();
    let current_img = image::open(NEW_PATH)
        .expect("failed to open new screenshot")
        .to_rgba8();

    assert_eq!(
        baseline_img.dimensions(),
        current_img.dimensions(),
        "screenshot dimensions changed: baseline={:?}, current={:?}",
        baseline_img.dimensions(),
        current_img.dimensions(),
    );

    let diff_count = count_different_pixels(&baseline_img, &current_img, CHANNEL_THRESHOLD);
    let total = baseline_img.width() as u64 * baseline_img.height() as u64;
    let diff_pct = diff_count as f64 / total as f64 * 100.0;

    // Clean up the new screenshot on success
    if diff_pct < MAX_DIFF_PERCENT {
        std::fs::remove_file(NEW_PATH).ok();
    } else {
        // Save a diff image for debugging
        let mut diff_img = image::RgbaImage::new(baseline_img.width(), baseline_img.height());
        for (x, y, pixel) in diff_img.enumerate_pixels_mut() {
            let pa = baseline_img.get_pixel(x, y);
            let pb = current_img.get_pixel(x, y);
            let differs = pa
                .0
                .iter()
                .zip(pb.0.iter())
                .any(|(ca, cb)| ca.abs_diff(*cb) > CHANNEL_THRESHOLD);
            if differs {
                *pixel = image::Rgba([255, 0, 0, 255]); // red for different
            } else {
                // dim version of original
                *pixel = image::Rgba([pa.0[0] / 3, pa.0[1] / 3, pa.0[2] / 3, 255]);
            }
        }
        let diff_path = "tests/snapshots/dungeon_scene.diff.png";
        diff_img.save(diff_path).ok();

        panic!(
            "Screenshot differs by {:.2}% ({} of {} pixels). \n\
             New:  {}\n\
             Diff: {}\n\
             If this is intentional, update the baseline with:\n  \
             UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored",
            diff_pct, diff_count, total, NEW_PATH, diff_path
        );
    }
}

/// Generate or update the baseline screenshot.
/// Run with: UPDATE_SNAPSHOTS=true cargo test --test screenshot_test -- --ignored
#[test]
#[ignore]
fn update_baseline() {
    if std::env::var("UPDATE_SNAPSHOTS").is_err() {
        panic!("Set UPDATE_SNAPSHOTS=true to generate/update baselines");
    }
    run_screenshot_capture(BASELINE_PATH);
    assert!(
        Path::new(BASELINE_PATH).exists(),
        "baseline was not created at {}",
        BASELINE_PATH
    );
    println!("Baseline updated: {}", BASELINE_PATH);
}
