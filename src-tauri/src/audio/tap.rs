//! Core Audio process taps (macOS 14.2 and later): the system-audio source on a Mac.
//!
//! A tap copies what processes play, before it reaches the speakers, without changing what is
//! heard. It is read through a private aggregate device that holds only the tap. Both are
//! created for one capture and destroyed with it ([`Tap`]'s `Drop`), and neither is visible to
//! other applications. macOS asks once for *System Audio Recording*, with the text of
//! `NSAudioCaptureUsageDescription`; no screen content is involved.
//!
//! Two kinds of tap serve the two capture modes of `applications::SystemCapture`:
//!
//! - [`Target::Output`]: every process playing to one output device, mixed as that device
//!   plays it. Unlike cpal's loopback, which opens a device's *input* when it has one, this
//!   works for an output that also has a microphone (a USB headset or speakerphone).
//! - [`Target::Processes`]: a stereo mix of the listed process objects only, never anything
//!   else; an empty list taps silence. The list can change while the tap runs
//!   ([`Tap::set_processes`]), for an application whose helper processes start playing later.
//!
//! Audio arrives on Core Audio's I/O thread and goes straight into the shared
//! [`CaptureState`], as the microphone's does: downmix, resampling, chunks and meter.

use std::ffi::{CStr, c_void};
use std::panic::AssertUnwindSafe;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use anyhow::{Context as _, Result, anyhow};
use objc2::rc::Retained;
use objc2::{AnyThread, Message};
use objc2_core_audio::{
    AudioDeviceCreateIOProcID, AudioDeviceDestroyIOProcID, AudioDeviceIOProcID, AudioDeviceStart,
    AudioDeviceStop, AudioHardwareCreateAggregateDevice, AudioHardwareCreateProcessTap,
    AudioHardwareDestroyAggregateDevice, AudioHardwareDestroyProcessTap,
    AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize, AudioObjectID,
    AudioObjectPropertyAddress, AudioObjectPropertySelector, AudioObjectSetPropertyData,
    CATapDescription, CATapMuteBehavior, kAudioAggregateDeviceIsPrivateKey,
    kAudioAggregateDeviceNameKey, kAudioAggregateDeviceTapAutoStartKey,
    kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDevices, kAudioHardwarePropertyProcessObjectList,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyScopeOutput, kAudioObjectSystemObject, kAudioProcessPropertyDevices,
    kAudioProcessPropertyIsRunningOutput, kAudioProcessPropertyPID,
    kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey, kAudioTapPropertyDescription,
    kAudioTapPropertyFormat,
};
use objc2_core_audio_types::{
    AudioBuffer, AudioBufferList, AudioStreamBasicDescription, AudioTimeStamp,
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved, kAudioFormatLinearPCM,
};
use objc2_core_foundation::{
    CFArray, CFDictionary, CFMutableDictionary, CFRetained, CFString, kCFAllocatorDefault,
    kCFTypeArrayCallBacks, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
};
use objc2_foundation::{NSArray, NSNumber, NSString};

use super::capture::CaptureState;
use crate::sync::lock;

/// What a tap copies.
pub enum Target<'a> {
    /// Every process playing to the output device with this Core Audio UID.
    Output { device_uid: &'a str },
    /// Only these process objects (`kAudioHardwarePropertyProcessObjectList`), mixed to stereo.
    Processes(&'a [AudioObjectID]),
}

/// Numbers the aggregate devices this process makes, so two never share a UID.
static NEXT_DEVICE: AtomicU32 = AtomicU32::new(0);

/// A running tap and the private aggregate device it is read through.
///
/// Dropping it closes the capture at once, then takes the tap down on a thread of its own.
/// Taking it down can block for as long as macOS likes: while the System Audio Recording
/// question is still on screen, `AudioDeviceDestroyIOProcID` waits on `coreaudiod`, which is
/// waiting on the operator. Done on the capture thread, that kept the session's audio channel
/// open, so Stop never finished and the window sat on "Stopping…".
pub struct Tap {
    /// Always `Some` until `Drop` hands it to the teardown thread.
    inner: Option<Inner>,
}

/// The Core Audio objects behind a [`Tap`], destroyed together when this is dropped.
struct Inner {
    tap_id: AudioObjectID,
    aggregate_id: AudioObjectID,
    proc_id: AudioDeviceIOProcID,
    started: bool,
    description: Retained<CATapDescription>,
    /// Owned here, lent to the I/O callback as its client data; freed only after the callback
    /// has been stopped and destroyed.
    context: *mut Context,
}

// SAFETY: every field is an object id, a Core Audio handle that any thread may use, or the
// context pointer, whose `Context` is `Send` and is only reached through its mutex and atomic.
// The description is touched by one thread at a time: the owner of the `Tap`, then the
// teardown thread it is handed to.
unsafe impl Send for Inner {}

/// The most separate channel buffers the callback interleaves; more are left out.
const MAX_PLANES: usize = 16;

/// What the I/O callback needs: the layout it was given and where the samples go.
struct Context {
    interleaved: bool,
    /// `None` once the tap is being dropped: the capture state, and with it the session's
    /// audio and meter channels, is released before Core Audio is asked to stop calling.
    sink: Mutex<Option<Sink>>,
    /// Set by the callback when a buffer holds anything but exact zeros; read and cleared by
    /// [`Tap::take_heard`].
    heard: AtomicBool,
}

struct Sink {
    state: CaptureState,
    /// Non-interleaved buffers are interleaved here, reused from one callback to the next.
    scratch: Vec<f32>,
}

impl Tap {
    /// Create the tap and its aggregate device, then start feeding `state`, which
    /// `make_state` builds once the tap's sample rate is known.
    pub fn start(
        target: &Target<'_>,
        make_state: impl FnOnce(u32) -> CaptureState,
    ) -> Result<Self> {
        let description = describe(target);
        let mut tap_id: AudioObjectID = 0;
        // SAFETY: `description` is a valid, initialised tap description, and `tap_id` is a
        // valid place for the new object's id.
        status(
            unsafe { AudioHardwareCreateProcessTap(Some(&description), &raw mut tap_id) },
            "could not create the audio tap (System Audio Recording may be refused)",
        )?;
        // From here on, `Drop` cleans up whatever has been made.
        let mut tap = Inner {
            tap_id,
            aggregate_id: 0,
            proc_id: None,
            started: false,
            description,
            context: std::ptr::null_mut(),
        };

        let format: AudioStreamBasicDescription = get_property(tap_id, kAudioTapPropertyFormat)
            .context("could not read the tap's format")?;
        let (rate, channels, interleaved) = check_format(&format)?;
        tracing::info!(rate, channels, interleaved, "Core Audio tap created");

        // SAFETY: the tap's UUID is a plain property of a live description.
        let tap_uid = unsafe { description_uid(&tap.description) };
        let instance = NEXT_DEVICE.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let properties = aggregate_properties(
            &tap_uid,
            &format!("io.github.fmadore.live-translation.tap.{pid}.{instance}"),
            "Live Translation system audio",
        );
        let mut aggregate_id: AudioObjectID = 0;
        // SAFETY: `properties` is a valid aggregate-device description that outlives the call,
        // and `aggregate_id` is a valid place for the new device's id.
        status(
            unsafe {
                AudioHardwareCreateAggregateDevice(&properties, NonNull::from(&mut aggregate_id))
            },
            "could not create the device that reads the tap",
        )?;
        tap.aggregate_id = aggregate_id;

        tap.context = Box::into_raw(Box::new(Context {
            interleaved,
            sink: Mutex::new(Some(Sink {
                state: make_state(rate),
                scratch: Vec::with_capacity(4096),
            })),
            heard: AtomicBool::new(false),
        }));
        let mut proc_id: AudioDeviceIOProcID = None;
        // SAFETY: `io_proc` matches `AudioDeviceIOProc`; `tap.context` stays valid until the
        // proc is destroyed in `Drop`; `proc_id` is a valid place for the new id.
        status(
            unsafe {
                AudioDeviceCreateIOProcID(
                    aggregate_id,
                    Some(io_proc),
                    tap.context.cast(),
                    NonNull::from(&mut proc_id),
                )
            },
            "could not attach to the device that reads the tap",
        )?;
        tap.proc_id = proc_id;
        // SAFETY: the device and the proc id were both created above and are still alive.
        status(
            unsafe { AudioDeviceStart(aggregate_id, proc_id) },
            "could not start reading the tap",
        )?;
        tap.started = true;
        Ok(Self { inner: Some(tap) })
    }

    /// Whether anything but digital silence has arrived since the last call.
    pub fn take_heard(&self) -> bool {
        self.inner.as_ref().is_some_and(|inner| {
            // SAFETY: the context lives as long as `inner`, which is alive here.
            !inner.context.is_null()
                && unsafe { &*inner.context }
                    .heard
                    .swap(false, Ordering::Relaxed)
        })
    }

    /// Tap these process objects instead of the current ones, without stopping. Only for a
    /// [`Target::Processes`] tap.
    pub fn set_processes(&mut self, processes: &[AudioObjectID]) -> Result<()> {
        let inner = self.inner.as_ref().context("the tap has been taken down")?;
        let list = process_array(processes);
        // SAFETY: setting the process list of a live description, which `CATapDescription`
        // copies.
        unsafe { inner.description.setProcesses(&list) };
        let pointer: *const CATapDescription = Retained::as_ptr(&inner.description);
        let address = address(kAudioTapPropertyDescription);
        // SAFETY: the property's value is a `CATapDescription *`; `pointer` is one, alive for
        // the call, and the size passed is the size of that pointer.
        status(
            unsafe {
                AudioObjectSetPropertyData(
                    inner.tap_id,
                    NonNull::from(&address),
                    0,
                    std::ptr::null(),
                    size_of_u32::<*const CATapDescription>(),
                    NonNull::from(&pointer).cast(),
                )
            },
            "could not change which processes the tap captures",
        )
    }
}

impl Drop for Tap {
    fn drop(&mut self) {
        let Some(inner) = self.inner.take() else {
            return;
        };
        // First let go of the capture state, so whoever reads the session's audio sees it end
        // now, whatever happens to the Core Audio calls below. The callback only ever
        // `try_lock`s and holds the lock for one buffer, so this waits a few milliseconds at
        // most; after it, the callback finds `None` and returns at once.
        if !inner.context.is_null() {
            // SAFETY: the context stays alive until `inner` is dropped, below or on the
            // teardown thread.
            let sink = lock(&unsafe { &*inner.context }.sink).take();
            drop(sink);
        }
        // Then take the tap down where a long wait harms nothing. If no thread can be started,
        // the refused closure is dropped here, and `inner` with it: blocking the capture
        // thread is still better than leaking the tap.
        let _ = std::thread::Builder::new()
            .name("tap-teardown".into())
            .spawn(move || {
                let started = std::time::Instant::now();
                drop(inner);
                let waited = started.elapsed();
                if waited > std::time::Duration::from_secs(1) {
                    tracing::warn!(?waited, "Core Audio took a long time to remove the tap");
                }
            });
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        // In reverse order of creation. Statuses are ignored: nothing useful can be done with
        // a failure here, and a panic in drop would be worse.
        // SAFETY: each call is made only on what was created, once. `AudioDeviceStop` waits
        // for a running callback to return, and the proc is destroyed before its context is
        // freed, so the callback never sees a dangling pointer.
        unsafe {
            if self.started {
                let _ = AudioDeviceStop(self.aggregate_id, self.proc_id);
            }
            if self.proc_id.is_some() {
                let _ = AudioDeviceDestroyIOProcID(self.aggregate_id, self.proc_id);
            }
            if self.aggregate_id != 0 {
                let _ = AudioHardwareDestroyAggregateDevice(self.aggregate_id);
            }
            let _ = AudioHardwareDestroyProcessTap(self.tap_id);
            if !self.context.is_null() {
                drop(Box::from_raw(self.context));
            }
        }
    }
}

/// The tap description for `target`: private (no other application sees it), and unmuted, so
/// the room still hears everything.
fn describe(target: &Target<'_>) -> Retained<CATapDescription> {
    let description = match target {
        Target::Output { device_uid } => {
            // An exclusive tap with no processes listed excludes none: every process playing
            // to this device, in the device's own format.
            let none: Retained<NSArray<NSNumber>> = NSArray::new();
            // SAFETY: a fresh allocation initialised with a valid array and device UID.
            let description = unsafe {
                CATapDescription::initWithProcesses_andDeviceUID_withStream(
                    CATapDescription::alloc(),
                    &none,
                    &NSString::from_str(device_uid),
                    0,
                )
            };
            // SAFETY: a setter on a live description.
            unsafe { description.setExclusive(true) };
            description
        }
        Target::Processes(processes) => {
            // SAFETY: a fresh allocation initialised with a valid array of process objects.
            unsafe {
                CATapDescription::initStereoMixdownOfProcesses(
                    CATapDescription::alloc(),
                    &process_array(processes),
                )
            }
        }
    };
    // SAFETY: setters on a live description.
    unsafe {
        description.setPrivate(true);
        description.setMuteBehavior(CATapMuteBehavior::Unmuted);
        description.setName(&NSString::from_str("Live Translation"));
    }
    description
}

/// Whether another process is playing sound into what `target` taps right now: to that output
/// device, or, for an application, from one of its processes. With [`Tap::take_heard`] it tells
/// a tap that macOS keeps silent apart from one that has nothing to copy. A property that
/// cannot be read counts as not playing, so an unknown never raises a false alarm.
pub fn playing(target: &Target<'_>) -> bool {
    let own = i32::try_from(std::process::id()).unwrap_or(-1);
    let system = AudioObjectID::try_from(kAudioObjectSystemObject).unwrap_or(1);
    let (objects, device) = match target {
        Target::Output { device_uid } => (
            get_ids(system, kAudioHardwarePropertyProcessObjectList).unwrap_or_default(),
            output_device(device_uid),
        ),
        Target::Processes(processes) => (processes.to_vec(), None),
    };
    if matches!(target, Target::Output { .. }) && device.is_none() {
        return false;
    }
    objects.into_iter().any(|object| {
        get_property::<i32>(object, kAudioProcessPropertyPID).is_ok_and(|pid| pid != own)
            && get_property::<u32>(object, kAudioProcessPropertyIsRunningOutput)
                .is_ok_and(|running| running != 0)
            && device.is_none_or(|device| {
                // Only the output scope answers: in the global scope the list comes back empty.
                let address = AudioObjectPropertyAddress {
                    mSelector: kAudioProcessPropertyDevices,
                    mScope: kAudioObjectPropertyScopeOutput,
                    mElement: kAudioObjectPropertyElementMain,
                };
                get_ids_at(object, &address).is_ok_and(|devices| devices.contains(&device))
            })
    })
}

/// The audio object of the device with this UID.
fn output_device(uid: &str) -> Option<AudioObjectID> {
    let system = AudioObjectID::try_from(kAudioObjectSystemObject).unwrap_or(1);
    get_ids(system, kAudioHardwarePropertyDevices)
        .ok()?
        .into_iter()
        .find(|&device| get_string(device, kAudioDevicePropertyDeviceUID).is_ok_and(|u| u == uid))
}

fn process_array(processes: &[AudioObjectID]) -> Retained<NSArray<NSNumber>> {
    let numbers: Vec<Retained<NSNumber>> =
        processes.iter().map(|&id| NSNumber::new_u32(id)).collect();
    NSArray::from_retained_slice(&numbers)
}

/// The tap's UID, which the aggregate device names it by.
///
/// # Safety
///
/// `description` must be a live tap description.
unsafe fn description_uid(description: &CATapDescription) -> Retained<NSString> {
    // SAFETY: the caller's.
    unsafe { description.UUID().UUIDString() }
}

/// Sample rate, channel count and layout, if the tap delivers what the callback reads: 32-bit
/// float linear PCM, the format Core Audio gives taps.
fn check_format(format: &AudioStreamBasicDescription) -> Result<(u32, usize, bool)> {
    let float32 = format.mFormatID == kAudioFormatLinearPCM
        && format.mFormatFlags & kAudioFormatFlagIsFloat != 0
        && format.mBitsPerChannel == 32;
    anyhow::ensure!(
        float32,
        "the tap delivers an unexpected format ({} bits, flags {:#x})",
        format.mBitsPerChannel,
        format.mFormatFlags
    );
    anyhow::ensure!(format.mChannelsPerFrame > 0, "the tap reports no channels");
    let rate = format.mSampleRate.round();
    anyhow::ensure!(
        rate >= 1.0 && rate <= f64::from(u32::MAX),
        "the tap reports a sample rate of {rate}"
    );
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rounded and checked to be within u32 just above"
    )]
    let rate = rate as u32;
    let channels = usize::try_from(format.mChannelsPerFrame)?;
    let interleaved = format.mFormatFlags & kAudioFormatFlagIsNonInterleaved == 0;
    Ok((rate, channels, interleaved))
}

/// Core Audio's I/O callback for the aggregate device: hand the tap's buffers to the capture
/// state. Never blocks, never allocates once the scratch buffer has grown, never unwinds into
/// Core Audio.
unsafe extern "C-unwind" fn io_proc(
    _device: AudioObjectID,
    _now: NonNull<AudioTimeStamp>,
    input: NonNull<AudioBufferList>,
    _input_time: NonNull<AudioTimeStamp>,
    _output: NonNull<AudioBufferList>,
    _output_time: NonNull<AudioTimeStamp>,
    client: *mut c_void,
) -> i32 {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: `client` is the `Context` that `Tap::start` passed, kept alive until the proc
        // is destroyed; `input` is a buffer list Core Audio owns for the length of the call.
        let (context, buffers) = unsafe { (&*client.cast::<Context>(), buffers(input)) };
        // Uncontended: only this callback locks it. Skip a buffer rather than wait.
        let Ok(mut sink) = context.sink.try_lock() else {
            return;
        };
        // Detached: the tap is being taken down.
        let Some(Sink { state, scratch }) = sink.as_mut() else {
            return;
        };
        // A refused permission delivers exact zeros; real sound, however quiet, does not.
        // Once something has been heard, later buffers are not scanned until the flag is read.
        let note = |samples: &[f32]| {
            if !context.heard.load(Ordering::Relaxed) && samples.iter().any(|&s| s != 0.0) {
                context.heard.store(true, Ordering::Relaxed);
            }
        };
        if context.interleaved {
            for buffer in buffers {
                // SAFETY: Core Audio's buffer, valid for the call, of float samples.
                let samples = unsafe { samples(buffer) };
                note(samples);
                let channels = usize::try_from(buffer.mNumberChannels).unwrap_or(1).max(1);
                state.push_samples(samples, channels);
            }
        } else {
            // One buffer per channel: interleave them for the shared downmix.
            let mut planes: [&[f32]; MAX_PLANES] = [&[]; MAX_PLANES];
            let channels = buffers.len().min(MAX_PLANES);
            for (plane, buffer) in planes.iter_mut().zip(buffers) {
                // SAFETY: Core Audio's buffer, valid for the call, of float samples.
                *plane = unsafe { samples(buffer) };
                note(plane);
            }
            let planes = &planes[..channels];
            let frames = planes.iter().map(|plane| plane.len()).min().unwrap_or(0);
            if frames == 0 {
                return;
            }
            scratch.clear();
            for frame in 0..frames {
                scratch.extend(planes.iter().map(|plane| plane[frame]));
            }
            state.push_samples(scratch, channels);
        }
    }));
    if result.is_err() {
        tracing::error!("the system audio callback panicked; its buffer was dropped");
    }
    0
}

/// The buffers of a list Core Audio passed to the callback.
///
/// # Safety
///
/// `list` must point to a valid `AudioBufferList` whose `mNumberBuffers` buffers follow it,
/// alive for `'a`.
unsafe fn buffers<'a>(list: NonNull<AudioBufferList>) -> &'a [AudioBuffer] {
    // SAFETY: the caller's; `mBuffers` is the start of a C flexible array.
    unsafe {
        let list = list.as_ptr();
        let count = usize::try_from((*list).mNumberBuffers).unwrap_or(0);
        std::slice::from_raw_parts((&raw const (*list).mBuffers).cast::<AudioBuffer>(), count)
    }
}

/// A buffer's samples, read as 32-bit floats.
///
/// # Safety
///
/// `buffer` must be a Core Audio buffer of float samples, valid for `'a`.
unsafe fn samples<'a>(buffer: &AudioBuffer) -> &'a [f32] {
    if buffer.mData.is_null() {
        return &[];
    }
    let count = usize::try_from(buffer.mDataByteSize).unwrap_or(0) / size_of::<f32>();
    // SAFETY: the caller's; Core Audio aligns sample buffers for their format.
    unsafe { std::slice::from_raw_parts(buffer.mData.cast::<f32>(), count) }
}

/// The description of a private aggregate device holding only the tap, started with it.
fn aggregate_properties(tap_uid: &NSString, uid: &str, name: &str) -> CFRetained<CFDictionary> {
    let yes = NSNumber::new_bool(true);
    // SAFETY: CF containers made with the standard callbacks, which retain what is put in;
    // every key and value is a live CF or toll-free bridged object.
    unsafe {
        let sub_tap = new_dictionary(2);
        set(
            &sub_tap,
            kAudioSubTapUIDKey,
            Retained::as_ptr(&tap_uid.retain()).cast(),
        );
        set(
            &sub_tap,
            kAudioSubTapDriftCompensationKey,
            Retained::as_ptr(&yes).cast(),
        );
        let entries = [CFRetained::as_ptr(&sub_tap)
            .as_ptr()
            .cast_const()
            .cast::<c_void>()];
        let taps = CFArray::new(
            kCFAllocatorDefault,
            entries.as_ptr().cast_mut(),
            1,
            &kCFTypeArrayCallBacks,
        )
        .expect("CFArrayCreate failed");

        let device = new_dictionary(5);
        let name = CFString::from_str(name);
        let uid = CFString::from_str(uid);
        set(
            &device,
            kAudioAggregateDeviceNameKey,
            CFRetained::as_ptr(&name).as_ptr().cast(),
        );
        set(
            &device,
            kAudioAggregateDeviceUIDKey,
            CFRetained::as_ptr(&uid).as_ptr().cast(),
        );
        set(
            &device,
            kAudioAggregateDeviceTapListKey,
            CFRetained::as_ptr(&taps).as_ptr().cast(),
        );
        set(
            &device,
            kAudioAggregateDeviceTapAutoStartKey,
            Retained::as_ptr(&yes).cast(),
        );
        set(
            &device,
            kAudioAggregateDeviceIsPrivateKey,
            Retained::as_ptr(&yes).cast(),
        );
        CFRetained::cast_unchecked::<CFDictionary>(device)
    }
}

/// # Safety
///
/// Calls CoreFoundation; the result is a valid, empty, mutable dictionary.
unsafe fn new_dictionary(capacity: isize) -> CFRetained<CFMutableDictionary> {
    // SAFETY: the standard callbacks for CF-typed keys and values.
    unsafe {
        CFMutableDictionary::new(
            kCFAllocatorDefault,
            capacity,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        )
    }
    .expect("CFDictionaryCreateMutable failed")
}

/// # Safety
///
/// `value` must be a live CF (or toll-free bridged) object.
unsafe fn set(dictionary: &CFMutableDictionary, key: &CStr, value: *const c_void) {
    let key = CFString::from_str(key.to_str().unwrap_or_default());
    // SAFETY: the caller's for `value`; `key` is alive for the call and retained by the
    // dictionary.
    unsafe {
        CFMutableDictionary::set_value(
            Some(dictionary),
            CFRetained::as_ptr(&key).as_ptr().cast_const().cast(),
            value,
        );
    }
}

/// A global property address on the main element.
pub(super) fn address(selector: AudioObjectPropertySelector) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

/// A fixed-size property of an audio object.
pub(super) fn get_property<T: Copy>(
    object: AudioObjectID,
    selector: AudioObjectPropertySelector,
) -> Result<T> {
    let address = address(selector);
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    let mut size = size_of_u32::<T>();
    // SAFETY: `value` has room for `size` bytes, which Core Audio may fill.
    status(
        unsafe {
            AudioObjectGetPropertyData(
                object,
                NonNull::from(&address),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
                NonNull::new_unchecked(value.as_mut_ptr().cast()),
            )
        },
        "could not read an audio property",
    )?;
    anyhow::ensure!(
        size == size_of_u32::<T>(),
        "an audio property had an unexpected size"
    );
    // SAFETY: Core Audio wrote the whole value, as the size check confirms.
    Ok(unsafe { value.assume_init() })
}

/// A property that is a list of audio object ids.
pub(super) fn get_ids(
    object: AudioObjectID,
    selector: AudioObjectPropertySelector,
) -> Result<Vec<AudioObjectID>> {
    get_ids_at(object, &address(selector))
}

/// A list of audio object ids at a full property address, for a property whose answer depends
/// on its scope.
fn get_ids_at(
    object: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<Vec<AudioObjectID>> {
    let address = *address;
    let mut size = 0u32;
    // SAFETY: asks only for the size, into a valid place.
    status(
        unsafe {
            AudioObjectGetPropertyDataSize(
                object,
                NonNull::from(&address),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
            )
        },
        "could not size an audio property",
    )?;
    let count = usize::try_from(size)? / size_of::<AudioObjectID>();
    let mut ids = vec![0 as AudioObjectID; count];
    if count == 0 {
        return Ok(ids);
    }
    // SAFETY: `ids` has room for `size` bytes.
    status(
        unsafe {
            AudioObjectGetPropertyData(
                object,
                NonNull::from(&address),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
                NonNull::new_unchecked(ids.as_mut_ptr().cast()),
            )
        },
        "could not read an audio property",
    )?;
    ids.truncate(usize::try_from(size)? / size_of::<AudioObjectID>());
    Ok(ids)
}

/// A property that is a `CFString`, returned under the create rule.
pub(super) fn get_string(
    object: AudioObjectID,
    selector: AudioObjectPropertySelector,
) -> Result<String> {
    let raw: *mut CFString = get_property(object, selector)?;
    let string = NonNull::new(raw).ok_or_else(|| anyhow!("an audio property was empty"))?;
    // SAFETY: Core Audio returns this string retained for the caller.
    let string = unsafe { CFRetained::from_raw(string) };
    Ok(string.to_string())
}

fn size_of_u32<T>() -> u32 {
    u32::try_from(size_of::<T>()).unwrap_or(u32::MAX)
}

/// A Core Audio status as a result, its four-character code in the message.
pub(super) fn status(code: i32, what: &'static str) -> Result<()> {
    if code == 0 {
        return Ok(());
    }
    let bytes = code.to_be_bytes();
    let fourcc = if bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
        format!("'{}'", String::from_utf8_lossy(&bytes))
    } else {
        code.to_string()
    };
    Err(anyhow!("{what} (Core Audio status {fourcc})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(flags: u32, bits: u32, channels: u32, rate: f64) -> AudioStreamBasicDescription {
        AudioStreamBasicDescription {
            mSampleRate: rate,
            mFormatID: kAudioFormatLinearPCM,
            mFormatFlags: flags,
            mBytesPerPacket: 0,
            mFramesPerPacket: 1,
            mBytesPerFrame: 0,
            mChannelsPerFrame: channels,
            mBitsPerChannel: bits,
            mReserved: 0,
        }
    }

    #[test]
    fn a_float_tap_is_read_and_anything_else_refused() {
        assert_eq!(
            check_format(&format(kAudioFormatFlagIsFloat, 32, 2, 48_000.0)).unwrap(),
            (48_000, 2, true)
        );
        assert_eq!(
            check_format(&format(
                kAudioFormatFlagIsFloat | kAudioFormatFlagIsNonInterleaved,
                32,
                2,
                44_100.0
            ))
            .unwrap(),
            (44_100, 2, false)
        );
        assert!(check_format(&format(0, 16, 2, 48_000.0)).is_err());
        assert!(check_format(&format(kAudioFormatFlagIsFloat, 32, 0, 48_000.0)).is_err());
        assert!(check_format(&format(kAudioFormatFlagIsFloat, 32, 2, 0.0)).is_err());
    }

    #[test]
    fn a_status_names_its_code() {
        assert!(status(0, "x").is_ok());
        let error = status(i32::from_be_bytes(*b"who?"), "could not").unwrap_err();
        assert_eq!(error.to_string(), "could not (Core Audio status 'who?')");
        assert_eq!(
            status(-50, "could not").unwrap_err().to_string(),
            "could not (Core Audio status -50)"
        );
    }
}
