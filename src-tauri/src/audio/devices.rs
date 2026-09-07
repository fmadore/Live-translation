//! Endpoint discovery and notification forwarding. COM callbacks only try-send a wakeup;
//! enumeration and webview IPC never run on Windows' notification or audio callbacks.
use crate::types::AudioDevice;

#[cfg(not(windows))]
pub fn list_outputs() -> anyhow::Result<Vec<AudioDevice>> {
    Ok(Vec::new())
}

#[cfg(windows)]
pub fn list_outputs() -> anyhow::Result<Vec<AudioDevice>> {
    use wasapi::{initialize_mta, DeviceEnumerator, Direction};
    initialize_mta().ok()?;
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
