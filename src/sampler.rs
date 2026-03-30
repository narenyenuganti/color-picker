use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::window::{
    kCGWindowImageDefault, kCGWindowImageNominalResolution,
    kCGWindowListOptionOnScreenBelowWindow, CGWindowID,
};

use crate::color::Color;

/// Returns the current cursor position in screen coordinates (top-left origin).
pub fn get_cursor_position() -> (f64, f64) {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .expect("failed to create event source");
    let event = CGEvent::new(source).expect("failed to create event");
    let point = event.location();
    (point.x, point.y)
}

/// Samples the pixel color at the given screen coordinates.
/// `exclude_window` is the CGWindowID of the overlay window to exclude from capture.
pub fn sample_color(x: f64, y: f64, exclude_window: CGWindowID) -> Option<Color> {
    let rect = CGRect::new(&CGPoint::new(x, y), &CGSize::new(1.0, 1.0));

    // Capture what's on screen below the overlay window, at nominal (1x) resolution
    // so we get exactly 1 pixel regardless of Retina scaling.
    let image = core_graphics::window::create_image(
        rect,
        kCGWindowListOptionOnScreenBelowWindow,
        exclude_window,
        kCGWindowImageDefault | kCGWindowImageNominalResolution,
    )?;

    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return None;
    }

    let data = image.data();
    let bytes: &[u8] = &data;

    // macOS pixel format is typically BGRA (32 bits per pixel)
    if bytes.len() >= 4 {
        Some(Color::new(bytes[2], bytes[1], bytes[0]))
    } else {
        None
    }
}
