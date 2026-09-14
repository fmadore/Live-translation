# Partner Center listing text — 1.2.4

Prepared for **1.2.4** (MSIX **1.2.4.0**); not yet submitted. Paste only the relevant
field blocks into Partner Center after final package testing. See the
[release handoff](store-updates.md#release-124-handoff) for verification and screenshot status.
The current local test executable is unpackaged and reports 1.2.4; final MSIX verification is pending.

## English (United States)

### Description — full text

Paste this into **Description**:

```text
Live microphone or system-audio captions and translation require your own compatible API key and account from Google Gemini, OpenAI, or Mistral. These third-party services may charge your account based on usage.

Live Translation & Subtitles is a Windows presentation tool for meetings, lectures, and conferences. It displays captions in a dedicated operator window and in a transparent, always-on-top overlay that can be positioned over slides, video calls, or other content.

A built-in English and French demonstration works without an account, API key, microphone, language pack, internet connection, or payment. It uses bundled scripted captions to demonstrate the overlay, timer, audio-level display, transcript, and export workflow. It does not recognize live speech.

For real-time use, select Gemini or OpenAI for English–French translation, or Gemini or Mistral for same-language subtitles. Both subtitle engines detect the spoken language themselves, and Gemini covers over 70 languages. Live providers can capture a selected room microphone, Windows system audio, or both. System capture can use all audio on an output device or one selected application and its child processes. Application notifications are included, and browsers may include multiple tabs. Save transcripts to a folder of your choice as plain text, Markdown, SRT, or VTT.

Make the overlay wider to fit more words per line, or taller to show more recent caption context. Fit window adapts the text area while keeping your chosen font size. Choose Compact to control line width separately.

The app does not sell subscriptions, credits, or API access. Provider keys are stored in Windows Credential Manager and sent only to the selected provider. The developer operates no server and receives no audio, keys, transcripts, analytics, or telemetry.

Requires Windows 11 and Microsoft Edge WebView2 Runtime. Live modes also require internet access, a compatible provider account and API key, and permission for the selected audio source. Native x64 and ARM64 packages are available.
```

### Features — one feature per line

Paste this into **Features**. Each line is one bullet; do not paste these lines into the full
description:

```text
Built-in English and French caption demonstration with no account, API key, microphone, or network
Live English–French translation with your own Google Gemini or OpenAI API key
Live same-language subtitles with your own Mistral or Google Gemini API key
Capture a selected microphone, system audio from an output device or one application, or both in live modes
Transparent, always-on-top caption overlay that remains click-through
Move, resize, show, or hide the overlay during a session
Fit captions to the window or choose Compact with adjustable line width
Save transcripts as text, Markdown, SRT, or VTT using the Windows Save As dialog
See elapsed time and estimated provider cost while live audio is streaming
```

### Short description — 175 characters

Paste this into **Short description**:

```text
Built-in caption demo with no setup, plus optional live microphone and system-audio subtitles and English–French translation using your own Gemini, OpenAI, or Mistral API key.
```

### What's new in this version — 1.2.4

Paste this into **What's new in this version** after packaged testing:

```text
Resize the caption overlay to suit your meeting. Fit window uses the available width and shows more recent caption context when you make the window taller. Your chosen font size stays the same. Align to bottom resets the overlay to a shallow caption strip. Switch to Compact for adjustable line width. Both microphone and system-audio captions share the available space.

Thanks to @valentinrabot for the feedback and responsive-layout suggestion in GitHub issue #77.
```

## Français (France)

### Description — texte complet

Collez ce texte dans **Description** :

```text
Les sous-titres et la traduction en direct depuis un microphone ou l’audio système exigent votre propre clé API et un compte compatibles chez Google Gemini, OpenAI ou Mistral. Ces services tiers peuvent facturer votre compte selon l’utilisation.

Live Translation & Subtitles est un outil de présentation Windows destiné aux réunions, cours et conférences. Il affiche les sous-titres dans une fenêtre de contrôle et dans une surimpression transparente et toujours visible, qui peut être placée au-dessus de diapositives, d’un appel vidéo ou d’un autre contenu.

Une démonstration intégrée en anglais et en français fonctionne sans compte, clé API, microphone, module linguistique, connexion Internet ni paiement. Elle utilise des sous-titres scénarisés fournis avec l’application pour montrer la surimpression, le chronomètre, l’indicateur de niveau, la transcription et l’export. Elle ne reconnaît pas la parole en direct.

Pour une utilisation en temps réel, sélectionnez Gemini ou OpenAI pour la traduction français–anglais, ou Gemini ou Mistral pour les sous-titres dans la langue parlée. Les deux moteurs de sous-titrage détectent eux-mêmes la langue parlée, et Gemini prend en charge plus de 70 langues. Les fournisseurs en direct peuvent capter un microphone sélectionné, l’audio système Windows ou les deux. La capture système peut utiliser tout l’audio d’une sortie ou une application sélectionnée et ses processus enfants. Les notifications de cette application sont incluses, et un navigateur peut inclure plusieurs onglets. Enregistrez les transcriptions dans le dossier de votre choix en texte brut, Markdown, SRT ou VTT.

Élargissez la surimpression pour afficher davantage de mots par ligne, ou agrandissez-la en hauteur pour conserver plus de contexte récent. Adapter à la fenêtre ajuste la zone de texte sans changer la taille des caractères choisie. Le mode Compact permet de régler séparément la largeur des lignes.

L’application ne vend ni abonnement, ni crédits, ni accès API. Les clés des fournisseurs sont enregistrées dans le Gestionnaire d’informations d’identification Windows et transmises uniquement au fournisseur sélectionné. Le développeur n’exploite aucun serveur et ne reçoit ni audio, ni clé, ni transcription, ni donnée analytique ou télémétrique.

Nécessite Windows 11 et Microsoft Edge WebView2 Runtime. Les modes en direct exigent également un accès Internet, un compte et une clé API compatibles, ainsi que l’autorisation d’utiliser la source audio sélectionnée. Des paquets x64 et ARM64 natifs sont disponibles.
```

### Fonctionnalités — une fonctionnalité par ligne

Collez ce bloc dans **Fonctionnalités**. Chaque ligne devient une puce; ne le collez pas dans
la description complète :

```text
Démonstration intégrée en anglais et en français, sans compte, clé API, microphone ni réseau
Traduction français–anglais en direct avec votre propre clé API Google Gemini ou OpenAI
Sous-titres en direct dans la langue parlée avec votre propre clé API Mistral ou Google Gemini
Capture d’un microphone, de l’audio système d’une sortie ou d’une application, ou des deux en direct
Surimpression de sous-titres transparente, toujours visible et transparente aux clics
Déplacement, redimensionnement, affichage ou masquage de la surimpression pendant une session
Sous-titres adaptés à la fenêtre ou mode Compact avec largeur des lignes réglable
Export en texte, Markdown, SRT ou VTT avec la fenêtre Enregistrer sous de Windows
Affichage du temps écoulé et du coût estimé pendant la transmission audio en direct
```

### Description courte — 171 caractères

Collez ce texte dans **Description courte** :

```text
Démo de sous-titres sans configuration, plus sous-titrage micro/audio système et traduction français–anglais en direct avec votre propre clé API Gemini, OpenAI ou Mistral.
```

### Nouveautés de cette version — 1.2.4

À coller après les tests du paquet dans **Nouveautés de cette version** :

```text
Adaptez la surimpression à votre réunion. Adapter à la fenêtre utilise la largeur disponible et affiche davantage de contexte récent lorsque vous agrandissez la fenêtre en hauteur. La taille des caractères choisie reste inchangée. Aligner en bas ramène la surimpression à une bande de sous-titres peu haute. Le mode Compact conserve le réglage de largeur des lignes. Les sous-titres du microphone et de l’audio système se partagent l’espace disponible.

Merci à @valentinrabot pour ses retours et sa suggestion de disposition adaptative dans le ticket GitHub nº 77.
```

## Notes for certification — separate Partner Center field

Paste the following into **Notes for certification**. This is not Store-facing description
text:

```text
Product ID: 9PFB8LR3RR9X

The previous 10.1.2.20 report, “Unusable Feature: Start Subtitles,” was addressed in version 1.0.5 by removing the device-dependent Windows Speech implementation. This update retains that built-in demonstration.

No account, API key, microphone, language pack, or network connection is required for the default test:

1. Install and launch the x64 or ARM64 package.
2. Keep the defaults: Subtitles, Demo audio, English, Built-in demo.
3. Click Start demo subtitles.
4. Within one second, the status changes to Demo, the Elapsed clock advances, and the Demo audio meter moves.
5. Partial and final English captions appear automatically in the operator window and presentation overlay.
6. Click Stop captions. The completed transcript remains available for export.
7. Choose each transcript format (text, Markdown, SRT, VTT), click Save as, select a writable folder and verify the saved file. Cancel another save and confirm that the transcript remains available.
8. Select Français and repeat to verify the French captions.

Also test the new caption layout during the demo: open Settings, verify Caption layout is Fit window, enter placement mode, resize the overlay and lock it again to resume the caption view. A wider window fits more words per line; a taller one can retain more available recent context. The font size remains unchanged. Switch to Compact, verify Line width becomes available, then reset the appearance and confirm Fit window returns. Repeat with the French interface (Adapter à la fenêtre / Compact). Placement mode intentionally shows positioning controls rather than live captions.

The built-in demo uses bundled scripted content. It does not open an audio device, invoke Windows speech recognition, contact a server, or use a publisher API key. It deterministically exercises the session state, timing, level meter, partial and final captions, overlay, transcript, Stop action, and export workflow on both x64 and ARM64.

Live microphone and system-audio recognition are optional provider-dependent modes. Mistral and Gemini provide live same-language subtitles. Gemini and OpenAI provide live English–French translation. These modes use an API key supplied by the user. No live-provider credential is needed to test the complete default built-in workflow.

Application audio capture is optional. With an open application playing audio, select System audio, One application, and the application, then run the level-only audio test without a provider key. Live speech recognition still requires the user’s compatible provider key. Closing the selected application must not switch capture to all system audio. Application capture includes child processes, not an individually selected browser tab.

The developer operates no backend, relay, telemetry, analytics, or crash-reporting service.
```

## Screenshots

The existing PNGs are the 1.2.2 set. These descriptions are prepared for fresh 1.2.4
captures; do not upload the old appearance screenshots with the new descriptions.

The listing is per-language and so are its screenshots. Both sets are kept in the repository
and uploaded after they are refreshed for 1.2.4 — `docs/store-screenshots/en/` and `docs/store-screenshots/fr/`, in
the order their filenames give.

Each screenshot has an optional **Description** field in Partner Center, on the same screen as
the upload. Fill it: it is the alt text a screen reader announces, so an empty one makes the
listing say nothing to the people most likely to need this app. Store descriptions cap at 200
characters — check the count after any edit rather than
assuming there is room.

Captions describe what the picture shows rather than restating the product description — a
visitor is scrolling images, and repeating the paragraph above them tells them nothing new.

### English captions

1. `1-idle.png` — the pre-flight screen: provider key, microphone, overlay placement and the
   hourly cost, with the session idle.

```text
Four checks before anything starts: the provider key, the microphone, where the captions will land, and what the hour will cost. All four lock once the session is running.
```

2. `2-running.png` — running demo showing Demo status, moving meter, elapsed time, and a
   caption.

```text
A running session shows the audio level, elapsed time and running cost. The caption language is set separately from the interface, so the room can read one language while you work in another.
```

3. `3-overlay.png` — the overlay in placement mode, sized and positioned before a session.

```text
The overlay is placed before the room fills: dragged to where the audience will read it, sized against the projector, then locked so nothing shifts mid-talk.
```

4. `4-appearance.png` — refreshed caption appearance showing Fit window, size, typeface,
   caption colour, backing colour and backing strength. Line width is shown only in Compact.

```text
Choose Fit window or Compact, then adjust caption size, typeface, colour and backing. Compact also lets you set line width.
```

5. `5-contrast.png` — the contrast readout naming a step that falls below its target.

```text
Every colour choice is measured against a bright slide and a dark one, and the app names the line that falls short rather than leaving you to find out from the back row.
```

### Légendes françaises

1. `1-idle.png`

```text
Quatre vérifications avant de commencer : la clé du fournisseur, le microphone, l’endroit où tomberont les sous-titres et ce que coûtera l’heure. Tout se verrouille une fois la session lancée.
```

2. `2-running.png`

```text
Une session affiche le niveau audio, le temps écoulé et le coût courant. La langue des sous-titres se règle à part de l’interface : la salle lit une langue pendant que vous travaillez dans une autre.
```

3. `3-overlay.png`

```text
La surimpression se place avant l’arrivée du public : on la fait glisser là où la salle lira, on la dimensionne sur le projecteur, puis on la verrouille pour que rien ne bouge en pleine conférence.
```

4. `4-appearance.png`

```text
Choisissez Adapter à la fenêtre ou Compact, puis réglez taille, police, couleur et fond. Le mode Compact permet aussi de régler la largeur des lignes.
```

5. `5-contrast.png`

```text
Chaque couleur est mesurée sur une diapositive claire et sur une sombre, et l’application nomme la ligne qui descend sous le seuil au lieu de vous le laisser découvrir du fond de la salle.
```

Capture rules, and when to re-capture, are in
[`store-updates.md`](store-updates.md#screenshots). Do not upload screenshots from the removed
Windows Speech implementation.
