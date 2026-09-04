// The English catalog, and the shape every other catalog has to match.
//
// `Messages` is `typeof en`, so a locale that is missing a key, has an extra one, or types a
// message as a string where English takes parameters fails `npm run check` — the completeness
// check issue #23 asks for is the type system, with `i18n.test.ts` as a second pass for
// anything structural typing cannot see (an empty string, a key left in English).
//
// Conventions:
//
// - Parameterised messages are functions, so word order stays the translator's decision. A
//   language that puts the count last can do so; a `{0}` placeholder could not.
// - Anything with a plural takes the count and decides for itself. English needs two forms,
//   French agrees differently, and neither is the other's business.
// - Proper nouns (Gemini, OpenAI, Voxtral, WASAPI, F2) are repeated in every catalog rather
//   than pulled out, because deciding whether a product name is translated is exactly the
//   kind of thing a translator has to be able to decide.

export const en = {
	locale: {
		/** Name of this language, written in this language, for the language selector. */
		name: 'English',
		/** BCP 47 tag used for dates, times and number formatting. */
		tag: 'en-GB',
		label: 'Interface language',
		note: 'Changes the app, not the captions.'
	},

	app: {
		name: 'Live Captions',
		tagline: 'Realtime translation & subtitles'
	},

	state: {
		idle: 'Idle',
		connecting: 'Connecting',
		running: 'Live',
		reconnecting: 'Reconnecting',
		error: 'Error',
		/** Shown in place of "Live" while the bundled demonstration is playing. */
		demo: 'Demo'
	},

	/** Spoken by the status region on a state change. Never drawn. */
	announce: {
		idle: 'Session idle.',
		connecting: 'Connecting to the caption engine.',
		running: 'Captions are live.',
		reconnecting: 'Connection lost — reconnecting.',
		error: 'Session error.'
	},

	mode: {
		translate: 'Translation',
		transcribe: 'Subtitles'
	},

	source: {
		microphone: 'Room mic',
		system: 'System audio',
		both: 'Both',
		/** The microphone tile and meter while the built-in demonstration is selected. */
		demo: 'Demo audio',
		/** The source chip during a rehearsal, which plays a bundled recording. */
		sample: 'Sample'
	},

	engine: {
		gemini: 'Gemini',
		'gemini-transcribe': 'Gemini',
		openai: 'OpenAI',
		mistral: 'Voxtral',
		ondevice: 'Built-in demo'
	},

	language: {
		en: 'English',
		fr: 'Français',
		/** Target-language chip when the engine detects the spoken language itself. */
		auto: 'Auto'
	},

	provider: {
		vendor: {
			gemini: 'Google Gemini',
			'gemini-transcribe': 'Google Gemini',
			openai: 'OpenAI',
			mistral: 'Mistral Voxtral',
			ondevice: 'Built-in demo'
		},
		/** Shown in the mono face under the vendor. Model ids are not translated; the
		 *  demonstration has no model, so it describes itself instead. */
		model: {
			ondevice: 'Bundled sample · deterministic'
		},
		costNote: {
			gemini:
				'Gemini: input billed on wall clock, output only while it translates — pauses and slide changes lower this.',
			'gemini-transcribe':
				'Gemini: audio in is billed on wall clock, transcript text only while someone is speaking.',
			openai:
				'OpenAI: audio in and text out are billed per minute for as long as the stream stays open.',
			mistral:
				'Voxtral: billed per minute of audio streamed, for as long as the session stays open.',
			ondevice:
				'Bundled demonstration: no live audio is captured, no service is contacted, and nothing is billed.'
		}
	},

	cost: {
		/** Appended to a published rate: "$3.06" + "/hr". */
		perHour: '/hr',
		free: 'Free',
		elapsed: 'Elapsed',
		streamed: 'Streamed',
		estimate: 'Est. cost',
		twoSources: '×2 sources'
	},

	rail: {
		locked: 'Session locked',
		lockedNote: 'Stop the session to change any of these.',
		demoNote: 'A bundled demonstration is playing — no microphone or system audio is captured.',
		rehearsalNote: 'A sample recording is playing — nothing in the room is being captured.',
		chip: {
			mode: 'Mode',
			source: 'Source',
			roomReads: 'Room reads',
			engine: 'Engine'
		},
		arriving: 'Audio arriving',
		stop: 'Stop captions',
		stopping: 'Stopping…',

		step: {
			whatToShow: 'What to show',
			whereFrom: 'Where the audio comes from',
			engine: 'Engine',
			/** Step 03's heading, which asks a different question per mode. */
			roomReads: 'The room reads',
			demoLanguage: 'Demo language',
			spokenLanguage: 'Spoken language'
		},

		translate: {
			title: 'Live translation',
			desc: 'Speech is detected and translated into the language the room reads.'
		},
		transcribe: {
			title: 'Subtitles',
			desc: 'Built-in demonstration or live speech with Voxtral or Gemini. Saveable as text or Markdown.'
		},

		sourceHint:
			'System audio captures whatever is playing on this machine — Zoom, Teams, a browser tab, a media player.',
		demoSourceHint:
			'Uses a bundled deterministic sample. Choose Voxtral or Gemini for live microphone or system-audio subtitles.',
		micDevice: 'Microphone device',
		systemDefault: 'System default',
		/** Appended to the name of the device Windows would pick on its own. */
		isDefault: (name: string) => `${name} (default)`,

		autoDetectHint: (engine: string) =>
			`${engine} auto-detects the spoken language and writes same-language subtitles. No translation target is needed.`,
		flipHint: 'Speakers alternating? Swap it before you start with',
		flipKey: 'F2',
		demoLanguageHint:
			'Choose the bundled demonstration language. This mode demonstrates the overlay and export; it does not listen to the room.'
	},

	settings: {
		heading: 'Settings',
		openLabel: 'Open settings',
		close: 'Close',
		closeLabel: 'Close settings',
		appearance: 'Caption appearance',
		/** Says where to look while choosing. The stand-in caption the overlay shows in
		 *  placement mode is set in whatever is chosen here, so placement is the preview. */
		appearanceNote:
			'Place the overlay to judge these on the projector — the stand-in caption is set in what you choose here.'
	},

	overlayControls: {
		heading: 'Overlay',
		captionSize: 'Caption size',
		smaller: 'Smaller captions',
		larger: 'Larger captions',
		captionWidth: 'Line width',
		narrower: 'Shorter caption lines',
		wider: 'Longer caption lines',
		captionFace: 'Caption typeface',
		faceDefault: (label: string) => `${label} (default)`,
		captionColour: 'Caption colour',
		scrimColour: 'Backing colour',
		scrimOpacity: 'Backing strength',
		weakerScrim: 'Weaker backing behind the captions',
		strongerScrim: 'Stronger backing behind the captions',
		contrast: (ratio: string) => `Contrast ${ratio}:1`,
		contrastOk: 'Readable over a bright slide and a dark one.',
		contrastLow: (step: string, target: string) =>
			`${step} falls below ${target}:1 over a bright or a dark slide.`,
		contrastStep: {
			live: 'The live caption',
			final: 'A finished line',
			label: 'The speaker label',
			lead: 'The previous line trailing in'
		},
		reset: 'Reset',
		resetLabel: 'Reset the overlay to how it looks by default',
		move: 'Move',
		done: 'Done',
		moveLabel: 'Move the overlay',
		moveDoneLabel: 'Finish moving the overlay',
		show: 'Show',
		hide: 'Hide',
		showLabel: 'Show the overlay',
		hideLabel: 'Hide the overlay'
	},

	window: {
		heading: 'Window',
		minimizeToTray: 'Minimize to tray',
		keepRunning: 'Keep running in the tray when I close this window',
		keepRunningNote:
			'Off by default, so the close button quits as usual. With it on, closing leaves the session captioning and the tray icon is how you get back.',
		needsDesktop: 'Needs the desktop app — a browser preview has no tray.'
	},

	stage: {
		browserBanner: {
			before: 'Running in a browser without the Tauri runtime — controls are disabled. Launch with',
			command: 'npm run tauri dev',
			after: 'for audio capture, translation, and subtitles.'
		},
		onScreen: 'On screen now',
		newestLast: 'Newest at the bottom',
		twoSpeakers: 'Two speakers · newest at the bottom',
		origin: {
			microphone: 'Room',
			system: 'Remote',
			demo: 'Demo'
		},
		originSub: {
			microphone: 'mic',
			system: 'system',
			demo: 'sample'
		},
		waitingTranslation: 'Translated captions will appear here and on the overlay.',
		waitingSubtitles: 'Live subtitles will appear here and on the overlay.',
		waitingDemo: 'Demonstration subtitles will appear here and on the overlay.'
	},

	preflight: {
		kicker: 'Pre-flight',
		heading: 'Ready when you are',
		intro:
			'Four checks, then one button. Everything on the left locks while captions are running, so nothing can be changed by accident mid-session.',

		demoRow: {
			title: 'Built-in demo · no key needed',
			checking: 'Checking the bundled demonstration…',
			/** The core names the readiness state; these word it. Kept here rather than in
			 *  `ondevice/mod.rs` so the French interface does not read them in English. */
			ready:
				'Ready — bundled sample captions can demonstrate the overlay without a microphone, account, key, language pack, or network.',
			checkFailed: 'The bundled demonstration could not be checked.'
		},

		audio: {
			/** Row title: names whatever is being checked. */
			title: {
				microphone: 'Room mic',
				system: 'System audio',
				both: 'Audio',
				demo: 'Demo audio'
			},
			/** What was heard, in the past tense, once a test has confirmed it. */
			heard: {
				microphone: 'The room mic was picking up sound',
				system: 'WASAPI loopback was receiving sound',
				both: 'Both the room mic and WASAPI loopback were receiving sound',
				demo: 'Bundled sample is ready — no microphone is opened'
			},
			/** The same fact in the present tense, while the test is still running. English
			 *  used to reach this by replacing "was" with "is"; that does not survive
			 *  translation, so both tenses are written out. */
			hearing: {
				microphone: 'The room mic is picking up sound — stop the test when you are satisfied',
				system: 'WASAPI loopback is receiving sound — stop the test when you are satisfied',
				both: 'Both the room mic and WASAPI loopback are receiving sound — stop the test when you are satisfied',
				demo: 'Bundled sample is ready — no microphone is opened'
			},
			listening: 'Listening — say something into the mic or play some audio',
			unchecked: 'Not checked yet — audio is only monitored during a test or a running session',
			test: 'Test audio',
			retest: 'Re-test',
			stopTest: 'Stop test'
		},

		overlay: {
			title: 'Overlay placement',
			placed: 'Placed — captions will appear where you locked them',
			unplaced: 'Not placed yet — captions will sit bottom-centre on this display',
			place: 'Place it',
			adjust: 'Adjust',
			done: 'Done',
			placeLabel: 'Place the overlay',
			adjustLabel: 'Adjust the overlay placement',
			doneLabel: 'Finish placing the overlay'
		},

		cost: {
			title: 'Running cost',
			billed: 'Billed per minute of streamed audio, for as long as the session is open',
			free: 'Built into the app — nothing is billed'
		},

		start: {
			translate: 'Start translating',
			subtitles: 'Start subtitles',
			demo: 'Start demo subtitles',
			starting: 'Starting…'
		},
		rehearse: {
			action: 'Rehearse',
			hint: 'Plays a bundled sample recording through the live pipeline — no microphone needed.',
			demoHint: 'Start demo subtitles already runs the bundled demonstration.'
		},
		privacy: {
			memoryOnly: 'Transcript is held in memory until you save it.',
			spooled: 'Transcript is held in memory and spooled locally until you save it.',
			demo: 'The bundled demo stays entirely inside the app.',
			cloud: (vendor: string) => `Nothing leaves the machine except audio to ${vendor}.`
		}
	},

	key: {
		title: (name: string) => `${name} key`,
		saved: 'Saved in Windows Credential Manager · read only by the Rust core',
		desc: {
			before: 'Stored in Windows Credential Manager, used only from the Rust core. Needs access to',
			after: '.'
		},
		getKey: 'Get a key',
		opensInBrowser: ' (opens in your browser)',
		placeholder: (name: string) => `Paste your ${name} API key`,
		save: 'Save',
		saving: 'Saving…',
		replace: 'Replace',
		remove: 'Remove',
		cancel: 'Cancel'
	},

	transcript: {
		heading: 'Transcript',
		lines: (n: number) => (n === 1 ? '1 line' : `${n} lines`),
		unsaved: 'Unsaved',
		saved: 'Saved',
		saveText: 'Save text',
		saveMarkdown: 'Save Markdown',
		clear: 'Clear',
		confirmClear: 'Discard unsaved lines?',
		savedTo: 'Saved to',
		savedAnnouncement: (path: string) => `Transcript saved to ${path}`,
		staleBefore: 'Lines added since the save to',
		staleAfter: 'are not on disk yet.',
		longSession: (threshold: number) =>
			`This is a long session and none of it has been saved since it grew past ${threshold} lines. Nothing is being dropped, but save it now so a crash cannot take it.`,
		emptyTranslate: 'Finalized translations collect here, ready to save as text or Markdown.',
		emptySubtitles: 'Finalized subtitles collect here, ready to save as text or Markdown.',
		side: {
			microphone: 'Room',
			system: 'Remote'
		},
		recovery: {
			title: 'Keep a local recovery copy while captioning',
			note: 'Writes the finalized lines to this PC every few seconds so a crash or a power cut cannot take the session. Never leaves the machine, holds no audio and no API key, and is deleted as soon as you save, clear, or switch this off.',
			needsDesktop: 'Needs the desktop app — a browser preview has nowhere to write it.'
		}
	},

	// What the saved file says. Issue #23 puts the transcript's headings in the interface
	// language: an operator working in French is writing a French document, and the file is
	// the part of this app that leaves the machine.
	export: {
		title: 'Live captions transcript',
		origin: {
			microphone: 'Microphone',
			system: 'System'
		}
	},

	prompt: {
		unsaved: {
			title: 'Save this transcript before closing?',
			sessionEnded: 'The session has been stopped and the last captions collected.',
			body: (lines: number) =>
				lines === 1
					? '1 line has not been saved. Closing without saving discards it.'
					: `${lines} lines have not been saved. Closing without saving discards them.`,
			failed: (error: string) => `Could not save: ${error}`,
			save: 'Save and close',
			saving: 'Saving…',
			discard: 'Discard and close',
			cancel: 'Cancel',
			note: 'Saves Markdown to your Documents folder. Cancel keeps the app open.'
		},
		recovery: {
			title: 'Recover the transcript from your last session?',
			body: (lines: number, savedAt: string) =>
				`The app closed with ${lines === 1 ? '1 unsaved line' : `${lines} unsaved lines`} still in the log. They were spooled locally at ${savedAt} and have not left this PC.`,
			restore: 'Restore the transcript',
			delete: 'Delete it',
			noteBefore: 'Either answer removes the spool file at',
			noteAfter:
				'. Restoring loads the lines back into the log, still unsaved, so you can save them where you want them.'
		},
		activeSession: {
			title: 'A caption session is running',
			body: (elapsed: string) =>
				`Captions have been live for ${elapsed}. Closing stops the session, waits for the last captions to arrive, and then quits.`,
			noteBefore:
				'To put the window away without stopping anything, use Minimize to tray — or turn on',
			noteEmphasis: 'Keep running in the tray when I close this window',
			noteAfter: '.',
			keep: 'Keep captioning',
			stop: 'Stop and close'
		},
		trayHide: {
			title: 'Live Translation will keep running',
			bodyBefore:
				'You asked for closing this window to leave the app running in the tray, so it will disappear from the taskbar',
			bodyRunning: 'and keep captioning.',
			bodyIdle: 'but stay ready.',
			bodyAfter:
				'Its icon stays in the notification area, next to the clock — open it, show or hide the overlay, stop the session, or quit from there.',
			noteBefore: 'Said once. Turn it off again with',
			noteEmphasis: 'Keep running in the tray when I close this window',
			noteAfter: '.',
			hide: 'Got it — hide to the tray',
			quit: 'Quit instead'
		}
	},

	overlay: {
		placeholder: (size: number) => `Captions will sit here, two lines at ${size} px.`,
		dragToPlace: 'Drag to place',
		moveMode: 'Move mode',
		paused: 'Captions are paused on the overlay',
		keysLocks: 'locks',
		keysCancels: 'cancels',
		keysNudge: 'nudge',
		keyEnter: 'Enter',
		keyEscape: 'Esc',
		keyArrows: 'Arrows',
		size: 'Size',
		smaller: 'Smaller captions',
		larger: 'Larger captions',
		snapToBottom: 'Snap to bottom',
		lock: 'Lock into place',
		origin: {
			microphone: 'Room',
			system: 'Remote'
		}
	},

	// One sentence per failure the core can name (`src-tauri/src/errors.rs`). The technical
	// detail is appended in parentheses by `describeError`, so none of these end in a full
	// stop. Keep the ids and the keys in step: `errors.test.ts` reads the Rust file and fails
	// if a failure the core can report has no sentence here.
	error: {
		deviceEnumeration: 'Windows would not list the microphones',
		keychain: 'Windows Credential Manager refused the request',
		demoUnavailable: 'The built-in demonstration could not be prepared',
		sessionStart: 'The session could not be started',
		audioTestStart: 'The audio test could not open that source',
		overlayWindow: 'The overlay window did not respond',
		transcriptDir: 'The transcript folder could not be created',
		transcriptWrite: 'The transcript could not be written',
		taskFailed: 'A background task stopped unexpectedly',
		micStream: 'The microphone stopped sending audio',
		micCapture:
			'Microphone capture failed. If access is blocked, enable it under Windows Settings > Privacy & security > Microphone (ms-settings:privacy-microphone), then start again',
		systemCapture: 'System audio capture failed',
		providerRejected: 'The provider rejected the connection — check the API key and model access',
		providerStopped: 'The provider ended the session',
		providerReconnecting: 'The connection dropped; reconnecting',
		/** Front-end only: the recovery spool is written from the operator window. */
		recoveryWrite: (detail: string) => `Recovery copy could not be written: ${detail}`
	}
};

export type Messages = typeof en;
