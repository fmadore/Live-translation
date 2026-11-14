// What a Mac says differently: the messages that name a Windows place (Credential Manager, the
// notification area, Windows Settings, "this PC") in the words macOS uses for its own (the
// Keychain, the menu bar, System Settings, "this Mac"), and how a Mac writes its modifier keys.
//
// Each catalog keeps one shape for every system (`Messages`); on a Mac `index.ts` lays these
// over it, so no component tests the system. `i18n.test.ts` checks that every path here exists
// in the catalog with the same kind of value, and that the three languages override the same
// messages.
//
// The words follow each language's macOS: System Settings / Réglages Système /
// Systemeinstellungen; Privacy & Security / Confidentialité et sécurité / Datenschutz &
// Sicherheit; Screen & System Audio Recording / Enregistrement de l’écran et de l’audio du
// système / Bildschirm- & Systemaudioaufnahme; Keychain / trousseau / Schlüsselbund; menu bar /
// barre des menus / Menüleiste.

import type { Messages } from './en';

/** A catalog's subset: any message may be left out; a message that is given keeps its kind,
 *  so a function stays a function with the same parameters. */
export type Overrides<T> = {
	[K in keyof T]?: T[K] extends (...args: never[]) => unknown
		? T[K]
		: T[K] extends object
			? Overrides<T[K]>
			: T[K];
};

export type MacMessages = Overrides<Messages>;

export const macEn: MacMessages = {
	usability: {
		silenced:
			'Sound is playing, but only silence arrives. Allow Live Translation under System Settings > Privacy & Security > Screen & System Audio Recording, then start again',
		profileHint:
			'Saves setup, appearance and overlay position on this Mac, without API keys. Load while stopped; check the selected devices before starting. Applications must be selected again.',
		profileMissing:
			'An audio device is unavailable; the macOS default is selected. Check it before starting.'
	},
	theme: {
		note: 'System follows the light or dark appearance set in macOS. The caption overlay keeps its own colours.'
	},
	// As a Mac writes a key combination: symbols, Shift before Command, nothing between them.
	keys: {
		ctrl: '⌘',
		shift: '⇧'
	},
	applications: {
		unsupported:
			'Application capture requires macOS 14.2 or later. Choose all output audio explicitly to use that mode.',
		hint: 'Captures the selected application and its helper processes, including its notifications. A browser selection may include multiple tabs. Safari plays through a process macOS shares with other apps: capture it with all output audio.',
		empty: 'No running applications found. Open the application and refresh.'
	},
	devices: {
		idleFallback:
			'A saved audio device is unavailable. The next session will use the macOS default.'
	},
	window: {
		minimizeToTray: 'Hide to the menu bar',
		keepRunning: 'Keep running in the menu bar when I close this window',
		keepRunningNote:
			'Off by default, so the close button quits as usual. With it on, closing leaves the session captioning, and the menu bar icon or the Dock is how you get back.',
		needsDesktop: 'Needs the desktop app — a browser preview has no menu bar icon.'
	},
	zoom: {
		privacy:
			'Sharing sends caption text to Zoom, even when captions are made on this Mac with Whisper.'
	},
	key: {
		saved: 'Saved in the macOS Keychain · read only by the Rust core',
		desc: {
			before: 'Stored in the macOS Keychain, used only from the Rust core. Needs access to'
		}
	},
	transcript: {
		recovery: {
			note: 'Writes the finalized lines to this Mac every few seconds so a crash or a power cut cannot take the session. Never leaves the machine, holds no audio and no API key, and is deleted as soon as you save, clear, or switch this off.'
		}
	},
	prompt: {
		recovery: {
			body: (lines: number, savedAt: string) =>
				lines === 1
					? `The app closed with 1 unsaved line still in the log. It was spooled locally at ${savedAt} and has not left this Mac.`
					: `The app closed with ${lines} unsaved lines still in the log. They were spooled locally at ${savedAt} and have not left this Mac.`
		},
		activeSession: {
			noteBefore:
				'To put the window away without stopping anything, use Hide to the menu bar — or turn on',
			noteEmphasis: 'Keep running in the menu bar when I close this window'
		},
		trayHide: {
			bodyBefore:
				'You asked for closing this window to leave the app running in the menu bar, so it will disappear from the screen',
			bodyAfter:
				'Its icon stays in the menu bar, at the top right of the screen — open it, show or hide the overlay, stop the session, or quit from there. Its Dock icon also brings the window back.',
			noteEmphasis: 'Keep running in the menu bar when I close this window',
			hide: 'Got it — hide to the menu bar'
		}
	},
	preflight: {
		audio: {
			// What Windows calls WASAPI loopback is a Core Audio tap here.
			heard: {
				system: 'System audio was receiving sound',
				both: 'Both the room mic and system audio were receiving sound'
			},
			hearing: {
				system: 'System audio is receiving sound — stop the test when you are satisfied',
				both: 'Both the room mic and system audio are receiving sound — stop the test when you are satisfied'
			},
			silenced:
				'Sound is playing, but only silence arrives. Allow Live Translation under System Settings > Privacy & Security > Screen & System Audio Recording, then start again'
		}
	},
	error: {
		deviceEnumeration: 'macOS could not list the audio devices',
		keychain: 'The macOS Keychain refused the request',
		openLink: 'macOS could not open the page in your browser',
		micStream:
			'The microphone stopped sending audio. Check the connection in System Settings > Sound, then retry',
		micCapture:
			'Microphone capture failed. If access is blocked, allow Live Translation under System Settings > Privacy & Security > Microphone, then start again',
		micNone: 'No microphone is connected to this Mac. Connect one, or choose System audio',
		systemCapture:
			'System audio capture failed. If access is blocked, allow Live Translation under System Settings > Privacy & Security > Screen & System Audio Recording; check the selected output in System Settings > Sound, then retry'
	}
};

export const macFr: MacMessages = {
	usability: {
		silenced:
			'Du son est diffusé, mais seul du silence arrive. Autorisez Live Translation dans Réglages Système > Confidentialité et sécurité > Enregistrement de l’écran et de l’audio du système, puis redémarrez',
		profileHint:
			'Enregistre la configuration, l’apparence et la position sur ce Mac, sans clé API. Chargez à l’arrêt et vérifiez les périphériques. Sélectionnez à nouveau l’application à capturer.',
		profileMissing:
			'Un périphérique audio est absent ; celui par défaut de macOS est sélectionné. Vérifiez-le avant de démarrer.'
	},
	theme: {
		note: '« Système » suit l’apparence claire ou sombre de macOS. La surimpression des sous-titres garde ses propres couleurs.'
	},
	keys: {
		ctrl: '⌘',
		shift: '⇧'
	},
	applications: {
		unsupported:
			'La capture par application nécessite macOS 14.2 ou ultérieur. Choisissez explicitement tout le son de la sortie pour utiliser ce mode.',
		hint: 'Capture l’application sélectionnée et ses processus auxiliaires, y compris ses notifications. Pour un navigateur, plusieurs onglets peuvent être inclus. Safari passe par un processus que macOS partage avec d’autres apps : capturez-le avec tout le son de la sortie.',
		empty: 'Aucune application ouverte. Ouvrez l’application, puis actualisez.'
	},
	devices: {
		idleFallback:
			'Un appareil audio mémorisé est indisponible. La prochaine session utilisera le choix par défaut de macOS.'
	},
	window: {
		minimizeToTray: 'Masquer dans la barre des menus',
		keepRunning: 'Garder l’application active dans la barre des menus quand je ferme cette fenêtre',
		keepRunningNote:
			'Désactivé par défaut : le bouton de fermeture quitte l’application comme d’habitude. Activé, fermer la fenêtre laisse la session sous-titrer, et l’icône de la barre des menus ou le Dock est le moyen d’y revenir.',
		needsDesktop:
			'Nécessite l’application de bureau — un aperçu dans un navigateur n’a pas d’icône dans la barre des menus.'
	},
	zoom: {
		privacy:
			'Le partage envoie le texte des sous-titres à Zoom, même quand ils sont produits sur ce Mac par Whisper.'
	},
	key: {
		saved: 'Enregistrée dans le trousseau macOS · lue uniquement par le cœur Rust',
		desc: {
			before:
				'Enregistrée dans le trousseau macOS, utilisée uniquement depuis le cœur Rust. Nécessite l’accès à'
		}
	},
	transcript: {
		recovery: {
			note: 'Écrit les lignes finalisées sur ce Mac toutes les quelques secondes, afin qu’un plantage ou une coupure de courant n’emporte pas la session. Ne quitte jamais la machine, ne contient ni audio ni clé API, et est supprimée dès que vous enregistrez, effacez ou désactivez cette option.'
		}
	},
	prompt: {
		recovery: {
			body: (lines: number, savedAt: string) =>
				lines === 1
					? `L’application s’est fermée alors que ${lines} ligne non enregistrée était encore dans le journal. Elle a été copiée localement à ${savedAt} et n’a pas quitté ce Mac.`
					: `L’application s’est fermée alors que ${lines} lignes non enregistrées étaient encore dans le journal. Elles ont été copiées localement à ${savedAt} et n’ont pas quitté ce Mac.`
		},
		activeSession: {
			noteBefore:
				'Pour ranger la fenêtre sans rien arrêter, utilisez Masquer dans la barre des menus — ou activez',
			noteEmphasis:
				'Garder l’application active dans la barre des menus quand je ferme cette fenêtre'
		},
		trayHide: {
			bodyBefore:
				'Vous avez demandé que la fermeture de cette fenêtre laisse l’application active dans la barre des menus : elle va donc disparaître de l’écran',
			bodyAfter:
				'Son icône reste dans la barre des menus, en haut à droite de l’écran — vous pouvez y ouvrir l’application, afficher ou masquer la surimpression, arrêter la session ou quitter. Son icône dans le Dock ramène aussi la fenêtre.',
			noteEmphasis:
				'Garder l’application active dans la barre des menus quand je ferme cette fenêtre',
			hide: 'Compris — masquer dans la barre des menus'
		}
	},
	preflight: {
		audio: {
			// What Windows calls WASAPI loopback is a Core Audio tap here.
			heard: {
				system: 'L’audio système recevait du son',
				both: 'Le micro de la salle et l’audio système recevaient tous les deux du son'
			},
			hearing: {
				system: 'L’audio système reçoit du son — arrêtez le test quand vous êtes satisfait',
				both: 'Le micro de la salle et l’audio système reçoivent tous les deux du son — arrêtez le test quand vous êtes satisfait'
			},
			silenced:
				'Du son est diffusé, mais seul du silence arrive. Autorisez Live Translation dans Réglages Système > Confidentialité et sécurité > Enregistrement de l’écran et de l’audio du système, puis redémarrez'
		}
	},
	error: {
		deviceEnumeration: 'macOS n’a pas pu énumérer les appareils audio',
		keychain: 'Le trousseau macOS a refusé la demande',
		openLink: 'macOS n’a pas pu ouvrir la page dans votre navigateur',
		micStream:
			'Le microphone a cessé d’envoyer de l’audio. Vérifiez la connexion dans Réglages Système > Son, puis réessayez',
		micCapture:
			'La capture du microphone a échoué. Si l’accès est bloqué, autorisez Live Translation dans Réglages Système > Confidentialité et sécurité > Microphone, puis redémarrez',
		micNone: 'Aucun microphone n’est connecté à ce Mac. Branchez-en un ou choisissez Audio système',
		systemCapture:
			'La capture de l’audio système a échoué. Si l’accès est bloqué, autorisez Live Translation dans Réglages Système > Confidentialité et sécurité > Enregistrement de l’écran et de l’audio du système ; vérifiez la sortie sélectionnée dans Réglages Système > Son, puis réessayez'
	}
};

export const macDe: MacMessages = {
	usability: {
		silenced:
			'Es wird Ton abgespielt, aber es kommt nur Stille an. Erlauben Sie Live Translation unter Systemeinstellungen > Datenschutz & Sicherheit > Bildschirm- & Systemaudioaufnahme und starten Sie erneut',
		profileHint:
			'Speichert Einstellungen, Aussehen und Overlay-Position auf diesem Mac, ohne API-Schlüssel. Nur bei gestoppter Sitzung laden und Geräte prüfen. Anwendungen müssen erneut ausgewählt werden.',
		profileMissing: 'Ein Audiogerät fehlt; der macOS-Standard ist ausgewählt. Vor dem Start prüfen.'
	},
	theme: {
		note: 'System folgt dem hellen oder dunklen Erscheinungsbild von macOS. Das Untertitel-Overlay behält seine eigenen Farben.'
	},
	keys: {
		ctrl: '⌘',
		shift: '⇧'
	},
	applications: {
		unsupported:
			'Die Anwendungsaufnahme erfordert macOS 14.2 oder neuer. Wählen Sie ausdrücklich das gesamte Ausgabeaudio, um diesen Modus zu verwenden.',
		hint: 'Nimmt die ausgewählte Anwendung und ihre Hilfsprozesse auf, einschließlich ihrer Benachrichtigungen. Bei einem Browser können mehrere Tabs enthalten sein. Safari spielt über einen Prozess, den macOS mit anderen Apps teilt: Nehmen Sie Safari mit dem gesamten Ausgabeaudio auf.',
		empty: 'Keine geöffneten Anwendungen gefunden. Öffnen Sie die Anwendung und aktualisieren Sie.'
	},
	devices: {
		idleFallback:
			'Ein gespeichertes Audiogerät ist nicht verfügbar. Die nächste Sitzung verwendet den macOS-Standard.'
	},
	window: {
		minimizeToTray: 'In der Menüleiste ausblenden',
		keepRunning: 'In der Menüleiste weiterlaufen lassen, wenn ich dieses Fenster schließe',
		keepRunningNote:
			'Standardmäßig aus, sodass die Schaltfläche zum Schließen wie gewohnt beendet. Ist die Option an, untertitelt die Sitzung nach dem Schließen weiter, und das Symbol in der Menüleiste oder das Dock ist der Weg zurück.',
		needsDesktop: 'Erfordert die Desktop-App – eine Browser-Vorschau hat kein Menüleistensymbol.'
	},
	zoom: {
		privacy:
			'Beim Teilen wird Untertiteltext an Zoom gesendet, auch wenn die Untertitel mit Whisper auf diesem Mac entstehen.'
	},
	key: {
		saved: 'Im macOS-Schlüsselbund gespeichert · nur vom Rust-Kern gelesen',
		desc: {
			before:
				'Im macOS-Schlüsselbund gespeichert, nur vom Rust-Kern verwendet. Benötigt Zugriff auf'
		}
	},
	transcript: {
		recovery: {
			note: 'Schreibt die fertigen Zeilen alle paar Sekunden auf diesen Mac, damit ein Absturz oder ein Stromausfall die Sitzung nicht mitnimmt. Verlässt nie den Rechner, enthält weder Audio noch API-Schlüssel und wird gelöscht, sobald Sie speichern, leeren oder dies ausschalten.'
		}
	},
	prompt: {
		recovery: {
			body: (lines: number, savedAt: string) =>
				lines === 1
					? `Die App wurde mit 1 nicht gespeicherten Zeile im Protokoll geschlossen. Sie wurde um ${savedAt} lokal zwischengespeichert und hat diesen Mac nicht verlassen.`
					: `Die App wurde mit ${lines} nicht gespeicherten Zeilen im Protokoll geschlossen. Sie wurden um ${savedAt} lokal zwischengespeichert und haben diesen Mac nicht verlassen.`
		},
		activeSession: {
			noteBefore:
				'Um das Fenster wegzulegen, ohne etwas zu beenden, verwenden Sie In der Menüleiste ausblenden – oder aktivieren Sie',
			noteEmphasis: 'In der Menüleiste weiterlaufen lassen, wenn ich dieses Fenster schließe'
		},
		trayHide: {
			bodyBefore:
				'Sie haben festgelegt, dass die App beim Schließen dieses Fensters in der Menüleiste weiterläuft, also verschwindet sie vom Bildschirm',
			bodyAfter:
				'Ihr Symbol bleibt in der Menüleiste oben rechts – von dort öffnen Sie die App, blenden das Overlay ein oder aus, beenden die Sitzung oder schließen die App. Auch das Symbol im Dock holt das Fenster zurück.',
			noteEmphasis: 'In der Menüleiste weiterlaufen lassen, wenn ich dieses Fenster schließe',
			hide: 'Verstanden – in die Menüleiste'
		}
	},
	preflight: {
		audio: {
			// What Windows calls WASAPI loopback is a Core Audio tap here.
			heard: {
				system: 'Das Systemaudio hat Ton empfangen',
				both: 'Sowohl das Raummikrofon als auch das Systemaudio haben Ton empfangen'
			},
			hearing: {
				system: 'Das Systemaudio empfängt Ton – beenden Sie den Test, wenn Sie zufrieden sind',
				both: 'Sowohl das Raummikrofon als auch das Systemaudio empfangen Ton – beenden Sie den Test, wenn Sie zufrieden sind'
			},
			silenced:
				'Es wird Ton abgespielt, aber es kommt nur Stille an. Erlauben Sie Live Translation unter Systemeinstellungen > Datenschutz & Sicherheit > Bildschirm- & Systemaudioaufnahme und starten Sie erneut'
		}
	},
	error: {
		deviceEnumeration: 'macOS konnte die Audiogeräte nicht auflisten',
		keychain: 'Der macOS-Schlüsselbund hat die Anfrage abgelehnt',
		openLink: 'macOS konnte die Seite nicht im Browser öffnen',
		micStream:
			'Das Mikrofon sendet kein Audio mehr. Prüfen Sie die Verbindung unter Systemeinstellungen > Ton und versuchen Sie es erneut',
		micCapture:
			'Die Mikrofonaufnahme ist fehlgeschlagen. Falls der Zugriff blockiert ist, erlauben Sie Live Translation unter Systemeinstellungen > Datenschutz & Sicherheit > Mikrofon und starten Sie erneut',
		micNone:
			'An diesen Mac ist kein Mikrofon angeschlossen. Schließen Sie eines an oder wählen Sie Systemaudio',
		systemCapture:
			'Die Aufnahme des Systemaudios ist fehlgeschlagen. Falls der Zugriff blockiert ist, erlauben Sie Live Translation unter Systemeinstellungen > Datenschutz & Sicherheit > Bildschirm- & Systemaudioaufnahme; prüfen Sie die ausgewählte Ausgabe unter Systemeinstellungen > Ton und versuchen Sie es erneut'
	}
};

/** `catalog` with `overrides` laid over it, as a new object; neither is changed. A message is
 *  a leaf whatever its kind, so a function replaces a function. */
export function withOverrides<T extends object>(catalog: T, overrides: Overrides<T>): T {
	const source: Record<string, unknown> = Object.fromEntries(Object.entries(catalog));
	const merged: Record<string, unknown> = { ...source };
	for (const [key, value] of Object.entries(overrides)) {
		const base = source[key];
		merged[key] =
			value && typeof value === 'object' && base && typeof base === 'object'
				? withOverrides(base, value as Overrides<object>)
				: value;
	}
	return merged as T;
}
