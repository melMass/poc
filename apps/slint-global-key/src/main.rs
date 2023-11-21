#[allow(clippy::all)]
mod generated_code {
    #![allow(dead_code)]
    slint::include_modules!();
}
use generated_code::*;
use global_hotkey::{
    hotkey::{Code, HotKey},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use log::debug;
use slint::{ComponentHandle, LogicalPosition, LogicalSize, WindowPosition, WindowSize};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

// Animation parameters
const ANIMATION_DURATION: Duration = Duration::from_millis(300);
// const WINDOW_START_POS: LogicalPosition = LogicalPosition::new(0.0, -400.0); // Hidden position
// const WINDOW_END_POS: LogicalPosition = LogicalPosition::new(0.0, 0.0); // Visible position

const WINDOW_START_SIZE: LogicalSize = LogicalSize::new(600.0, 0.0); // Hidden SizeLogicalSize
const WINDOW_END_SIZE: LogicalSize = LogicalSize::new(600.0, 400.0); // Visible SizeLogicalSize
                                                                     // Animation logic
fn animate_window(show: bool, app_weak: slint::Weak<generated_code::App>) {
    // let start_pos = WINDOW_START_POS;
    // let end_pos = WINDOW_END_POS;

    let start_size = WINDOW_START_SIZE;
    let end_size = WINDOW_END_SIZE;

    let start_time = Instant::now();

    if show {
        app_weak
            .upgrade_in_event_loop(|app| {
                app.window().show().unwrap();
            })
            .unwrap();
    }

    while start_time.elapsed() < ANIMATION_DURATION {
        let fraction = start_time.elapsed().as_secs_f32() / ANIMATION_DURATION.as_secs_f32();
        // let interpolated_pos = if show {
        //     interpolate_position(start_pos, end_pos, fraction)
        // } else {
        //     interpolate_position(end_pos, start_pos, fraction)
        // };
        let interpolated_size = if show {
            interpolate_size(start_size, end_size, fraction)
        } else {
            interpolate_size(end_size, start_size, fraction)
        };
        debug!(
            "Current size: {}x{}",
            interpolated_size.width, interpolated_size.height
        );
        // Update the window position on the UI thread
        app_weak
            .upgrade_in_event_loop(move |app| {
                // app.window()
                //     .set_position(WindowPosition::Logical(interpolated_pos));
                app.window().set_size(interpolated_size);
            })
            .unwrap();

        std::thread::sleep(Duration::from_millis(16));
    }

    if !show {
        app_weak
            .upgrade_in_event_loop(|app| {
                app.window().hide().unwrap();
            })
            .unwrap();
    }
}

// fn interpolate_position(
//     start: LogicalPosition,
//     end: LogicalPosition,
//     fraction: f32,
// ) -> LogicalPosition {
//     LogicalPosition::new(
//         start.x + (end.x - start.x) * fraction,
//         start.y + (end.y - start.y) * fraction,
//     )
// }

fn interpolate_size(start: LogicalSize, end: LogicalSize, fraction: f32) -> LogicalSize {
    LogicalSize::new(
        start.width + (end.width - start.width) * fraction,
        start.height + (end.height - start.height) * fraction,
    )
}

fn main() {
    flexi_logger::Logger::try_with_str("debug, my::critical::module=trace")
        .unwrap()
        .start()
        .unwrap();

    // workaround from https://github.com/slint-ui/slint/issues/1499#issuecomment-1794517946
    i_slint_backend_selector::with_platform(|b| {
        b.set_event_loop_quit_on_last_window_closed(false);
        Ok(())
    })
    .unwrap();
    debug!("App won't quit on last 'hidden' window");

    // the slint generated app
    let app = App::new().unwrap();
    let app_weak = app.as_weak();
    let show_window = Arc::new(AtomicBool::new(true));

    // global-hotkey setup
    let running = Arc::new(AtomicBool::new(true));

    let manager = GlobalHotKeyManager::new().expect("failed to initiate hotkeymanager");
    let hotkey = HotKey::new(None, Code::Digit2);
    manager.register(hotkey).expect("failed to register hotkey");

    let receiver = GlobalHotKeyEvent::receiver();

    let keyloop = std::thread::spawn({
        let running = running.clone();
        let show_window = show_window.clone();
        let app_weak = app_weak.clone(); // Clone the weak reference for the thread

        move || {
            debug!("Keyloop: Listening for global hotkey");
            while running.load(Ordering::SeqCst) {
                if let Ok(event) = receiver.try_recv() {
                    if event.state == HotKeyState::Released {
                        let current_state = show_window.load(Ordering::SeqCst);
                        show_window.store(!current_state, Ordering::SeqCst);

                        // Start the animation on a new thread
                        let show_window_clone = show_window.clone();
                        let app_weak_clone = app_weak.clone(); // Clone the weak reference for the animation thread
                        std::thread::spawn(move || {
                            animate_window(
                                show_window_clone.load(Ordering::SeqCst),
                                app_weak_clone,
                            );
                        });
                    }

                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
    let key_arc = Arc::new(Mutex::new(Some(keyloop)));
    // app.window()
    //     .set_position(WindowPosition::Logical(LogicalPosition::new(0.0, 0.0)));
    // app.window()
    //     .set_size(WindowSize::Logical(LogicalSize::new(55.0, 16.0)));

    app.window().on_close_requested({
        let key_arc = key_arc.clone();
        let running = running.clone();
        move || {
            // cleanup
            debug!("Close requested, shutting down the keyloop");
            running.store(false, Ordering::SeqCst);
            let mut keyloop_guard = key_arc.lock().unwrap();
            if let Some(keyloop) = keyloop_guard.take() {
                keyloop.join().expect("failed to join keyloop thread");
            }
            slint::CloseRequestResponse::HideWindow
        }
    });

    app.on_close({
        let running = running.clone();
        let show_window = show_window.clone();
        let app_weak = app_weak.clone(); // Clone the weak reference for the thread

        move || {
            let current_state = show_window.load(Ordering::SeqCst);
            show_window.store(!current_state, Ordering::SeqCst);

            // Start the animation on a new thread
            let show_window_clone = show_window.clone();
            let app_weak_clone = app_weak.clone(); // Clone the weak reference for the animation thread
            std::thread::spawn(move || {
                animate_window(show_window_clone.load(Ordering::SeqCst), app_weak_clone);
            });
        }
    });
    // Run the application UI
    app.show().unwrap();
    slint::run_event_loop().unwrap();

    debug!("Bye");
}
