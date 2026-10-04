# Partner Center listing text — 1.6.0

Prepared for **1.6.0 / MSIX 1.6.0.0**; not submitted. The last confirmed Store release is
1.5.1. Paste each block into its named field after the
[release checks](store-updates.md#release-160-handoff). These descriptions lead with the actual
captioning workflow; the scripted demo is a secondary setup aid.

Microsoft's [listing field documentation](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
allows 10,000 characters for Description, 1,500 for What's new, and 20 features of 200
characters each. Short description accepts 1,000 characters, but only the first 270 appear
in some views. All three short descriptions below fit within 270 characters.

## English (United States)

### Description — full text

```text
Follow meetings, lectures and video calls with captions over the content you are watching. Live Translation & Subtitles transcribes speech locally with Whisper after a one-time model download. Optional cloud translation and subtitles require your own Google Gemini, OpenAI or Mistral account and API key; provider usage charges may apply.

Place a transparent, always-on-top caption overlay over slides, a call or any other Windows app. Capture a room microphone, Windows system audio, one application's process tree, or microphone and system audio together. The overlay stays click-through while you work.

For offline subtitles, choose a multilingual Whisper model and one of 99 spoken languages, or use automatic detection. Tiny, Base and Small models download separately (about 31, 57 and 182 MiB). Once a model is installed, recognition runs on your CPU without an account, API key or internet connection. Speed and accuracy vary with the computer, model and language. Whisper transcribes in the spoken language; it does not translate.

For multilingual audiences, translate with Gemini or OpenAI. The app offers 78 Gemini and 13 OpenAI translation targets. Show two target languages at once, or display the original speech beneath a translation in Fit window or Compact when the engine provides it. A second target opens an additional cloud session for each audio source and increases provider usage. Gemini and Mistral also offer cloud subtitles in the spoken language.

Make captions comfortable to read with adjustable type, size, colour, contrast, persistence and update pace. Choose Fit window, Compact or Stable reading, including right-to-left text. Customize the words hidden by the optional filler filter while preserving the raw transcript. Reuse meeting profiles and pause during breaks without splitting the session.

Export transcripts as text, Markdown, SRT or WebVTT. Optional local history saves finalized captions progressively and lets you name, search, reopen, copy, export and delete sessions. History and crash recovery are off by default. With Whisper, Stop finishes any pending audio before the transcript is complete.

Whisper audio stays on the PC; pending speech is buffered in an automatically deleted temporary file. Cloud audio goes directly to your selected provider. API keys are stored in Windows Credential Manager. The developer receives no audio, keys, transcripts or telemetry. The app sells no subscriptions, credits or API access.

Use the interface in English, French or German, independently of the caption language. An optional English/French scripted demo lets you try the display controls without setup; it does not recognize speech.

Requires Windows 11 and Microsoft Edge WebView2 Runtime. Native x64 and ARM64 packages are available. Download a Whisper model before offline use, and allow access to the audio source you select. Capturing one application includes its child processes and notifications, and may include several browser tabs.
```

### Features — one feature per line

```text
Offline Whisper subtitles with 99 spoken-language choices or automatic detection
Download Tiny, Base or Small once, then transcribe without an account, API key or internet
Cloud translation with your own Gemini or OpenAI key; provider usage charges may apply
Two translation languages at once, plus optional original speech when available
Microphone, system audio, one application's process tree, or microphone and system audio together
Transparent, always-on-top, click-through captions over slides and video calls
Fit window, Compact and Stable reading, with adjustable appearance and right-to-left support
Pause and resume, reusable meeting profiles, searchable languages and favourites
Editable filler-word filter for the overlay, with raw transcripts preserved
Text, Markdown, SRT and WebVTT export, including available original speech
Optional local transcript history with titles, search, reopening and deletion
English, French and German interface; native Windows x64 and ARM64
```

### Short description

```text
Live captions over meetings, calls and slides. Transcribe offline with Whisper in 99 languages, or translate with cloud engines using your own API key. Capture mic or app audio, show two caption languages and export your transcript.
```

### What's new in this version — 1.6.0

```text
Whisper brings offline subtitles to your PC. Download a multilingual Tiny, Base or Small model, choose from 99 spoken languages or automatic detection, and caption microphone or system audio without an account or API key. Audio stays local. Speed and accuracy depend on your PC, model and language; Stop lets pending speech finish processing.

For bilingual meetings, show two translation languages at once or include available original speech beneath translations and in exports. Cloud translation uses your own provider key; a second target adds provider usage.

Pause and resume without splitting your transcript. Cloud connections close during breaks; Whisper keeps processing speech already queued. Edit the overlay's filler-word list while preserving the raw transcript.

New installs open on Whisper with Base and automatic language detection; existing saved setups are preserved. Also includes faster caption layout and history updates, improved Gemini reconnection and a more consistent operator interface. Native x64 and ARM64, with an English, French or German interface. Thanks to @valentinrabot for suggesting local transcription in issue #98.
```

## Français (France)

### Description — texte complet

```text
Suivez réunions, cours et appels vidéo grâce à des sous-titres affichés sur votre contenu. Live Translation & Subtitles transcrit la parole localement avec Whisper après le téléchargement initial d'un modèle. La traduction et les sous-titres cloud facultatifs nécessitent votre propre compte et clé API Google Gemini, OpenAI ou Mistral ; des frais d'utilisation peuvent s'appliquer.

Placez des sous-titres transparents, toujours visibles, au-dessus de diapositives, d'un appel ou d'une autre application Windows. Captez un microphone, l'audio système, une application et ses processus enfants, ou le microphone et l'audio système ensemble. Les clics traversent la surimpression pour vous laisser travailler.

Pour les sous-titres hors ligne, choisissez un modèle Whisper multilingue et l'une des 99 langues parlées proposées, ou la détection automatique. Les modèles Tiny, Base et Small se téléchargent séparément (environ 31, 57 et 182 Mio). Une fois le modèle installé, la reconnaissance utilise le processeur de votre PC, sans compte, clé API ni connexion Internet. La vitesse et la précision varient selon l'ordinateur, le modèle et la langue. Whisper transcrit dans la langue parlée ; il ne traduit pas.

Pour un public multilingue, traduisez avec Gemini ou OpenAI. L'application propose 78 langues cibles avec Gemini et 13 avec OpenAI. Affichez deux langues cibles à la fois, ou le texte original sous la traduction en mode Adapter à la fenêtre ou Compact lorsque le moteur le fournit. Une seconde langue ouvre une session cloud supplémentaire par source audio et augmente l'utilisation facturée. Gemini et Mistral proposent aussi des sous-titres cloud dans la langue parlée.

Réglez la police, la taille, la couleur, le contraste, la durée d'affichage et le rythme des mises à jour. Choisissez Adapter à la fenêtre, Compact ou Lecture stable, avec prise en charge des écritures de droite à gauche. Personnalisez les mots masqués par le filtre d'hésitations tout en conservant la transcription originale. Réutilisez vos profils de réunion et faites une pause sans scinder la session.

Exportez en texte, Markdown, SRT ou WebVTT. L'historique local facultatif enregistre progressivement les sous-titres finalisés et permet de nommer, rechercher, relire, copier, exporter et supprimer les sessions. L'historique et la copie de récupération sont désactivés par défaut. Avec Whisper, la commande Arrêter termine le traitement de l'audio en attente avant que la transcription soit complète.

L'audio Whisper reste sur le PC ; la parole en attente est placée dans un fichier temporaire supprimé automatiquement. L'audio cloud est envoyé directement au fournisseur choisi. Les clés API sont conservées dans le Gestionnaire d'informations d'identification Windows. Le développeur ne reçoit ni audio, ni clé, ni transcription, ni télémétrie. L'application ne vend ni abonnement, ni crédits, ni accès API.

Utilisez l'interface en français, anglais ou allemand, indépendamment de la langue des sous-titres. Une démo scénarisée facultative en français et en anglais permet d'essayer l'affichage sans configuration ; elle ne reconnaît pas la parole.

Nécessite Windows 11 et Microsoft Edge WebView2 Runtime. Des paquets x64 et ARM64 natifs sont disponibles. Téléchargez un modèle Whisper avant l'utilisation hors ligne et autorisez l'accès à la source audio choisie. La capture d'une application inclut ses processus enfants et notifications, et peut inclure plusieurs onglets d'un navigateur.
```

### Fonctionnalités — une fonctionnalité par ligne

```text
Sous-titres Whisper hors ligne : 99 langues parlées ou détection automatique
Téléchargez Tiny, Base ou Small, puis transcrivez sans compte, clé API ni connexion Internet
Traduction cloud avec votre clé Gemini ou OpenAI ; des frais d'utilisation peuvent s'appliquer
Deux langues de traduction à la fois et affichage facultatif du texte original disponible
Microphone, audio système, une application et ses processus enfants, ou microphone et audio système ensemble
Sous-titres transparents, toujours visibles et traversables par les clics, sur vos diapositives et appels
Adapter à la fenêtre, Compact et Lecture stable ; apparence réglable et écritures de droite à gauche
Pause et reprise, profils de réunion, recherche de langues et favoris
Liste d'hésitations modifiable pour la surimpression, avec conservation de la transcription originale
Export texte, Markdown, SRT et WebVTT, avec texte original lorsqu'il est disponible
Historique local facultatif avec titres, recherche, relecture et suppression
Interface en français, anglais et allemand ; versions Windows x64 et ARM64 natives
```

### Description courte

```text
Sous-titrez réunions, appels et présentations. Transcrivez hors ligne avec Whisper en 99 langues, ou traduisez via le cloud avec votre clé API. Captez le micro ou une application, affichez deux langues et exportez vos transcriptions.
```

### Nouveautés de cette version — 1.6.0

```text
Whisper apporte les sous-titres hors ligne sur votre PC. Téléchargez un modèle multilingue Tiny, Base ou Small, choisissez parmi 99 langues parlées ou la détection automatique, et transcrivez le micro ou l'audio système sans compte ni clé API. L'audio reste local. La vitesse et la précision dépendent du PC, du modèle et de la langue ; Arrêter laisse l'audio en attente finir son traitement.

Pour les réunions bilingues, affichez deux langues de traduction à la fois ou ajoutez le texte original disponible sous les traductions et dans les exports. La traduction cloud utilise votre clé API ; une seconde langue augmente l'utilisation du fournisseur.

Faites une pause puis reprenez sans scinder la transcription. Les connexions cloud se ferment pendant les pauses ; Whisper continue de traiter l'audio déjà en attente. Modifiez la liste d'hésitations de la surimpression sans altérer la transcription originale.

Les nouvelles installations s'ouvrent sur Whisper avec Base et la détection automatique ; les réglages enregistrés restent conservés. Inclut aussi un affichage et un historique plus efficaces, une reconnexion Gemini améliorée et une interface de contrôle harmonisée. Versions x64 et ARM64 natives ; interface en français, anglais ou allemand. Merci à @valentinrabot pour sa suggestion de transcription locale dans le ticket #98.
```

## Deutsch (Deutschland)

### Beschreibung — vollständiger Text

```text
Verfolgen Sie Besprechungen, Vorträge und Videoanrufe mit Untertiteln direkt über Ihren Inhalten. Live Translation & Subtitles transkribiert Sprache lokal mit Whisper nach einem einmaligen Modelldownload. Optionale Cloud-Übersetzungen und Cloud-Untertitel benötigen Ihr eigenes Konto mit API-Schlüssel bei Google Gemini, OpenAI oder Mistral; dabei können nutzungsabhängige Kosten entstehen.

Platzieren Sie ein transparentes Untertitel-Overlay über Folien, einem Anruf oder einer anderen Windows-App. Erfassen Sie ein Mikrofon, das Windows-Systemaudio, eine Anwendung mit ihren Unterprozessen oder Mikrofon und Systemaudio gemeinsam. Das Overlay bleibt im Vordergrund und lässt Mausklicks zur darunterliegenden Anwendung durch.

Für Offline-Untertitel wählen Sie ein mehrsprachiges Whisper-Modell und eine von 99 gesprochenen Sprachen oder die automatische Erkennung. Tiny, Base und Small werden separat heruntergeladen (etwa 31, 57 und 182 MiB). Danach läuft die Erkennung auf der CPU Ihres PCs, ohne Konto, API-Schlüssel oder Internetverbindung. Geschwindigkeit und Genauigkeit hängen von Computer, Modell und Sprache ab. Whisper transkribiert in der gesprochenen Sprache; es übersetzt nicht.

Für ein mehrsprachiges Publikum übersetzen Sie mit Gemini oder OpenAI. Die App bietet 78 Zielsprachen für Gemini und 13 für OpenAI. Zeigen Sie zwei Zielsprachen gleichzeitig oder den Originaltext unter der Übersetzung in An Fenster anpassen oder Kompakt, sofern die Engine ihn liefert. Eine zweite Zielsprache öffnet für jede Audioquelle eine zusätzliche Cloud-Sitzung und erhöht die Anbieternutzung. Gemini und Mistral bieten auch Cloud-Untertitel in der gesprochenen Sprache.

Passen Sie Schrift, Größe, Farbe, Kontrast, Anzeigedauer und Aktualisierungstempo an. Wählen Sie An Fenster anpassen, Kompakt oder Ruhiger Lesemodus, auch für Text von rechts nach links. Bearbeiten Sie die Wörter des optionalen Füllwortfilters, während das Originaltranskript erhalten bleibt. Nutzen Sie gespeicherte Besprechungsprofile und pausieren Sie, ohne die Sitzung aufzuteilen.

Exportieren Sie Transkripte als Text, Markdown, SRT oder WebVTT. Der optionale lokale Verlauf speichert abgeschlossene Untertitel fortlaufend; Sie können Sitzungen benennen, durchsuchen, erneut öffnen, kopieren, exportieren und löschen. Verlauf und Wiederherstellungskopie sind standardmäßig deaktiviert. Bei Whisper verarbeitet Stopp noch ausstehendes Audio, bevor das Transkript vollständig ist.

Whisper-Audio bleibt auf dem PC; ausstehende Sprache wird in einer automatisch gelöschten temporären Datei gepuffert. Cloud-Audio geht direkt an den gewählten Anbieter. API-Schlüssel liegen in der Windows-Anmeldeinformationsverwaltung. Der Entwickler erhält weder Audio noch Schlüssel, Transkripte oder Telemetrie. Die App verkauft keine Abonnements, Guthaben oder API-Zugänge.

Die Oberfläche ist auf Deutsch, Englisch und Französisch verfügbar, unabhängig von der Untertitelsprache. Eine optionale skriptbasierte Demo auf Englisch und Französisch zeigt die Anzeigefunktionen ohne Einrichtung; sie erkennt keine Sprache.

Erfordert Windows 11 und die Microsoft Edge WebView2-Runtime. Native x64- und ARM64-Pakete sind verfügbar. Laden Sie vor der Offline-Nutzung ein Whisper-Modell herunter und erlauben Sie den Zugriff auf die ausgewählte Audioquelle. Die Aufnahme einer Anwendung umfasst ihre Unterprozesse und Benachrichtigungen und kann mehrere Browser-Tabs einschließen.
```

### Funktionen — eine Funktion pro Zeile

```text
Offline-Untertitel mit Whisper: 99 gesprochene Sprachen oder automatische Erkennung
Tiny, Base oder Small herunterladen und danach ohne Konto, API-Schlüssel oder Internet transkribieren
Cloud-Übersetzung mit eigenem Gemini- oder OpenAI-Schlüssel; nutzungsabhängige Kosten möglich
Zwei Übersetzungssprachen gleichzeitig und optional der verfügbare Originaltext
Mikrofon, Systemaudio, eine Anwendung mit Unterprozessen oder Mikrofon und Systemaudio gemeinsam
Transparente, durchklickbare Untertitel im Vordergrund über Folien und Videoanrufen
An Fenster anpassen, Kompakt und Ruhiger Lesemodus; anpassbare Darstellung und Text von rechts nach links
Pause und Fortsetzen, Besprechungsprofile, Sprachsuche und Favoriten
Bearbeitbarer Füllwortfilter für das Overlay; das Originaltranskript bleibt erhalten
Export als Text, Markdown, SRT und WebVTT, einschließlich verfügbarem Originaltext
Optionaler lokaler Transkriptverlauf mit Titeln, Suche, erneutem Öffnen und Löschen
Oberfläche auf Deutsch, Englisch und Französisch; native Windows-Versionen für x64 und ARM64
```

### Kurzbeschreibung

```text
Untertitel über Meetings, Anrufen und Folien. Mit Whisper offline in 99 Sprachen transkribieren oder per Cloud mit eigenem API-Schlüssel übersetzen. Mikrofon oder App-Audio erfassen, zwei Sprachen anzeigen und Transkripte exportieren.
```

### Neu in dieser Version — 1.6.0

```text
Whisper bringt Offline-Untertitel auf Ihren PC. Laden Sie ein mehrsprachiges Tiny-, Base- oder Small-Modell herunter und wählen Sie aus 99 gesprochenen Sprachen oder die automatische Erkennung. Transkribieren Sie Mikrofon- oder Systemaudio ohne Konto oder API-Schlüssel. Audio bleibt lokal. Geschwindigkeit und Genauigkeit hängen von PC, Modell und Sprache ab; Stopp lässt ausstehendes Audio fertig verarbeiten.

Für zweisprachige Meetings zeigen Sie zwei Übersetzungssprachen gleichzeitig oder ergänzen den verfügbaren Originaltext unter Übersetzungen und in Exporten. Cloud-Übersetzung nutzt Ihren API-Schlüssel; eine zweite Zielsprache erhöht die Anbieternutzung.

Pausieren und fortsetzen, ohne das Transkript aufzuteilen. Cloud-Verbindungen werden in Pausen geschlossen; Whisper verarbeitet bereits wartendes Audio weiter. Bearbeiten Sie die Füllwortliste des Overlays, ohne das Originaltranskript zu ändern.

Neue Installationen starten mit Whisper, Base und automatischer Spracherkennung; gespeicherte Einstellungen bleiben erhalten. Außerdem: effizientere Untertiteldarstellung und Verlaufsspeicherung, verbesserte Gemini-Wiederverbindung und eine einheitlichere Bedienoberfläche. Native x64- und ARM64-Versionen; Oberfläche auf Deutsch, Englisch oder Französisch. Danke an @valentinrabot für den Vorschlag zur lokalen Transkription in Issue #98.
```

## Adding the German listing

The interface supports German. Before publishing a German listing, review the German copy
and capture the final packaged app with its interface set to German. Existing English and
French images are historical too; see the [screenshot plan](store-screenshots/README.md).

## Additional system requirements

Enter each line as a separate requirement (under 200 characters):

```text
Windows 11 with Microsoft Edge WebView2 Runtime; native x64 and ARM64 packages
Whisper: internet for the initial model download (31–182 MiB), then offline CPU transcription
Cloud engines: internet, your own compatible provider account and API key; usage charges may apply
Audio-source access required for speech recognition; a microphone is needed only for microphone capture
```

Recommended memory: 8 GB. Whisper speed depends on the CPU and model; this is not a guarantee
of real-time processing on all supported hardware. Localized requirements:

### Configuration supplémentaire — français

```text
Windows 11 et Microsoft Edge WebView2 Runtime ; paquets x64 et ARM64 natifs
Whisper : Internet pour télécharger le modèle (31–182 Mio), puis transcription locale hors ligne sur le processeur
Moteurs cloud : Internet, votre compte et clé API compatibles ; des frais d'utilisation peuvent s'appliquer
Accès à la source audio pour la reconnaissance ; microphone nécessaire uniquement pour la capture du micro
```

### Zusätzliche Systemanforderungen — deutsch

```text
Windows 11 mit Microsoft Edge WebView2-Runtime; native Pakete für x64 und ARM64
Whisper: Internet für den Modelldownload (31–182 MiB), danach Offline-Transkription auf der CPU
Cloud-Engines: Internet, eigenes kompatibles Konto und API-Schlüssel; nutzungsabhängige Kosten möglich
Zugriff auf die Audioquelle für Spracherkennung; ein Mikrofon ist nur für Mikrofonaufnahme erforderlich
```

## Notes for certification

```text
Product: Live Translation & Subtitles. Product ID: 9PFB8LR3RR9X. Version: 1.6.0.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime.

Real speech recognition without a paid account or API key:
1. A clean install opens on Subtitles / Whisper / Base / Detect automatically, with the default microphone selected. No download or capture starts automatically. Choose the microphone or system audio source. Grant microphone access if using a microphone. Existing saved setups are preserved on upgrade.
2. Choose the spoken language or Detect automatically. Download Tiny (about 31 MiB) for the lightest setup, or Base (about 57 MiB). The initial HTTPS model download needs internet access; the model is not bundled. No account or key is needed.
3. Start captioning and speak a few sentences or play speech on the selected output. Allow for CPU processing time. Verify captions in the operator window and overlay, elapsed time and input levels.
4. Use Move overlay to place it, lock it back to click-through, and try Fit window, Compact and Stable reading. Pause blocks new speech entering recognition; existing pending audio can still finish.
5. Stop and let Finishing the transcript complete. Export text or Markdown, then SRT or WebVTT. Disconnect the network and repeat with the installed model to verify offline operation.
6. In Settings, optionally enable transcript history; run a disposable session and reopen, rename, export and delete it. History and recovery are disabled by default. Test the editable filler-word list: it changes only the overlay.

Setup-free display check if capture hardware or model downloads are unavailable:
In Subtitles, select Built-in demo in the engine list, choose English and click Start demo subtitles. The bundled English/French script exercises status, meter animation, timer, overlay, Stop and export without devices, network or credentials. Repeat with French. It is explicitly a scripted demo and does not recognize speech; Whisper above is the first-launch local speech engine.

Optional cloud features:
Gemini/OpenAI translation and Gemini/Mistral subtitles require the user's compatible third-party account and API key and may incur provider charges. The app supplies no cloud account or credits. Keys are entered in the app and kept in Windows Credential Manager. Cloud testing requires suitable test credentials to be arranged separately before submission. Never place a personal key in public listing fields or screenshots. Choose two supported translation targets to check bilingual output; this creates a second provider session per audio source. Original speech is shown only when the provider supplies it.

The full-trust desktop process performs WASAPI/microphone capture, CPU Whisper inference, user-requested verified model downloads, Credential Manager access, window/overlay management and explicit transcript export. Whisper buffers pending audio in an automatically deleted local temporary file. Cloud engines send audio directly to the selected provider. There is no developer relay, telemetry, driver, service, auto-start task, Windows AI component or required speech language pack.
```

Resolve cloud test credentials or Microsoft reviewer access requirements before submission;
the credential-free Whisper and demo routes do not establish acceptance of cloud features.

## Screenshots

Capture real, non-private sessions from the final MSIX, following the
[screenshot plan](store-screenshots/README.md). These captions describe planned images;
do not upload them against old screenshots. Each caption is under 200 characters.

| Image | English | Français | Deutsch |
| --- | --- | --- | --- |
| 1 — Whisper | Offline captions with Whisper: choose a model, spoken language and audio source. | Sous-titres hors ligne avec Whisper : choisissez un modèle, une langue et une source audio. | Offline-Untertitel mit Whisper: Modell, gesprochene Sprache und Audioquelle auswählen. |
| 2 — Overlay | Keep captions visible over your slides or video call in a transparent, click-through overlay. | Gardez les sous-titres visibles sur vos diapositives ou appels, dans une surimpression traversable par les clics. | Untertitel bleiben über Folien oder Videoanrufen sichtbar, in einem transparenten, durchklickbaren Overlay. |
| 3 — Bilingual | Show two translation languages and include the original speech when the engine supplies it. | Affichez deux langues de traduction et le texte original lorsque le moteur le fournit. | Zwei Übersetzungssprachen anzeigen und den Originaltext einblenden, wenn die Engine ihn liefert. |
| 4 — Reading | Adjust caption appearance and reading pace, with presets and your own filler-word list. | Réglez l'apparence et le rythme de lecture, avec des préréglages et votre liste d'hésitations. | Darstellung und Lesetempo anpassen, mit Voreinstellungen und einer eigenen Füllwortliste. |
| 5 — History | Find a saved session, read its transcript and export text or timed subtitles. | Retrouvez une session, relisez sa transcription et exportez du texte ou des sous-titres minutés. | Gespeicherte Sitzungen finden, Transkripte lesen und Text oder zeitbasierte Untertitel exportieren. |
| 6 — Optional demo | Try the display controls with a scripted English/French demo, without audio capture or setup. | Essayez l'affichage avec une démo scénarisée en français ou en anglais, sans capture audio ni configuration. | Anzeige mit einer englischen oder französischen Skript-Demo ausprobieren, ohne Audioaufnahme oder Einrichtung. |
