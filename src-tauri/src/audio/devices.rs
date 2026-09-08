//! Endpoint discovery and notification forwarding. COM callbacks only try-send a wakeup;
//! enumeration and webview IPC never run on Windows' notification or audio callbacks.
use crate::types::AudioDevice;

#[cfg(not(windows))]
pub fn list_outputs() -> anyhow::Result<Vec<AudioDevice>> {
    Ok(Vec::new())
}

#[cfg(windows)]
pub fn list_outputs() -> anyhow::Result<Vec<AudioDevice>> {
    on_mta_thread(enumerate_outputs)
}

/// Blocking-pool threads may already belong to CPAL's STA apartment. Never change
/// their COM model or leave COM initialized on a reusable thread.
#[cfg(windows)]
fn on_mta_thread<T: Send + 'static>(
    work: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> anyhow::Result<T> {
    std::thread::Builder::new()
        .name("enumerate-audio-outputs".into())
        .spawn(move || {
            wasapi::initialize_mta().ok()?;
            struct Apartment;
            impl Drop for Apartment {
                fn drop(&mut self) {
                    wasapi::deinitialize();
                }
            }
            let _apartment = Apartment;
            work()
        })?
        .join()
        .map_err(|_| anyhow::anyhow!("audio output enumeration thread panicked"))?
}

#[cfg(windows)]
fn enumerate_outputs() -> anyhow::Result<Vec<AudioDevice>> {
    use wasapi::{DeviceEnumerator, Direction};
    let enumerator = DeviceEnumerator::new()?;
    let default_id = enumerator
        .get_default_device(&Direction::Render)
        .ok()
        .and_then(|d| d.get_id().ok());
    let collection = enumerator.get_device_collection(&Direction::Render)?;
    let mut devices = Vec::new();
    for index in 0..collection.get_nbr_devices()? {
        let device = collection.get_device_at_index(index)?;
        let id = device.get_id()?;
        devices.push(AudioDevice {
            is_default: Some(&id) == default_id.as_ref(),
            id,
            name: device.get_friendlyname()?,
        });
    }
    Ok(devices)
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn output_enumeration_does_not_change_the_callers_sta_apartment() {
        std::thread::spawn(|| {
            wasapi::initialize_sta().ok().unwrap();
            // Reproduce the exact error the pooled worker used to produce.
            assert_eq!(wasapi::initialize_mta().0 as u32, 0x80010106);
            for _ in 0..3 {
                assert!(on_mta_thread(|| Ok(wasapi::initialize_sta().is_err())).unwrap());
            }
            assert_eq!(wasapi::initialize_mta().0 as u32, 0x80010106);
            wasapi::deinitialize();
        })
        .join()
        .unwrap();
    }
}

#[cfg(not(windows))]
pub fn watch(_app: tauri::AppHandle) {}

#[cfg(windows)]
pub fn watch(app: tauri::AppHandle) {
    use tauri::Emitter;
    // Process-lifetime worker. The registration guard is retained for its entire lifetime
    // and unregisters before the enumerator is dropped if the channel closes.
    let result = std::thread::Builder::new()
        .name("audio-device-events".into())
        .spawn(move || {
            let run = || -> anyhow::Result<()> {
                wasapi::initialize_mta().ok()?;
                let enumerator = wasapi::DeviceEnumerator::new()?;
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                let mut callbacks = wasapi::DeviceEventCallbacks::new();
                let wake = tx.clone();
                callbacks.set_device_added_callback(move |_| {
                    let _ = wake.try_send(());
                });
                let wake = tx.clone();
                callbacks.set_device_removed_callback(move |_| {
                    let _ = wake.try_send(());
                });
                let wake = tx.clone();
                callbacks.set_device_state_callback(move |_, _| {
                    let _ = wake.try_send(());
                });
                let wake = tx.clone();
                callbacks.set_default_device_callback(move |_, _, _| {
                    let _ = wake.try_send(());
                });
                callbacks.set_property_value_callback(move |_, _| {
                    let _ = tx.try_send(());
                });
                let _registration = enumerator.register_notification_callback(callbacks)?;
                while rx.recv().is_ok() {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    while rx.try_recv().is_ok() {}
                    let _ = app.emit(crate::types::events::AUDIO_DEVICES_CHANGED, ());
                }
                Ok(())
            };
            if let Err(error) = run() {
                tracing::warn!("Audio-device notifications unavailable: {error:#}");
            }
        });
    if let Err(error) = result {
        tracing::warn!("Audio-device watcher unavailable: {error}");
    }
}
