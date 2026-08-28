// The French catalog.
//
// The wording follows the French Store copy in `docs/store-listing.md`, which was written
// first and is what a French-speaking operator will have read before installing:
// **surimpression** for the overlay, **transcription** for the transcript, **démonstration
// intégrée** for the built-in demo, **zone de notification** for the tray, **Gestionnaire
// d'informations d'identification Windows** for Credential Manager.
//
// Two typographic conventions, both invisible in a diff and both enforced by `i18n.test.ts`
// rather than left to good intentions:
//
// - The space before a colon, question mark, semicolon or exclamation mark is a
//   **non-breaking space** (U+00A0), as French requires. Without it a line break can strand
//   the punctuation on its own line — and in a rail this dense, that is where lines break.
// - The apostrophe is the typographic one (U+2019), not the straight quote, matching the
//   Store copy.
//
// Product names are not translated (Gemini, OpenAI, Voxtral, WASAPI, Zoom, Teams, Markdown),
// and neither are the two Windows setting paths an operator has to find on their own screen.

import type { Messages } from './en';

export const fr: Messages = {
	locale: {
		/** Name of this language, written in this language, for the language selector. */
		name: 'Français',
		/** BCP 47 tag used for dates, times and number formatting. */
		tag: 'fr-FR',
		label: 'Langue de l’interface',
		note: 'Change l’application, pas les sous-titres.'
	},

	// The product's own name is not translated — the Store lists it in English, and inventing a
	// French one would give the same app a third name. The line under it is prose, so it is.
	app: {
		name: 'Live Captions',
		tagline: 'Traduction et sous-titrage en temps réel'
	},

	state: {
		idle: 'Inactif',
		connecting: 'Connexion',
		running: 'En direct',
		reconnecting: 'Reconnexion',
		error: 'Erreur',
		/** Shown in place of "Live" while the bundled demonstration is playing. */
		demo: 'Démo'
	},

	/** Spoken by the status region on a state change. Never drawn. */
	announce: {
		idle: 'Session inactive.',
		connecting: 'Connexion au moteur de sous-titrage.',
		running: 'Les sous-titres sont en direct.',
		reconnecting: 'Connexion perdue — reconnexion en cours.',
		error: 'Erreur de session.'
	},

	mode: {
		translate: 'Traduction',
		transcribe: 'Sous-titres'
	},

	source: {
		microphone: 'Micro de la salle',
		system: 'Audio système',
		both: 'Les deux',
		/** The microphone tile and meter while the built-in demonstration is selected. */
		demo: 'Audio de démo',
		/** The source chip during a rehearsal, which plays a bundled recording. */
		sample: 'Échantillon'
	},

	engine: {
		gemini: 'Gemini',
		'gemini-transcribe': 'Gemini',
		openai: 'OpenAI',
		mistral: 'Voxtral',
		ondevice: 'Démo intégrée'
	},

	language: {
		en: 'Anglais',
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
			ondevice: 'Démo intégrée'
		},
		/** Shown in the mono face under the vendor. Model ids are not translated; the
		 *  demonstration has no model, so it describes itself instead. */
		model: {
			ondevice: 'Échantillon fourni · déterministe'
		},
		costNote: {
			gemini:
				'Gemini : l’entrée est facturée au temps réel, la sortie uniquement pendant la traduction — les pauses et les changements de diapositive font baisser ce montant.',
			'gemini-transcribe':
				'Gemini : l’audio entrant est facturé au temps réel, le texte de la transcription uniquement lorsque quelqu’un parle.',
			openai:
				'OpenAI : l’audio entrant et le texte sortant sont facturés à la minute tant que le flux reste ouvert.',
			mistral: 'Voxtral : facturé à la minute d’audio transmis, tant que la session reste ouverte.',
			ondevice:
				'Démonstration intégrée : aucun audio en direct n’est capté, aucun service n’est contacté et rien n’est facturé.'
		}
	},

	cost: {
		/** Appended to a published rate: "$3.06" + "/hr". */
		perHour: '/h',
		free: 'Gratuit',
		elapsed: 'Écoulé',
		streamed: 'Transmis',
		estimate: 'Coût est.',
		twoSources: '×2 sources'
	},

	rail: {
		locked: 'Session verrouillée',
		lockedNote: 'Arrêtez la session pour modifier l’un de ces réglages.',
		demoNote:
			'Une démonstration intégrée est en cours — ni le microphone ni l’audio système ne sont captés.',
		rehearsalNote:
			'Un enregistrement d’échantillon est en cours — rien de ce qui se dit dans la salle n’est capté.',
		chip: {
			mode: 'Mode',
			source: 'Source',
			roomReads: 'La salle lit',
			engine: 'Moteur'
		},
		arriving: 'Audio entrant',
		stop: 'Arrêter les sous-titres',
		stopping: 'Arrêt…',

		step: {
			whatToShow: 'Ce qu’il faut afficher',
			whereFrom: 'D’où vient l’audio',
			engine: 'Moteur',
			/** Step 03's heading, which asks a different question per mode. */
			roomReads: 'La salle lit',
			demoLanguage: 'Langue de la démo',
			spokenLanguage: 'Langue parlée'
		},

		translate: {
			title: 'Traduction en direct',
			desc: 'La parole est détectée puis traduite dans la langue que lit la salle.'
		},
		transcribe: {
			title: 'Sous-titres',
			desc: 'Démonstration intégrée ou parole en direct avec Voxtral ou Gemini. Enregistrables en texte ou en Markdown.'
		},

		sourceHint:
			'L’audio système capte tout ce qui est joué sur cette machine — Zoom, Teams, un onglet de navigateur, un lecteur multimédia.',
		demoSourceHint:
			'Utilise un échantillon déterministe fourni avec l’application. Choisissez Voxtral ou Gemini pour des sous-titres en direct du microphone ou de l’audio système.',
		micDevice: 'Périphérique microphone',
		systemDefault: 'Périphérique par défaut',
		/** Appended to the name of the device Windows would pick on its own. */
		isDefault: (name: string) => `${name} (par défaut)`,

		autoDetectHint: (engine: string) =>
			`${engine} détecte lui-même la langue parlée et écrit les sous-titres dans cette même langue. Aucune langue cible n’est nécessaire.`,
		flipHint: 'Des intervenants qui alternent ? Changez avant de démarrer avec',
		flipKey: 'F2',
		demoLanguageHint:
			'Choisissez la langue de la démonstration intégrée. Ce mode illustre la surimpression et l’export ; il n’écoute pas la salle.'
	},

	settings: {
		heading: 'Paramètres',
		openLabel: 'Ouvrir les paramètres',
		close: 'Fermer',
		closeLabel: 'Fermer les paramètres',
		appearance: 'Apparence des sous-titres',
		/** Says where to look while choosing. The stand-in caption the overlay shows in
		 *  placement mode is set in whatever is chosen here, so placement is the preview. */
		appearanceNote:
			'Placez la surimpression pour juger ces réglages sur le projecteur — le sous-titre témoin adopte ce qui est choisi ici.'
	},

	overlayControls: {
		heading: 'Surimpression',
		captionSize: 'Taille des sous-titres',
		smaller: 'Réduire les sous-titres',
		larger: 'Agrandir les sous-titres',
		captionWidth: 'Largeur des lignes',
		narrower: 'Raccourcir les lignes de sous-titres',
		wider: 'Allonger les lignes de sous-titres',
		captionFace: 'Police des sous-titres',
		faceDefault: (label: string) => `${label} (par défaut)`,
		captionColour: 'Couleur des sous-titres',
		scrimColour: 'Couleur du fond',
		scrimOpacity: 'Intensité du fond',
		weakerScrim: 'Fond plus discret derrière les sous-titres',
		strongerScrim: 'Fond plus marqué derrière les sous-titres',
		contrast: (ratio: string) => `Contraste ${ratio}:1`,
		contrastOk: 'Lisible sur une diapositive claire comme sur une sombre.',
		contrastLow: (step: string, target: string) =>
			`${step} descend sous ${target}:1 sur une diapositive claire ou sombre.`,
		contrastStep: {
			live: 'Le sous-titre en cours',
			final: 'Une ligne terminée',
			label: 'L’étiquette du locuteur',
			lead: 'La ligne précédente en rappel'
		},
		reset: 'Rétablir',
		resetLabel: 'Rétablir l’apparence par défaut de la surimpression',
		move: 'Déplacer',
		done: 'Terminé',
		moveLabel: 'Déplacer la surimpression',
		moveDoneLabel: 'Terminer le déplacement de la surimpression',
		show: 'Afficher',
		hide: 'Masquer',
		showLabel: 'Afficher la surimpression',
		hideLabel: 'Masquer la surimpression'
	},

	window: {
		heading: 'Fenêtre',
		minimizeToTray: 'Réduire dans la zone de notification',
		keepRunning:
			'Garder l’application active dans la zone de notification quand je ferme cette fenêtre',
		keepRunningNote:
			'Désactivé par défaut : le bouton de fermeture quitte l’application comme d’habitude. Activé, fermer la fenêtre laisse la session sous-titrer, et l’icône de la zone de notification est le moyen d’y revenir.',
		needsDesktop:
			'Nécessite l’application de bureau — un aperçu dans un navigateur n’a pas de zone de notification.'
	},

	stage: {
		browserBanner: {
			before:
				'Exécution dans un navigateur sans le runtime Tauri — les commandes sont désactivées. Lancez',
			command: 'npm run tauri dev',
			after: 'pour la capture audio, la traduction et les sous-titres.'
		},
		onScreen: 'À l’écran maintenant',
		newestLast: 'Le plus récent en bas',
		twoSpeakers: 'Deux intervenants · le plus récent en bas',
		origin: {
			microphone: 'Salle',
			system: 'Distant',
			demo: 'Démo'
		},
		originSub: {
			microphone: 'micro',
			system: 'système',
			demo: 'échantillon'
		},
		waitingTranslation: 'Les sous-titres traduits apparaîtront ici et dans la surimpression.',
		waitingSubtitles: 'Les sous-titres en direct apparaîtront ici et dans la surimpression.',
		waitingDemo: 'Les sous-titres de démonstration apparaîtront ici et dans la surimpression.'
	},

	preflight: {
		kicker: 'Avant de démarrer',
		heading: 'Prêt quand vous l’êtes',
		intro:
			'Quatre vérifications, puis un seul bouton. Tout ce qui se trouve à gauche se verrouille pendant le sous-titrage : rien ne peut être modifié par accident en pleine session.',

		demoRow: {
			title: 'Démo intégrée · aucune clé nécessaire',
			checking: 'Vérification de la démonstration intégrée…'
		},

		audio: {
			/** Row title: names whatever is being checked. */
			title: {
				microphone: 'Micro de la salle',
				system: 'Audio système',
				both: 'Audio',
				demo: 'Audio de démo'
			},
			/** What was heard, in the past tense, once a test has confirmed it. */
			heard: {
				microphone: 'Le micro de la salle captait du son',
				system: 'Le rebouclage WASAPI recevait du son',
				both: 'Le micro de la salle et le rebouclage WASAPI recevaient tous les deux du son',
				demo: 'L’échantillon fourni est prêt — aucun microphone n’est ouvert'
			},
			/** The same fact in the present tense, while the test is still running. English
			 *  used to reach this by replacing "was" with "is"; that does not survive
			 *  translation, so both tenses are written out. */
			hearing: {
				microphone: 'Le micro de la salle capte du son — arrêtez le test quand vous êtes satisfait',
				system: 'Le rebouclage WASAPI reçoit du son — arrêtez le test quand vous êtes satisfait',
				both: 'Le micro de la salle et le rebouclage WASAPI reçoivent tous les deux du son — arrêtez le test quand vous êtes satisfait',
				demo: 'L’échantillon fourni est prêt — aucun microphone n’est ouvert'
			},
			listening: 'Écoute en cours — parlez dans le micro ou lancez un son',
			unchecked:
				'Pas encore vérifié — l’audio n’est surveillé que pendant un test ou une session en cours',
			test: 'Tester l’audio',
			retest: 'Retester',
			stopTest: 'Arrêter le test'
		},

		overlay: {
			title: 'Placement de la surimpression',
			placed: 'Placée — les sous-titres apparaîtront là où vous les avez verrouillés',
			unplaced: 'Pas encore placée — les sous-titres se placeront en bas au centre de cet écran',
			place: 'Placer',
			adjust: 'Ajuster',
			done: 'Terminé',
			placeLabel: 'Placer la surimpression',
			adjustLabel: 'Ajuster le placement de la surimpression',
			doneLabel: 'Terminer le placement de la surimpression'
		},

		cost: {
			title: 'Coût en cours',
			billed: 'Facturé à la minute d’audio transmis, tant que la session reste ouverte',
			free: 'Intégré à l’application — rien n’est facturé'
		},

		start: {
			translate: 'Démarrer la traduction',
			subtitles: 'Démarrer les sous-titres',
			demo: 'Démarrer la démo',
			starting: 'Démarrage…'
		},
		rehearse: {
			action: 'Répéter',
			hint: 'Fait passer un enregistrement fourni dans la chaîne en direct — aucun microphone nécessaire.',
			demoHint: 'Démarrer la démo lance déjà la démonstration intégrée.'
		},
		privacy: {
			memoryOnly: 'La transcription reste en mémoire jusqu’à ce que vous l’enregistriez.',
			spooled:
				'La transcription reste en mémoire et est copiée localement jusqu’à ce que vous l’enregistriez.',
			demo: 'La démo intégrée reste entièrement à l’intérieur de l’application.',
			cloud: (vendor: string) => `Rien ne quitte la machine, hormis l’audio envoyé à ${vendor}.`
		}
	},

	key: {
		title: (name: string) => `Clé ${name}`,
		saved:
			'Enregistrée dans le Gestionnaire d’informations d’identification Windows · lue uniquement par le cœur Rust',
		desc: {
			before:
				'Enregistrée dans le Gestionnaire d’informations d’identification Windows, utilisée uniquement depuis le cœur Rust. Nécessite l’accès à',
			after: '.'
		},
		getKey: 'Obtenir une clé',
		opensInBrowser: ' (s’ouvre dans votre navigateur)',
		placeholder: (name: string) => `Collez votre clé API ${name}`,
		save: 'Enregistrer',
		saving: 'Enregistrement…',
		replace: 'Remplacer',
		remove: 'Supprimer',
		cancel: 'Annuler'
	},

	transcript: {
		heading: 'Transcription',
		lines: (n: number) => (n === 1 ? '1 ligne' : `${n} lignes`),
		unsaved: 'Non enregistrée',
		saved: 'Enregistrée',
		saveText: 'Enregistrer en texte',
		saveMarkdown: 'Enregistrer en Markdown',
		clear: 'Effacer',
		confirmClear: 'Supprimer les lignes non enregistrées ?',
		savedTo: 'Enregistrée dans',
		savedAnnouncement: (path: string) => `Transcription enregistrée dans ${path}`,
		staleBefore: 'Les lignes ajoutées depuis l’enregistrement dans',
		staleAfter: 'ne sont pas encore sur le disque.',
		longSession: (threshold: number) =>
			`Cette session est longue et rien n’en a été enregistré depuis qu’elle a dépassé ${threshold} lignes. Rien n’est supprimé, mais enregistrez-la maintenant pour qu’un plantage ne l’emporte pas.`,
		emptyTranslate:
			'Les traductions finalisées s’accumulent ici, prêtes à être enregistrées en texte ou en Markdown.',
		emptySubtitles:
			'Les sous-titres finalisés s’accumulent ici, prêts à être enregistrés en texte ou en Markdown.',
		side: {
			microphone: 'Salle',
			system: 'Distant'
		},
		recovery: {
			title: 'Conserver une copie de récupération locale pendant le sous-titrage',
			note: 'Écrit les lignes finalisées sur ce PC toutes les quelques secondes, afin qu’un plantage ou une coupure de courant n’emporte pas la session. Ne quitte jamais la machine, ne contient ni audio ni clé API, et est supprimée dès que vous enregistrez, effacez ou désactivez cette option.',
			needsDesktop:
				'Nécessite l’application de bureau — un aperçu dans un navigateur n’a nulle part où l’écrire.'
		}
	},

	// What the saved file says. Issue #23 puts the transcript's headings in the interface
	// language: an operator working in French is writing a French document, and the file is
	// the part of this app that leaves the machine.
	export: {
		title: 'Transcription des sous-titres en direct',
		origin: {
			microphone: 'Microphone',
			system: 'Système'
		}
	},

	prompt: {
		unsaved: {
			title: 'Enregistrer cette transcription avant de fermer ?',
			sessionEnded: 'La session a été arrêtée et les derniers sous-titres ont été récupérés.',
			body: (lines: number) =>
				lines === 1
					? '1 ligne n’a pas été enregistrée. Fermer sans enregistrer la supprime.'
					: `${lines} lignes n’ont pas été enregistrées. Fermer sans enregistrer les supprime.`,
			failed: (error: string) => `Enregistrement impossible : ${error}`,
			save: 'Enregistrer et fermer',
			saving: 'Enregistrement…',
			discard: 'Supprimer et fermer',
			cancel: 'Annuler',
			note: 'Enregistre un fichier Markdown dans votre dossier Documents. Annuler laisse l’application ouverte.'
		},
		recovery: {
			title: 'Récupérer la transcription de votre dernière session ?',
			body: (lines: number, savedAt: string) =>
				`L’application s’est fermée alors que ${lines === 1 ? '1 ligne non enregistrée était encore' : `${lines} lignes non enregistrées étaient encore`} dans le journal. Elles ont été copiées localement à ${savedAt} et n’ont pas quitté ce PC.`,
			restore: 'Restaurer la transcription',
			delete: 'La supprimer',
			noteBefore: 'Les deux réponses suppriment le fichier de récupération situé à',
			noteAfter:
				'. Restaurer recharge les lignes dans le journal, toujours non enregistrées, pour que vous puissiez les enregistrer où vous le souhaitez.'
		},
		activeSession: {
			title: 'Une session de sous-titrage est en cours',
			body: (elapsed: string) =>
				`Les sous-titres sont en direct depuis ${elapsed}. Fermer arrête la session, attend les derniers sous-titres, puis quitte l’application.`,
			noteBefore:
				'Pour ranger la fenêtre sans rien arrêter, utilisez Réduire dans la zone de notification — ou activez',
			noteEmphasis:
				'Garder l’application active dans la zone de notification quand je ferme cette fenêtre',
			noteAfter: '.',
			keep: 'Continuer le sous-titrage',
			stop: 'Arrêter et fermer'
		},
		trayHide: {
			title: 'Live Translation va rester actif',
			bodyBefore:
				'Vous avez demandé que la fermeture de cette fenêtre laisse l’application active dans la zone de notification : elle va donc disparaître de la barre des tâches',
			bodyRunning: 'et continuer à sous-titrer.',
			bodyIdle: 'mais rester prête.',
			bodyAfter:
				'Son icône reste dans la zone de notification, à côté de l’horloge — vous pouvez y ouvrir l’application, afficher ou masquer la surimpression, arrêter la session ou quitter.',
			noteBefore: 'Dit une seule fois. Désactivez-le de nouveau avec',
			noteEmphasis:
				'Garder l’application active dans la zone de notification quand je ferme cette fenêtre',
			noteAfter: '.',
			hide: 'Compris — réduire dans la zone de notification',
			quit: 'Quitter plutôt'
		}
	},

	overlay: {
		placeholder: (size: number) =>
			`Les sous-titres se placeront ici, sur deux lignes en ${size} px.`,
		dragToPlace: 'Glisser pour placer',
		moveMode: 'Mode déplacement',
		paused: 'Les sous-titres sont en pause sur la surimpression',
		keysLocks: 'verrouille',
		keysCancels: 'annule',
		keysNudge: 'déplacent',
		keyEnter: 'Entrée',
		keyEscape: 'Échap',
		keyArrows: 'Flèches',
		size: 'Taille',
		smaller: 'Réduire les sous-titres',
		larger: 'Agrandir les sous-titres',
		snapToBottom: 'Aligner en bas',
		lock: 'Verrouiller en place',
		origin: {
			microphone: 'Salle',
			system: 'Distant'
		}
	},

	// One sentence per failure the core can name (`src-tauri/src/errors.rs`). The technical
	// detail is appended in parentheses by `describeError`, so none of these end in a full
	// stop. Keep the ids and the keys in step: `errors.test.ts` reads the Rust file and fails
	// if a failure the core can report has no sentence here.
	error: {
		deviceEnumeration: 'Windows n’a pas voulu énumérer les microphones',
		keychain: 'Le Gestionnaire d’informations d’identification Windows a refusé la demande',
		demoUnavailable: 'La démonstration intégrée n’a pas pu être préparée',
		sessionStart: 'La session n’a pas pu démarrer',
		audioTestStart: 'Le test audio n’a pas pu ouvrir cette source',
		overlayWindow: 'La fenêtre de surimpression n’a pas répondu',
		transcriptDir: 'Le dossier de transcription n’a pas pu être créé',
		transcriptWrite: 'La transcription n’a pas pu être écrite',
		taskFailed: 'Une tâche en arrière-plan s’est arrêtée de façon inattendue',
		micStream: 'Le microphone a cessé d’envoyer de l’audio',
		micCapture:
			'La capture du microphone a échoué. Si l’accès est bloqué, activez-le dans Paramètres Windows > Confidentialité et sécurité > Microphone (ms-settings:privacy-microphone), puis redémarrez',
		systemCapture: 'La capture de l’audio système a échoué',
		providerRejected:
			'Le fournisseur a refusé la connexion — vérifiez la clé API et l’accès au modèle',
		providerStopped: 'Le fournisseur a mis fin à la session',
		providerReconnecting: 'La connexion a été perdue ; reconnexion en cours',
		/** Front-end only: the recovery spool is written from the operator window. */
		recoveryWrite: (detail: string) => `La copie de récupération n’a pas pu être écrite : ${detail}`
	}
};
