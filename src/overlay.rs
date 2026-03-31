use objc2::rc::{autoreleasepool, Retained};
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBezierPath, NSColor,
    NSEventMask, NSScreen, NSView, NSWindow, NSWindowStyleMask,
};
use objc2_core_graphics::{CGWindowLevelForKey, CGWindowLevelKey};
use objc2_foundation::{NSDefaultRunLoopMode, NSPoint, NSRect, NSSize};

const OVERLAY_SIZE: f64 = 40.0;

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    struct CircleOverlayView;

    impl CircleOverlayView {
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty_rect: NSRect) {
            let bounds = self.bounds();
            let inset_rect = NSRect::new(
                NSPoint::new(bounds.origin.x + 4.0, bounds.origin.y + 4.0),
                NSSize::new(bounds.size.width - 8.0, bounds.size.height - 8.0),
            );

            let path = NSBezierPath::bezierPathWithOvalInRect(inset_rect);

            let dark = NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 0.5);
            dark.set();
            path.setLineWidth(3.0);
            path.stroke();

            let white = NSColor::colorWithSRGBRed_green_blue_alpha(1.0, 1.0, 1.0, 0.9);
            white.set();
            path.setLineWidth(1.5);
            path.stroke();
        }

        #[unsafe(method(isOpaque))]
        fn is_opaque(&self) -> bool {
            false
        }
    }
);

impl CircleOverlayView {
    fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), initWithFrame: frame] }
    }
}

pub struct Overlay {
    window: Retained<NSWindow>,
}

impl Overlay {
    /// Creates the overlay. Must be called from the main thread after NSApplication init.
    pub fn new() -> Self {
        let mtm = main_thread_marker();

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
                ),
                NSWindowStyleMask::Borderless,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        window.setOpaque(false);
        window.setBackgroundColor(Some(&NSColor::clearColor()));

        let level = CGWindowLevelForKey(CGWindowLevelKey::FloatingWindowLevelKey) as i64 + 1;
        window.setLevel(level as _);
        window.setIgnoresMouseEvents(true);
        window.setHasShadow(false);

        let frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
        );
        let view = CircleOverlayView::new(mtm, frame);
        window.setContentView(Some(&view));
        window.makeKeyAndOrderFront(None);

        Self { window }
    }

    /// Returns the CGWindowID of the overlay window (for excluding from screenshots).
    pub fn window_id(&self) -> u32 {
        self.window.windowNumber() as u32
    }

    /// Moves the overlay to follow the cursor.
    /// `x` and `y` are in CoreGraphics coordinates (top-left origin).
    pub fn update_position(&self, x: f64, y: f64) {
        let Some(screen) = NSScreen::mainScreen(main_thread_marker()) else {
            return;
        };

        let screen_frame = screen.frame();
        let ns_y = screen_frame.size.height - y - (OVERLAY_SIZE / 2.0);
        let ns_x = x - (OVERLAY_SIZE / 2.0);
        let origin = NSPoint::new(ns_x, ns_y);
        self.window.setFrameOrigin(origin);
    }

    /// Hides the overlay window.
    pub fn hide(&self) {
        self.window.orderOut(None);
    }
}

/// Initializes the macOS application environment. Must be called from the main thread.
pub fn init_macos_app() {
    let app = NSApplication::sharedApplication(main_thread_marker());
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

/// Processes pending macOS events. Call this in the main loop.
pub fn process_events() {
    autoreleasepool(|_| {
        let app = NSApplication::sharedApplication(main_thread_marker());
        let run_loop_mode = unsafe { NSDefaultRunLoopMode };

        loop {
            let event = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::Any,
                None,
                run_loop_mode,
                true,
            );
            let Some(event) = event else {
                break;
            };
            app.sendEvent(&event);
        }
    });
}

fn main_thread_marker() -> MainThreadMarker {
    MainThreadMarker::new().expect("overlay APIs must run on the main thread")
}
