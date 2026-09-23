# Partner Center listing text — 1.5.1 prepared

Prepared for **1.5.1** (MSIX **1.5.1.0**), not submitted to the Store. This supersedes the
unsubmitted 1.5.0 listing. Only the "What's new" blocks changed. 1.5.1 adds no features, so
the descriptions, short descriptions, features and certification notes are unchanged from
1.5.0. Complete the live-provider and final-package checks in
[the 1.5.1 handoff](store-updates.md#release-151-handoff) before submission.

The German listing is new. Native-language review and final-package screenshots remain
pending; see [Adding the German listing](#adding-the-german-listing). Paste only the relevant
field blocks. See the [release handoff](store-updates.md#release-151-handoff) for remaining checks.

## English (United States)

### Description — full text

Paste this into **Description**:

```text
Live microphone or system-audio captions and translation require your own compatible API key and account from Google Gemini, OpenAI, or Mistral. These third-party services may charge your account based on usage.

Live Translation & Subtitles is a Windows presentation tool for meetings, lectures, and conferences. It displays captions in a dedicated operator window and in a transparent, always-on-top overlay that can be positioned over slides, video calls, or other content.

A built-in English and French demonstration works without an account, API key, microphone, language pack, internet connection, or payment. It uses bundled scripted captions to demonstrate the overlay, timer, audio-level display, transcript, and export workflow. It does not recognize live speech.

For real-time use, select Gemini or OpenAI for translation with engine-dependent language coverage, or Gemini or Mistral for same-language subtitles. Both subtitle engines detect the spoken language themselves, and Gemini covers over 70 languages. Live providers can capture a selected room microphone, Windows system audio, or both. System capture can use all audio on an output device or one selected application and its child processes. Application notifications are included, and browsers may include multiple tabs. Save transcripts to a folder of your choice as plain text, Markdown, SRT, or VTT.

For long meetings, choose Stable reading: left-aligned text stays at the top and advances by whole lines. Optional Hide filler words cleans obvious hesitation sounds only in the overlay; raw transcripts and exports remain unchanged. Fit window and Compact are also available.

Optional transcript history progressively saves finalized captions locally, with session date, duration and known language information. Reopen sessions to read, copy, export or delete them. History is off by default. Turning it off keeps existing sessions until you delete them. No audio is saved.

Save named meeting profiles to reuse setup, caption appearance and overlay position on this PC. Adjust completed-caption persistence, choose steadier interim updates, and preview reading presets. Name saved sessions and search their text, dates and caption language. Per-source input status and keyboard shortcuts help operate live sessions. Profiles exclude API keys; check devices and placement before starting. Gemini subtitles use Smart transcription: saved text reflects provider cleanup, while the optional local filler filter only changes the overlay.

Use the interface in English, French or German, independently of the caption language. Caption coverage depends on the engine: 78 translation targets with Gemini, 13 with OpenAI. Search names or codes and pin favourites. The built-in demo stays English/French.

The app does not sell subscriptions, credits, or API access. Provider keys are stored in Windows Credential Manager and sent only to the selected provider. The developer operates no server and receives no audio, keys, transcripts, analytics, or telemetry.

Requires Windows 11 and Microsoft Edge WebView2 Runtime. Live modes also require internet access, a compatible provider account and API key, and permission for the selected audio source. Native x64 and ARM64 packages are available.
```

### Features — one feature per line

Paste this into **Features**. Each line is one bullet; do not paste these lines into the full
description:

```text
Built-in English and French caption demonstration with no account, API key, microphone, or network
Live translation with engine-dependent language coverage with your own Google Gemini or OpenAI API key
Live same-language subtitles with your own Mistral or Google Gemini API key
Capture a selected microphone, system audio from an output device or one application, or both in live modes
Transparent, always-on-top caption overlay that remains click-through
Move, resize, show, or hide the overlay during a session
Fit window, Compact, or Stable reading with direction-aware captions and whole-line scrolling
Optional filler cleanup in the overlay while preserving raw transcripts
Optional local session history with progressive saving, reopening, copying, export and deletion
Save transcripts as text, Markdown, SRT, or VTT using the Windows Save As dialog
See elapsed time and estimated provider cost while live audio is streaming
Local meeting profiles without API keys, with device checks on load
Caption persistence, steadier updates, appearance previews and reading presets
Session titles and text, date and caption-language search
Per-source live status and operator-window keyboard shortcuts
```

### Short description — 172 characters

Paste this into **Short description**:

```text
Built-in caption demo, plus live subtitles and multilingual translation from microphone or system audio with your own provider key. Language coverage depends on the engine.
```

### What's new in this version — 1.5.1

Paste after final packaged-app verification:

```text
Version 1.5.1 keeps more speech detail in the audio sent to caption engines, keeps the app and its transcript open if one audio source fails, and saves history more lightly during long meetings. When a closing prompt opens over Settings, the keyboard now reaches its buttons.

Find caption languages by name or code and pin favourites. Coverage depends on the engine: Gemini offers 78 translation targets, OpenAI 13. Unsupported choices stay visible with a reason and block Start. F2 swaps the first two supported favourites before starting. Demo scripts remain English/French; subtitle engines auto-detect.

Arabic, Hebrew, Persian and Urdu captions support right-to-left reading. Language rows and stars are vertically centered. The English/French/German interface remains independent of caption language.

Also includes local meeting profiles, optional transcript history with search and export, reading presets, steadier updates, and persistent Start/Stop controls. No audio is saved. Thanks to @valentinrabot for the earlier meeting feedback.
```

## Français (France)

### Description — texte complet

Collez ce texte dans **Description** :

```text
Les sous-titres et la traduction en direct depuis un microphone ou l’audio système exigent votre propre clé API et un compte compatibles chez Google Gemini, OpenAI ou Mistral. Ces services tiers peuvent facturer votre compte selon l’utilisation.

Live Translation & Subtitles est un outil de présentation Windows destiné aux réunions, cours et conférences. Il affiche les sous-titres dans une fenêtre de contrôle et dans une surimpression transparente et toujours visible, qui peut être placée au-dessus de diapositives, d’un appel vidéo ou d’un autre contenu.

Une démonstration intégrée en anglais et en français fonctionne sans compte, clé API, microphone, module linguistique, connexion Internet ni paiement. Elle utilise des sous-titres scénarisés fournis avec l’application pour montrer la surimpression, le chronomètre, l’indicateur de niveau, la transcription et l’export. Elle ne reconnaît pas la parole en direct.

Pour une utilisation en temps réel, sélectionnez Gemini ou OpenAI pour la traduction avec une couverture linguistique propre à chaque moteur, ou Gemini ou Mistral pour les sous-titres dans la langue parlée. Les deux moteurs de sous-titrage détectent eux-mêmes la langue parlée, et Gemini prend en charge plus de 70 langues. Les fournisseurs en direct peuvent capter un microphone sélectionné, l’audio système Windows ou les deux. La capture système peut utiliser tout l’audio d’une sortie ou une application sélectionnée et ses processus enfants. Les notifications de cette application sont incluses, et un navigateur peut inclure plusieurs onglets. Enregistrez les transcriptions dans le dossier de votre choix en texte brut, Markdown, SRT ou VTT.

Pour les longues réunions, choisissez Lecture stable : le texte est aligné à gauche, reste en haut et défile par lignes entières. L’option Masquer les hésitations nettoie uniquement la surimpression ; les transcriptions originales et les exports restent inchangés. Adapter à la fenêtre et Compact restent disponibles.

L’historique facultatif enregistre progressivement les sous-titres finalisés en local, avec la date, la durée et les langues connues de chaque session. Rouvrez une session pour la lire, la copier, l’exporter ou la supprimer. L’historique est désactivé par défaut. Sa désactivation conserve les sessions existantes jusqu’à leur suppression. Aucun audio n’est enregistré.

Enregistrez des profils de réunion pour retrouver la configuration, l’apparence et la position de la surimpression sur ce PC. Réglez la durée des sous-titres terminés, stabilisez les mises à jour partielles et prévisualisez les préréglages. Nommez les sessions enregistrées et recherchez leur texte, leur date et leur langue. Le statut de chaque source et les raccourcis facilitent le pilotage en direct. Les profils excluent les clés API ; vérifiez les périphériques et la position avant de démarrer. Les sous-titres Gemini utilisent la transcription Smart : le texte enregistré reflète le nettoyage du fournisseur ; le filtre local ne modifie que la surimpression.

L’interface est disponible en anglais, français et allemand, indépendamment de la langue des sous-titres. La couverture dépend du moteur : 78 langues cibles avec Gemini, 13 avec OpenAI. Recherchez un nom ou un code et épinglez vos favoris. La démo reste en anglais et en français.

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
Adapter à la fenêtre, Compact ou Lecture stable avec alignement à gauche et défilement par lignes
Masquage facultatif des hésitations dans la surimpression, avec conservation des transcriptions originales
Historique local facultatif avec enregistrement progressif, lecture, copie, export et suppression
Export en texte, Markdown, SRT ou VTT avec la fenêtre Enregistrer sous de Windows
Affichage du temps écoulé et du coût estimé pendant la transmission audio en direct
Profils locaux sans clés API, avec vérification des périphériques au chargement
Durée des sous-titres, mises à jour plus stables, aperçus et préréglages de lecture
Titres de session et recherche par texte, date et langue des sous-titres
Statut de chaque source et raccourcis dans la fenêtre de contrôle
```

### Description courte — 148 caractères

Collez ce texte dans **Description courte** :

```text
Démo intégrée, sous-titres et traduction multilingue du micro ou de l’audio système avec votre clé API. Les langues disponibles dépendent du moteur.
```

### Nouveautés de cette version — 1.5.1

Paste after final packaged-app verification:

```text
La version 1.5.1 conserve davantage de détails de la parole dans l’audio envoyé aux moteurs de sous-titrage, garde l’application et sa transcription ouvertes si une source audio échoue et allège l’enregistrement de l’historique pendant les longues réunions. Quand une demande de fermeture s’ouvre par-dessus les Paramètres, le clavier atteint désormais ses boutons.

Recherchez une langue par nom ou code et épinglez vos favoris. La couverture dépend du moteur : 78 langues cibles avec Gemini, 13 avec OpenAI. Un choix incompatible reste visible avec une explication et bloque le démarrage. F2 permute les deux premiers favoris compatibles avant de démarrer. La démo reste anglais/français ; les moteurs de sous-titrage détectent la langue.

Les sous-titres arabes, hébreux, persans et ourdous se lisent de droite à gauche. Les langues et étoiles sont centrées verticalement. L’interface anglais/français/allemand reste indépendante.

Inclut aussi profils locaux, historique facultatif avec recherche et export, préréglages de lecture, mises à jour plus stables et commandes Démarrer/Arrêter persistantes. Aucun audio n’est enregistré. Merci à @valentinrabot pour ses retours sur les réunions.
```

## Deutsch (Deutschland)

### Beschreibung — vollständiger Text

Fügen Sie diesen Text in **Beschreibung** ein:

```text
Untertitel und Übersetzung in Echtzeit aus Mikrofon- oder Systemaudio erfordern einen eigenen kompatiblen API-Schlüssel und ein Konto bei Google Gemini, OpenAI oder Mistral. Diese Drittanbieterdienste können Ihr Konto nutzungsabhängig abrechnen.

Live Translation & Subtitles ist ein Windows-Präsentationswerkzeug für Besprechungen, Lehrveranstaltungen und Konferenzen. Es zeigt Untertitel in einem eigenen Bedienfenster und in einem transparenten Overlay, das immer im Vordergrund bleibt und über Folien, Videoanrufe oder andere Inhalte gelegt werden kann.

Eine integrierte Demonstration auf Englisch und Französisch funktioniert ohne Konto, API-Schlüssel, Mikrofon, Sprachpaket, Internetverbindung oder Bezahlung. Sie verwendet mitgelieferte, skriptbasierte Untertitel, um Overlay, Zeitmessung, Pegelanzeige, Transkript und Export vorzuführen. Sie erkennt keine live gesprochene Sprache.

Für den Einsatz in Echtzeit wählen Sie Gemini oder OpenAI für die Übersetzung mit engineabhängiger Sprachauswahl, oder Gemini oder Mistral für Untertitel in der gesprochenen Sprache. Beide Untertitel-Engines erkennen die gesprochene Sprache selbst, und Gemini deckt über 70 Sprachen ab. Die Live-Anbieter können ein ausgewähltes Raummikrofon, das Windows-Systemaudio oder beides aufnehmen. Die Systemaufnahme kann das gesamte Audio eines Ausgabegeräts verwenden oder eine ausgewählte Anwendung samt ihrer Unterprozesse. Benachrichtigungen dieser Anwendung sind enthalten, und ein Browser kann mehrere Tabs umfassen. Speichern Sie Transkripte in einem Ordner Ihrer Wahl als reinen Text, Markdown, SRT oder VTT.

Für längere Besprechungen wählen Sie Ruhiger Lesemodus: Der Text bleibt linksbündig am oberen Rand und rückt in ganzen Zeilen weiter. Fülllaute ausblenden entfernt optional eindeutige Zögerlaute nur im Overlay; Originaltranskripte und Exporte bleiben unverändert. An Fenster anpassen und Kompakt bleiben verfügbar.

Der optionale Transkriptverlauf speichert abgeschlossene Untertitel fortlaufend lokal, mit Datum, Dauer und bekannten Sprachangaben. Öffnen Sie frühere Sitzungen zum Lesen, Kopieren, Exportieren oder Löschen. Der Verlauf ist standardmäßig deaktiviert. Beim Deaktivieren bleiben vorhandene Sitzungen bis zum Löschen erhalten. Audio wird nicht gespeichert.

Speichern Sie Besprechungsprofile mit Einstellungen, Untertiteldarstellung und Overlay-Position auf diesem PC. Passen Sie die Anzeigedauer fertiger Untertitel an, bündeln Sie vorläufige Aktualisierungen und prüfen Sie Lesevoreinstellungen in der Vorschau. Benennen Sie gespeicherte Sitzungen und durchsuchen Sie Text, Datum und Untertitelsprache. Eingangsstatus je Quelle und Tastenkombinationen erleichtern die Bedienung. Profile enthalten keine API-Schlüssel; prüfen Sie Geräte und Position vor dem Start. Gemini-Untertitel verwenden Smart-Transkription: Gespeicherter Text enthält die Bereinigung des Anbieters, der optionale lokale Filter ändert nur das Overlay.

Die Oberfläche steht auf Deutsch, Englisch und Französisch zur Verfügung und wird unabhängig von der Sprache der Untertitel gewählt: Sie arbeiten auf Deutsch, während der Raum in einer anderen Sprache mitliest.

Die App verkauft weder Abonnements noch Guthaben oder API-Zugang. Anbieterschlüssel werden in der Windows-Anmeldeinformationsverwaltung gespeichert und ausschließlich an den ausgewählten Anbieter gesendet. Der Entwickler betreibt keinen Server und erhält weder Audio noch Schlüssel, Transkripte, Analyse- oder Telemetriedaten.

Erfordert Windows 11 und die Microsoft Edge WebView2-Runtime. Live-Modi erfordern zusätzlich einen Internetzugang, ein kompatibles Anbieterkonto mit API-Schlüssel sowie die Berechtigung für die ausgewählte Audioquelle. Native x64- und ARM64-Pakete stehen zur Verfügung.
```

### Funktionen — eine Funktion pro Zeile

Fügen Sie diesen Block in **Funktionen** ein. Jede Zeile wird zu einem Aufzählungspunkt; fügen
Sie ihn nicht in die vollständige Beschreibung ein:

```text
Integrierte Untertitel-Demonstration auf Englisch und Französisch, ohne Konto, API-Schlüssel, Mikrofon oder Netzwerk
Live-Übersetzung mit engineabhängiger Sprachauswahl mit Ihrem eigenen Google-Gemini- oder OpenAI-API-Schlüssel
Live-Untertitel in der gesprochenen Sprache mit Ihrem eigenen Mistral- oder Google-Gemini-API-Schlüssel
Aufnahme eines ausgewählten Mikrofons, des Systemaudios eines Ausgabegeräts oder einer Anwendung, oder beider in Live-Modi
Transparentes Untertitel-Overlay, immer im Vordergrund und durchklickbar
Overlay während einer Sitzung verschieben, in der Größe ändern, ein- oder ausblenden
An Fenster anpassen, Kompakt oder Ruhiger Lesemodus mit linksbündigen Untertiteln und ganzen Zeilen
Optionale Bereinigung von Fülllauten im Overlay bei unveränderten Originaltranskripten
Optionaler lokaler Verlauf mit fortlaufender Speicherung, Lesen, Kopieren, Export und Löschen
Transkripte als Text, Markdown, SRT oder VTT über den Windows-Dialog Speichern unter sichern
Verstrichene Zeit und geschätzte Anbieterkosten während der Live-Audioübertragung ablesen
Lokale Besprechungsprofile ohne API-Schlüssel, mit Geräteprüfung beim Laden
Anzeigedauer, ruhigere Aktualisierungen, Vorschau und Lesevoreinstellungen
Sitzungstitel und Suche nach Text, Datum und Untertitelsprache
Live-Status je Quelle und Tastenkombinationen im Bedienfenster
Oberfläche auf Deutsch, Englisch und Französisch, unabhängig von der Sprache der Untertitel
```

The German list carries one bullet the English and French ones do not — the interface
languages. It is the single reason a German-speaking visitor is looking at a German listing at
all, and burying it in a paragraph wastes it. Partner Center allows up to 20 entries, so the
additional line fits within that limit.

### Kurzbeschreibung — 164 Zeichen

Fügen Sie diesen Text in **Kurzbeschreibung** ein:

```text
Integrierte Demo, Live-Untertitel und mehrsprachige Übersetzung aus Mikrofon- oder Systemaudio mit Ihrem API-Schlüssel. Die Sprachabdeckung hängt von der Engine ab.
```

All three short descriptions are below the documented 270-character Partner Center cap.
The counts above were refreshed for 1.5.0 and are unchanged in 1.5.1. The 1.5.1 "What's new" blocks measure 1,047 (EN), 1,193 (FR) and 1,215 (DE) characters, within the 1,500-character limit.

### Neuerungen in dieser Version — 1.5.1

Paste after final packaged-app verification:

```text
Version 1.5.1 bewahrt mehr Sprachdetails im Audio, das an die Untertitel-Engines gesendet wird, hält App und Transkript offen, wenn eine Audioquelle ausfällt, und speichert den Verlauf in langen Besprechungen sparsamer. Öffnet sich eine Schließen-Abfrage über den Einstellungen, erreicht die Tastatur jetzt ihre Schaltflächen.

Untertitelsprachen nach Name oder Code suchen und als Favoriten anheften. Die Abdeckung hängt von der Engine ab: 78 Übersetzungsziele mit Gemini, 13 mit OpenAI. Nicht unterstützte Ziele bleiben mit Erklärung sichtbar und verhindern den Start. F2 tauscht vor dem Start die ersten beiden unterstützten Favoriten. Die Demo bleibt Englisch/Französisch; Untertitel-Engines erkennen die Sprache automatisch.

Arabisch, Hebräisch, Persisch und Urdu unterstützen die Leserichtung von rechts nach links. Sprachzeilen und Sterne sind vertikal zentriert. Die deutsche, englische oder französische Oberfläche bleibt unabhängig.

Enthalten sind auch lokale Besprechungsprofile, optionaler Transkriptverlauf mit Suche und Export, Lesevoreinstellungen, ruhigere Aktualisierungen und ständig erreichbare Start/Stopp-Tasten. Audio wird nicht gespeichert. Danke an @valentinrabot für das frühere Feedback.
```

The interface language and caption language remain separate. This 1.5.1 listing includes
German among the translation targets. Publish that claim only with a build that contains #78
and after its live acceptance checks pass.

### Adding the German listing

Before the German listing is published, all of these have to be true. None of them is a
formality:

- [ ] The release that ships `src/lib/i18n/de.ts` is built, packaged and verified — the
      listing cannot be live before the interface it describes.
- [ ] A native German speaker has reviewed the catalog and this copy. This is the same gate
      French passed and the one that catches what a translation check cannot.
- [ ] German screenshots exist in `docs/store-screenshots/de/`, captured from the final MSIX
      with the interface set to German — five, same filenames and order as the other languages.
- [x] The English and French **What's new** blocks for that same release are written. Adding a
      language does not rewrite them, but a release still needs all three.
- [x] The certification notes name the German interface, so the reviewer exercises it.

Worth doing, and deliberately not claimed here until it is: **German subtitles are unverified.**
Both subtitle engines detect the spoken language themselves and Gemini documents over 70
languages, so German subtitles most likely already work with no code at all — but nobody has
put German speech in front of them. Until somebody does, the German listing claims exactly what
the English one does and singles out no language. Verifying it would let this listing say
"Untertitel in gesprochenem Deutsch", which for a German-language Store listing is the strongest
line available and currently unusable.

### Deutsche Bildunterschriften

Same five screenshots, same order. Each is under the 200-character cap; re-count after an edit.

1. `1-idle.png`

```text
Vier Prüfungen, bevor irgendetwas startet: der Anbieterschlüssel, das Mikrofon, wohin die Untertitel fallen und was die Stunde kostet. Alle vier werden gesperrt, sobald die Sitzung läuft.
```

2. `2-running.png`

```text
Eine laufende Sitzung zeigt Pegel, verstrichene Zeit und laufende Kosten. Die Untertitelsprache ist von der Oberfläche getrennt: Der Raum liest eine Sprache, Sie arbeiten in einer anderen.
```

3. `3-overlay.png`

```text
Das Overlay wird platziert, bevor der Raum sich füllt: dorthin gezogen, wo das Publikum liest, am Projektor bemessen und verankert, damit mitten im Vortrag nichts verrutscht.
```

4. `4-appearance.png`

```text
Wählen Sie An Fenster anpassen, Ruhiger Lesemodus oder Kompakt. Stellen Sie Schrift, Farbe und Hintergrund ein und blenden Sie bei Bedarf Fülllaute aus.
```

5. `5-contrast.png`

```text
Jede Farbwahl wird gegen eine helle und eine dunkle Folie gemessen, und die App benennt die Zeile, die darunter bleibt, statt Sie es aus der letzten Reihe herausfinden zu lassen.
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
4. Within one second, the status changes to Demo, the clock in the status pill advances, and the Demo audio meter moves.
5. Partial and final English captions appear automatically in the operator window and presentation overlay.
6. Click Stop captions. The completed transcript remains available for export.
7. Choose each transcript format (text, Markdown, SRT, VTT), click Save as, select a writable folder and verify the saved file. Cancel another save and confirm that the transcript remains available.
8. Select Français and repeat to verify the French captions.
9. In Settings → App, switch the interface language between English, Français and Deutsch and confirm the operator window and overlay follow it. The built-in demo caption languages are English and French; the interface language is chosen separately and does not change them.

Retained from 1.3.0: enable Automatically save sessions locally in Transcript history, start and stop two demo sessions, restart the app and browse both sessions. Verify date, duration and known language information, raw caption/source text, Copy transcript and all four exports. Delete one session with confirmation; the other must remain. Turning history off must stop future saves while retaining existing sessions. History is local, unencrypted, opt-in, and never uploads audio or transcripts. It is independent of the single recovery copy.

In 1.4.2, open Settings and use the Captions, Reading, Transcript history and App tabs. Presets and bright/dark previews are always visible in Captions. Changes apply immediately; close with the top-right button or Escape. Manage profiles above step 01; save, rename, load and confirm deletion. History has a session list and detail pane with filters and Clear filters. Check EN/FR/DE tray labels and live status.

Switch all four tabs and verify the dialog and tab bar remain stationary while the content
scrolls. In History, test both date bounds and Clear filters in English, French and German.
Hints are YYYY-MM-DD, AAAA-MM-JJ and JJJJ-MM-TT; entered values remain year-month-day.
Check valid leap days, invalid-date messages and native calendar selection in the packaged app.

Introduced in 1.4.0 and included in 1.4.2: while stopped, save a Meeting profile, change appearance, then load it. Verify setup and appearance return without starting capture or changing history/recovery preferences. Restart to check profile persistence. Profiles exclude API keys and application process IDs; missing devices require checking the default fallback, and application capture requires reselection.

In Settings → Captions, try Standard, Large room and High contrast. Fit window/Compact allow a 2–30 second hold; Stable reading retains text until it rolls out or the session stops. Choose Steadier updates and verify completed demo captions still appear promptly. Live interim timing and Gemini cleanup need a separate provider test; the deterministic demo cannot validate cloud recognition.

Name a saved demo session, search title and transcript text, filter inclusive dates and caption language, and test a long title at enlarged text size. Clearing the title restores its date label. A title change must preserve the recorded lines. Test Ctrl+Shift+Space, Ctrl+Shift+O and Ctrl+Shift+Up/Down in the operator window, then verify they do not intercept typing in inputs or dialogs. Inspect per-source status and keyboard focus. No custom filter list is included.

During a demo, choose Stable reading in Settings. Text should stay left-aligned at the top, wrap naturally and advance by complete lines as the area fills. A pause should retain it, and Stop should clear it. Resize, then return to Fit window and check normal expiry resumes. Toggle Hide filler words; it affects only the overlay, not the raw transcript/history/export. The scripted demo does not guarantee filler sounds, so the absence of a visible text change there is expected. The filter handles exact hesitation tokens such as um, uh, erm, hmm, euh and heu; it is not semantic speech analysis.

Also test Fit window and Compact during the demo: open Settings → Reading, verify Caption layout is Fit window, enter placement mode, resize the overlay and lock it again to resume the caption view. A wider window fits more words per line; a taller one can retain more available recent context. The font size remains unchanged. Switch to Compact, verify Line width becomes available, then reset the appearance and confirm Fit window returns. Repeat with the French interface (Adapter à la fenêtre / Compact) and the German one (An Fenster anpassen / Kompakt). Placement mode intentionally shows positioning controls rather than live captions.

The built-in demo uses bundled scripted content. It does not open an audio device, invoke Windows speech recognition, contact a server, or use a publisher API key. It deterministically exercises the session state, timing, level meter, partial and final captions, overlay, transcript, Stop action, and export workflow on both x64 and ARM64.

Live microphone and system-audio recognition are optional provider-dependent modes. Mistral and Gemini provide live same-language subtitles. Gemini and OpenAI provide live translation with engine-dependent language coverage. These modes use an API key supplied by the user. No live-provider credential is needed to test the complete default built-in workflow.

Application audio capture is optional. With an open application playing audio, select System audio, One application, and the application, then run the level-only audio test without a provider key. Live speech recognition still requires the user’s compatible provider key. Closing the selected application must not switch capture to all system audio. Application capture includes child processes, not an individually selected browser tab.

The developer operates no backend, relay, telemetry, analytics, or crash-reporting service.
```

## Screenshots

The existing PNGs are the 1.2.2 set. These descriptions are prepared for fresh 1.4.2
captures; do not upload the old appearance screenshots with the new descriptions.

The listing is per-language and so are its screenshots. Every set is kept in the repository and
uploaded in the order the filenames give — `docs/store-screenshots/en/`,
`docs/store-screenshots/fr/` and, once the German interface ships,
`docs/store-screenshots/de/`.

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
Choose Fit window, Stable reading or Compact. Adjust caption size, typeface, colour and backing, and optionally hide filler words.
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
Choisissez Adapter à la fenêtre, Lecture stable ou Compact. Réglez taille, police, couleur et fond, et masquez les hésitations si souhaité.
```

5. `5-contrast.png`

```text
Chaque couleur est mesurée sur une diapositive claire et sur une sombre, et l’application nomme la ligne qui descend sous le seuil au lieu de vous le laisser découvrir du fond de la salle.
```

Capture rules, and when to re-capture, are in
[`store-updates.md`](store-updates.md#screenshots). Do not upload screenshots from the removed
Windows Speech implementation.
