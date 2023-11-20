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
    time::Duration,
};
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

    // the slint generated app
    let app = App::new().unwrap();

    // global-hotkey setup
    let running = Arc::new(AtomicBool::new(true));

    let manager = GlobalHotKeyManager::new().expect("failed to initiate hotkeymanager");
    let hotkey = HotKey::new(None, Code::Digit2);
    manager.register(hotkey).expect("failed to register hotkey");

    let receiver = GlobalHotKeyEvent::receiver();

    let keyloop = std::thread::spawn({
        let running = running.clone();
        let app_weak = app.as_weak();
        move || {
            debug!("Keyloop: Starting to listen for global shortcuts");
            while running.load(Ordering::SeqCst) {
                if let Ok(event) = receiver.try_recv() {
                    println!("tray event: {event:?}");
                    if event.state == HotKeyState::Released {
                        app_weak
                            .upgrade_in_event_loop(|app| {
                                if app.window().is_visible() {
                                    app.window().hide().unwrap();
                                    // app.window().set_position(WindowPosition::Logical(
                                    //     LogicalPosition::new(0.0, 0.0),
                                    // ));
                                    // app.window().set_size(WindowSize::Logical(LogicalSize::new(
                                    //     55.0, 16.0,
                                    // )));

                                    // app.set_small(true);
                                } else {
                                    // app.window().set_position(WindowPosition::Logical(
                                    //     LogicalPosition::new(0.0, 0.0),
                                    // ));
                                    // app.window().set_size(WindowSize::Logical(LogicalSize::new(
                                    //     800.0, 400.0,
                                    // )));
                                    app.window().show().unwrap();
                                    // app.set_small(false);
                                }
                            })
                            .expect("failed to update in event loop");
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            debug!("Keyloop: Terminated");
        }
    });
    let key_arc = Arc::new(Mutex::new(Some(keyloop)));
    // app.window()
    //     .set_position(WindowPosition::Logical(LogicalPosition::new(0.0, 0.0)));
    // app.window()
    //     .set_size(WindowSize::Logical(LogicalSize::new(55.0, 16.0)));

    app.window().on_close_requested({
        let key_arc = key_arc.clone();
        move || {
            // cleanup
            running.store(false, Ordering::SeqCst);
            let mut keyloop_guard = key_arc.lock().unwrap();
            if let Some(keyloop) = keyloop_guard.take() {
                keyloop.join().expect("failed to join keyloop thread");
            }
            slint::CloseRequestResponse::HideWindow
        }
    });
    // Run the application UI
    app.show().unwrap();
    slint::run_event_loop().unwrap();
    // app.run().unwrap();
}
