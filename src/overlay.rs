use cocoa::appkit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor, NSWindow,
    NSWindowStyleMask,
};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, NO, YES};
use objc::{class, msg_send, sel, sel_impl};

const OVERLAY_SIZE: f64 = 40.0;

extern "C" {
    fn CGWindowLevelForKey(key: i32) -> i32;
}

const K_CG_FLOATING_WINDOW_LEVEL_KEY: i32 = 5;

/// Registers a custom NSView subclass that draws a circle.
fn register_circle_view_class() -> &'static Class {
    let superclass = class!(NSView);
    let mut decl = ClassDecl::new("CircleOverlayView", superclass).unwrap();

    extern "C" fn draw_rect(this: &Object, _sel: Sel, _dirty_rect: NSRect) {
        unsafe {
            let bounds: NSRect = msg_send![this, bounds];
            let inset_rect = NSRect::new(
                NSPoint::new(bounds.origin.x + 4.0, bounds.origin.y + 4.0),
                NSSize::new(bounds.size.width - 8.0, bounds.size.height - 8.0),
            );

            // Create oval bezier path
            let path: id =
                msg_send![class!(NSBezierPath), bezierPathWithOvalInRect: inset_rect];

            // Dark outline for visibility on light backgrounds
            let dark: id = NSColor::colorWithRed_green_blue_alpha_(nil, 0.0, 0.0, 0.0, 0.5);
            let _: () = msg_send![dark, set];
            let _: () = msg_send![path, setLineWidth: 3.0f64];
            let _: () = msg_send![path, stroke];

            // White inner stroke for visibility on dark backgrounds
            let white: id = NSColor::colorWithRed_green_blue_alpha_(nil, 1.0, 1.0, 1.0, 0.9);
            let _: () = msg_send![white, set];
            let _: () = msg_send![path, setLineWidth: 1.5f64];
            let _: () = msg_send![path, stroke];
        }
    }

    extern "C" fn is_opaque(_this: &Object, _sel: Sel) -> BOOL {
        NO
    }

    unsafe {
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&Object, Sel, NSRect),
        );
        decl.add_method(
            sel!(isOpaque),
            is_opaque as extern "C" fn(&Object, Sel) -> BOOL,
        );
    }

    decl.register()
}

pub struct Overlay {
    window: id,
}

impl Overlay {
    /// Creates the overlay. Must be called from the main thread after NSApplication init.
    pub fn new() -> Self {
        let view_class = register_circle_view_class();

        unsafe {
            let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
                ),
                NSWindowStyleMask::NSBorderlessWindowMask,
                NSBackingStoreType::NSBackingStoreBuffered,
                NO,
            );

            window.setOpaque_(NO);
            window.setBackgroundColor_(NSColor::clearColor(nil));

            let level = CGWindowLevelForKey(K_CG_FLOATING_WINDOW_LEVEL_KEY) as i64 + 1;
            window.setLevel_(level);
            window.setIgnoresMouseEvents_(YES);
            window.setHasShadow_(NO);

            let frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(OVERLAY_SIZE, OVERLAY_SIZE),
            );
            let view: id = msg_send![view_class, alloc];
            let view: id = msg_send![view, initWithFrame: frame];
            window.setContentView_(view);
            window.makeKeyAndOrderFront_(nil);

            Self { window }
        }
    }

    /// Returns the CGWindowID of the overlay window (for excluding from screenshots).
    pub fn window_id(&self) -> u32 {
        unsafe {
            let num: i64 = msg_send![self.window, windowNumber];
            num as u32
        }
    }

    /// Moves the overlay to follow the cursor.
    /// `x` and `y` are in CoreGraphics coordinates (top-left origin).
    pub fn update_position(&self, x: f64, y: f64) {
        unsafe {
            let screen: id = msg_send![class!(NSScreen), mainScreen];
            let screen_frame: NSRect = msg_send![screen, frame];
            // Convert CG (top-left origin) to NS (bottom-left origin)
            let ns_y = screen_frame.size.height - y - (OVERLAY_SIZE / 2.0);
            let ns_x = x - (OVERLAY_SIZE / 2.0);
            let origin = NSPoint::new(ns_x, ns_y);
            self.window.setFrameOrigin_(origin);
        }
    }

    /// Hides the overlay window.
    pub fn hide(&self) {
        unsafe {
            self.window.orderOut_(nil);
        }
    }
}

/// Initializes the macOS application environment. Must be called from the main thread.
pub fn init_macos_app() {
    unsafe {
        let app = NSApplication::sharedApplication(nil);
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );
    }
}

/// Processes pending macOS events. Call this in the main loop.
pub fn process_events() {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let app = NSApplication::sharedApplication(nil);
        loop {
            let event: id = msg_send![app,
                nextEventMatchingMask: 0xFFFFFFFFFFFFFFFFu64
                untilDate: nil
                inMode: cocoa::foundation::NSDefaultRunLoopMode
                dequeue: YES
            ];
            if event == nil {
                break;
            }
            let _: () = msg_send![app, sendEvent: event];
        }
        let _: () = msg_send![pool, drain];
    }
}
