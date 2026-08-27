//! Deterministic, keyless product demonstration.
//!
//! The `ondevice` provider identifier is retained for settings compatibility, but this path
//! deliberately performs no speech recognition. It plays a bundled demonstration timeline
//! that exercises the same caption, overlay, level-meter, elapsed-time, and export paths as
//! a live provider. That makes first launch useful and testable on every supported Windows
//! architecture without a microphone, language pack, network connection, or API key.

use anyhow::Result;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::Receiver;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::audio::AudioChunk;
use crate::errors::AppError;
use crate::realtime::{emit_caption, TurnAccumulator};
use crate::types::{events, AudioLevel, Origin, SessionState, StatusUpdate, TargetLanguage};

pub struct OnDeviceConfig {
    pub origin: Origin,
    pub language: TargetLanguage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnDeviceReadiness {
    pub ready: bool,
    pub engine: String,
    pub state: String,
    pub can_prepare: bool,
    pub detail: String,
}

pub fn readiness(_app: &AppHandle) -> OnDeviceReadiness {
    OnDeviceReadiness {
        ready: true,
        engine: "built-in-demo".into(),
        state: "ready".into(),
        can_prepare: false,
        detail: "Ready — bundled sample captions can demonstrate the overlay without a microphone, account, key, language pack, or network.".into(),
    }
}

pub fn prepare() -> Result<()> {
    Ok(())
}

#[derive(Clone, Copy)]
struct DemoLine {
    partial: &'static str,
    final_text: &'static str,
}

const ENGLISH_SCRIPT: &[DemoLine] = &[
    DemoLine {
        partial: "This is a built-in demonstration",
        final_text: "This is a built-in demonstration of the live caption display.",
    },
    DemoLine {
        partial: "Captions appear in the operator window",
        final_text: "Captions appear in the operator window and in the presentation overlay.",
    },
    DemoLine {
        partial: "The elapsed time and audio level",
        final_text: "The elapsed time and audio level continue to update while the demo runs.",
    },
    DemoLine {
        partial: "You can stop and save",
        final_text: "You can stop the demonstration and save this transcript at any time.",
    },
];

const FRENCH_SCRIPT: &[DemoLine] = &[
    DemoLine {
        partial: "Ceci est une démonstration intégrée",
        final_text: "Ceci est une démonstration intégrée de l’affichage des sous-titres.",
    },
    DemoLine {
        partial: "Les sous-titres apparaissent dans la fenêtre",
        final_text:
            "Les sous-titres apparaissent dans la fenêtre de contrôle et dans la surimpression.",
    },
    DemoLine {
        partial: "Le temps écoulé et le niveau audio",
        final_text:
            "Le temps écoulé et le niveau audio continuent d’avancer pendant la démonstration.",
    },
    DemoLine {
        partial: "Vous pouvez arrêter et enregistrer",
        final_text:
            "Vous pouvez arrêter la démonstration et enregistrer cette transcription à tout moment.",
    },
];

fn script(language: TargetLanguage) -> &'static [DemoLine] {
    match language {
        TargetLanguage::En => ENGLISH_SCRIPT,
        TargetLanguage::Fr => FRENCH_SCRIPT,
    }
}

/// Run the built-in demonstration on the ordinary caption event path. The audio receiver is
/// intentionally unused: this provider owns a deterministic timeline and never opens a device.
pub async fn run_session(
    app: AppHandle,
    config: OnDeviceConfig,
    _audio_rx: Receiver<AudioChunk>,
    cancel: CancellationToken,
) {
    let origin = config.origin;
    emit_status(&app, SessionState::Connecting, None, origin);

    if cancellable_delay(&cancel, Duration::from_millis(250)).await {
        return;
    }
    emit_status(&app, SessionState::Running, None, origin);

    let mut acc = TurnAccumulator::default();
    let mut pulse_index = 0usize;

    'session: loop {
        for line in script(config.language) {
            // Make the input indicator visibly active before the first words appear.
            for _ in 0..5 {
                if emit_pulse(&app, origin, pulse_index, &cancel).await {
                    break 'session;
                }
                pulse_index += 1;
            }

            acc.translated = line.partial.into();
            emit_caption(&app, origin, &acc, false);

            for _ in 0..7 {
                if emit_pulse(&app, origin, pulse_index, &cancel).await {
                    break 'session;
                }
                pulse_index += 1;
            }

            acc.translated = line.final_text.into();
            emit_caption(&app, origin, &acc, true);
            acc.next_turn();

            if cancellable_delay(&cancel, Duration::from_millis(350)).await {
                break 'session;
            }
        }

        // Keep the feature alive until Stop is clicked and make repeated cycles obvious.
        if cancellable_delay(&cancel, Duration::from_millis(700)).await {
            break;
        }
    }

    let _ = app.emit(
        events::LEVEL,
        AudioLevel {
            source: origin,
            rms: 0.0,
            peak: 0.0,
        },
    );
}

async fn emit_pulse(
    app: &AppHandle,
    origin: Origin,
    pulse_index: usize,
    cancel: &CancellationToken,
) -> bool {
    const RMS: [f32; 8] = [0.08, 0.18, 0.31, 0.23, 0.42, 0.28, 0.15, 0.35];
    let rms = RMS[pulse_index % RMS.len()];
    let _ = app.emit(
        events::LEVEL,
        AudioLevel {
            source: origin,
            rms,
            peak: (rms + 0.19).min(0.92),
        },
    );
    cancellable_delay(cancel, Duration::from_millis(120)).await
}

async fn cancellable_delay(cancel: &CancellationToken, duration: Duration) -> bool {
    tokio::select! {
        _ = cancel.cancelled() => true,
        _ = sleep(duration) => false,
    }
}

fn emit_status(app: &AppHandle, state: SessionState, message: Option<AppError>, origin: Origin) {
    let _ = app.emit(
        events::STATUS,
        StatusUpdate {
            state,
            message,
            origin: Some(origin),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_demo_languages_have_complete_non_empty_lines() {
        for language in [TargetLanguage::En, TargetLanguage::Fr] {
            let lines = script(language);
            assert!(lines.len() >= 3);
            assert!(lines
                .iter()
                .all(|line| !line.partial.is_empty() && !line.final_text.is_empty()));
        }
    }

    #[test]
    fn setup_is_always_a_no_op() {
        assert!(prepare().is_ok());
    }
}
