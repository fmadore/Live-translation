# Partner Center listing text

Each heading below corresponds to a separate Partner Center field. Paste only the text inside
the relevant code block. Do not paste the headings or explanations.

## English (United States)

### Description — full text

Paste this into **Description**:

```text
Live microphone or system-audio captions and translation require your own compatible API key and account from Google Gemini, OpenAI, or Mistral. These third-party services may charge your account based on usage.

Live Translation & Subtitles is a Windows presentation tool for meetings, lectures, and conferences. It displays captions in a dedicated operator window and in a transparent, always-on-top overlay that can be positioned over slides, video calls, or other content.

A built-in English and French demonstration works without an account, API key, microphone, language pack, internet connection, or payment. It uses bundled scripted captions to demonstrate the overlay, timer, audio-level display, transcript, and export workflow. It does not recognize live speech.

For real-time use, select Gemini or OpenAI for English–French translation, or Gemini or Mistral for same-language subtitles. Both subtitle engines detect the spoken language themselves, and Gemini covers over 70 languages. Live providers can capture a selected room microphone, Windows system audio, or both.

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
Capture a selected microphone, Windows system audio, or both in live modes
Transparent, always-on-top caption overlay that remains click-through
Move, resize, show, or hide the overlay during a session
Save completed transcripts as plain text or Markdown
See elapsed time and estimated provider cost while live audio is streaming
```

### Short description — 175 characters

Paste this into **Short description**:

```text
Built-in caption demo with no setup, plus optional live microphone and system-audio subtitles and English–French translation using your own Gemini, OpenAI, or Mistral API key.
```

## Français (France)

### Description — texte complet

Collez ce texte dans **Description** :

```text
Les sous-titres et la traduction en direct depuis un microphone ou l’audio système exigent votre propre clé API et un compte compatibles chez Google Gemini, OpenAI ou Mistral. Ces services tiers peuvent facturer votre compte selon l’utilisation.

Live Translation & Subtitles est un outil de présentation Windows destiné aux réunions, cours et conférences. Il affiche les sous-titres dans une fenêtre de contrôle et dans une surimpression transparente et toujours visible, qui peut être placée au-dessus de diapositives, d’un appel vidéo ou d’un autre contenu.

Une démonstration intégrée en anglais et en français fonctionne sans compte, clé API, microphone, module linguistique, connexion Internet ni paiement. Elle utilise des sous-titres scénarisés fournis avec l’application pour montrer la surimpression, le chronomètre, l’indicateur de niveau, la transcription et l’export. Elle ne reconnaît pas la parole en direct.

Pour une utilisation en temps réel, sélectionnez Gemini ou OpenAI pour la traduction français–anglais, ou Gemini ou Mistral pour les sous-titres dans la langue parlée. Les deux moteurs de sous-titrage détectent eux-mêmes la langue parlée, et Gemini prend en charge plus de 70 langues. Les fournisseurs en direct peuvent capter un microphone sélectionné, l’audio système Windows ou les deux.

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
Capture d’un microphone sélectionné, de l’audio système Windows ou des deux en mode direct
Surimpression de sous-titres transparente, toujours visible et transparente aux clics
Déplacement, redimensionnement, affichage ou masquage de la surimpression pendant une session
Enregistrement des transcriptions terminées en texte brut ou en Markdown
Affichage du temps écoulé et du coût estimé pendant la transmission audio en direct
```

### Description courte — 171 caractères

Collez ce texte dans **Description courte** :

```text
Démo de sous-titres sans configuration, plus sous-titrage micro/audio système et traduction français–anglais en direct avec votre propre clé API Gemini, OpenAI ou Mistral.
```

## Notes for certification — separate Partner Center field

Paste the following into **Notes for certification**. This is not Store-facing description
text:

```text
Product ID: 9PFB8LR3RR9X

Version 1.0.5 addresses the previous 10.1.2.10 report, “Unusable Feature: Start Subtitles.” The device-dependent Windows Speech implementation has been removed.

No account, API key, microphone, language pack, or network connection is required for the default test:

1. Install and launch the x64 or ARM64 package.
2. Keep the defaults: Subtitles, Demo audio, English, Built-in demo.
3. Click Start demo subtitles.
4. Within one second, the status changes to Demo, the Elapsed clock advances, and the Demo audio meter moves.
5. Partial and final English captions appear automatically in the operator window and presentation overlay.
6. Click Stop captions. The completed transcript remains available for export.
7. Select Français and repeat to verify the French captions.

The built-in demo uses bundled scripted content. It does not open an audio device, invoke Windows speech recognition, contact a server, or use a publisher API key. It deterministically exercises the session state, timing, level meter, partial and final captions, overlay, transcript, Stop action, and export workflow on both x64 and ARM64.

Live microphone and system-audio recognition are optional provider-dependent modes. Mistral and Gemini provide live same-language subtitles. Gemini and OpenAI provide live English–French translation. These modes use an API key supplied by the user. No live-provider credential is needed to test the complete default built-in workflow.

The developer operates no backend, relay, telemetry, analytics, or crash-reporting service.
```

## Screenshot order

1. Idle screen showing Built-in demo, Demo audio, Free, and Start demo subtitles.
2. Running English demo showing Demo status, moving meter, elapsed time, and overlay caption.
3. Running French demo showing a French caption.
4. Optional live-provider configuration showing its key requirement and estimated cost.

Do not upload screenshots from the removed Windows Speech implementation.
