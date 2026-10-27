# Partner Center listing text — 1.12.0

Written for **1.12.0 / MSIX 1.12.0.0**, tagged on 27 October 2026 and not yet submitted. The Store serves 1.11.0
(MSIX 1.11.0.0), accepted as reported by the maintainer on 26 October 2026, so What's new covers
only the changes since 1.11.0. For the next version, replace What's new and the certification
notes' first two paragraphs. Paste each block into its named field after the
[release checks](store-updates.md). These descriptions lead with the actual captioning
workflow; the scripted demo is a secondary setup aid.

For 1.12.0, What's new is replaced in all three languages: one paragraph on Captions in Zoom
([#189](https://github.com/fmadore/Live-translation/pull/189), part of
[#187](https://github.com/fmadore/Live-translation/issues/187)) and one on the fixes (#188 and
the pre-flight's application prompt). Each description gains four sentences: one in the overlay
paragraph, after the audio sources, on sharing a caption language with a Zoom meeting; one in
the privacy paragraph saying that sharing sends caption text to Zoom; one in the requirements
paragraph on the host's Zoom settings; and, as a last paragraph, the trademark line "Zoom is a
trademark of Zoom Communications, Inc." with a statement that the app is not affiliated with
Zoom (Store Policies 10.1 and 11.2; see [store-listing-next.md](store-listing-next.md)). The
features gain a twentieth line, the most Partner Center allows. The certification notes change
in their first paragraph (the version, and what is new since 1.11.0) and gain a second, on
Captions in Zoom; the paragraph on the support link follows it, unchanged. The short
descriptions, keywords and requirements are unchanged. The German and French sentences, including Zoom's own menu names, were reviewed by the
maintainer on 27 October 2026.

Every paragraph inside a block is one unwrapped line, because Partner Center keeps line
breaks; blank lines separate paragraphs. Prettier ignores Markdown in this repository
(`.prettierignore`), so nothing re-wraps them.

Microsoft's [listing field documentation](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
allows 10,000 characters for Description, 1,500 for What's new, and 20 features of 200
characters each. Short description accepts 1,000 characters, but only the first 270 appear
in some views. All three short descriptions below fit within 270 characters. Keywords allow
up to 7 entries of at most 40 characters, with no more than 21 words across all of
them; enter each line below as one keyword.

Each language's blocks follow Partner Center's field order: Description, What's new,
Features, Short description, Keywords.

## English (United States)

### Description — full text

```text
Put live captions over any meeting, lecture, video call or slide deck. Live Translation & Subtitles transcribes speech on your own PC with Whisper: download a model once, and captions run offline with no account or subscription. Whisper can also translate into English offline. When your audience needs another language, add cloud translation with your own Google Gemini or OpenAI API key; provider usage charges may apply.

Captions appear in a transparent overlay that stays on top of your slides, call or any other Windows app, and lets clicks pass through to the window beneath. It reopens where you last placed it, remembered for each display setup. Choose what to listen to: a room microphone, the audio playing on your PC, a single app, or the microphone and system audio together. In a Zoom meeting, you can also share one caption language with Zoom's own caption bar, using the caption link the host copies in Zoom; attendees turn the captions on in Zoom and install nothing. Gamers can follow voice chat in another language: the overlay is a separate window, not something injected into the game, so run the game in windowed or borderless mode.

For offline subtitles, pick a multilingual Whisper model and one of 99 spoken languages, or let the app detect the language. The Tiny, Base and Small models download separately, at about 31, 57 and 182 MiB. Once a model is installed, recognition runs on your CPU with no account, API key or internet connection. Speed and accuracy depend on the computer, the model and the language. Whisper captions in the language being spoken, or translates it into English; for translation, choose Small, as Tiny and Base translate poorly. For any other language, use cloud translation.

For multilingual audiences, translate with Gemini into 78 languages or with OpenAI into 13. Show two languages at once, or set the original speech beneath the translation in Fit window or Compact when the engine provides it. A second language opens another cloud session for each audio source and adds to provider usage. Gemini and Mistral also offer cloud subtitles in the spoken language, with your own API key; for Mistral, a list of names and terms helps it spell speaker names, institutions and acronyms.

Tune captions for the room: typeface, size, colours, backing, how long lines stay on screen and how quickly they update. Choose Fit window, Compact or Stable reading, with support for right-to-left scripts. The optional filler filter hides hesitations from the overlay; edit its word list while the raw transcript stays intact. Save meeting profiles for the rooms you use often, and pause during breaks without splitting the session.

During a session, the Transcript tab shows everything said so far, and Copy puts the last 5, 10 or 30 minutes, or the whole transcript, on the clipboard as plain text. Export transcripts as text, Markdown, SRT or WebVTT. Turn on local history to save finalized captions as you go, then name, search, reopen, copy, export and delete past sessions. History and crash recovery stay off until you enable them. With Whisper, Stop lets queued audio finish so the transcript is complete.

With Whisper, audio stays on your PC; speech waiting to be processed sits in a temporary file that is deleted automatically. With a cloud engine, audio goes straight to the provider you chose. Sharing captions with Zoom, which only starts when you choose it, sends caption text, never audio, directly to Zoom. API keys are kept in Windows Credential Manager. The developer receives no audio, keys, transcripts or telemetry, and the app sells no subscriptions, credits or API access. An optional Buy Me a Coffee link supports development and unlocks nothing.

Use the interface in English, French or German, whatever the caption language. The operator window works from the keyboard and with Narrator, and follows Windows text size and contrast themes. An optional English/French demo lets you try the display without any setup; it plays a prepared script and does not recognize speech.

Requires Windows 11 and the Microsoft Edge WebView2 Runtime, with native x64 and ARM64 packages. On x64, Whisper needs a processor with AVX2; the cloud engines work without it. Download a Whisper model before using it offline, and allow access to the audio source you select. Capturing a single app also captures its child processes and notifications, and may include several browser tabs. Sharing with Zoom needs a meeting whose host has turned on manual captions and allowed the caption API token for third-party services.

Zoom is a trademark of Zoom Communications, Inc. Live Translation & Subtitles is not affiliated with or endorsed by Zoom.
```

### What's new in this version — 1.12.0

```text
Captions in Zoom: show one caption language in a Zoom meeting's own caption bar, so attendees turn captions on in Zoom and install nothing. Choose Zoom captions in the toolbar, paste the caption link (API token) the meeting's host copies in Zoom, and start sharing; Send test caption confirms that it arrives. Sharing is off until you start it, carries on across Stop and Start, and sends caption text only, never audio, even when Whisper makes the captions on your PC. The link is kept only until the app closes.

Fixes: Start now says when it is waiting for you to choose an application to capture, and the setup and transcript no longer flash or jump when lists load, keys are checked or you scroll.
```

### Features — one feature per line

```text
Offline Whisper captions in 99 spoken languages, or automatic language detection
Download Tiny, Base or Small once, then transcribe without an account, API key or internet
Offline translation into English with Whisper, from any of its 99 languages, with no API key
Cloud translation with your own Gemini or OpenAI key; provider usage charges may apply
Cloud subtitles with your own Gemini or Mistral key; Mistral takes a list of names and terms to favour
Two translation languages at once, plus optional original speech when available
Capture a microphone, system audio, a single app, or microphone and system audio together
Transparent, always-on-top, click-through captions over slides and video calls
Follow voice chat in online games in another language, in a separate caption window with nothing injected into the game
Overlay reopens where you last placed it, remembered for each display setup
Fit window, Compact and Stable reading, with adjustable appearance and right-to-left support
Pause and resume, reusable meeting profiles, searchable languages and favourites
Editable filler-word filter for the overlay, with raw transcripts preserved
Read the whole transcript during a session and copy the last 5, 10 or 30 minutes, or all of it
Text, Markdown, SRT and WebVTT export, including available original speech
Optional local transcript history with titles, search, reopening and deletion
Keyboard shortcuts and Narrator support; follows Windows text size and contrast themes
Light or dark operator window, or follow the light or dark app mode set in Windows
English, French and German interface; native Windows x64 and ARM64
Share one caption language with a Zoom meeting's own caption bar, using the host's caption link
```

### Short description

```text
Free live captions over meetings, calls and slides. Transcribe offline with Whisper in 99 languages or translate into English, or use your own cloud API key for other languages. Capture a microphone or any app, show two caption languages and export the transcript.
```

### Keywords — up to 7

```text
live captions
offline Whisper transcription
game voice chat translation
real-time translation
speech to text
meeting accessibility
on-device speech recognition
```

## Français (France)

### Description — texte complet

```text
Affichez des sous-titres en direct sur vos réunions, cours, appels vidéo et présentations. Live Translation & Subtitles transcrit la parole sur votre propre PC avec Whisper. Après le téléchargement unique d'un modèle, les sous-titres fonctionnent hors ligne, sans compte ni abonnement. Whisper peut aussi traduire vers l'anglais hors ligne. Si votre public a besoin d'une autre langue, ajoutez la traduction cloud avec votre propre clé API Google Gemini ou OpenAI ; des frais d'utilisation du fournisseur peuvent s'appliquer.

Les sous-titres s'affichent dans une surimpression transparente qui reste au-dessus de vos diapositives, de votre appel ou de toute autre application Windows, et laisse passer les clics vers la fenêtre en dessous. Elle se rouvre là où vous l'avez placée, mémorisée pour chaque configuration d'écrans. Choisissez la source : un microphone de salle, l'audio joué sur votre PC, une application précise, ou le microphone et l'audio système ensemble. Dans une réunion Zoom, vous pouvez aussi partager une langue de sous-titres dans la barre de sous-titres de Zoom, avec le lien de sous-titrage que l'organisateur copie dans Zoom ; les participants activent les sous-titres dans Zoom, sans rien installer. Les joueurs peuvent suivre le chat vocal dans une autre langue : la surimpression est une fenêtre distincte, rien n'est injecté dans le jeu ; lancez donc le jeu en mode fenêtré ou fenêtré sans bordure.

Pour les sous-titres hors ligne, choisissez un modèle Whisper multilingue et l'une des 99 langues parlées, ou laissez l'application détecter la langue. Les modèles Tiny, Base et Small se téléchargent séparément et pèsent environ 31, 57 et 182 Mio. Une fois le modèle installé, la reconnaissance s'effectue sur le processeur de votre PC, sans compte, clé API ni connexion Internet. La vitesse et la précision dépendent de l'ordinateur, du modèle et de la langue. Whisper sous-titre dans la langue parlée ou traduit la parole vers l'anglais ; pour traduire, choisissez Small, car Tiny et Base traduisent mal. Pour toute autre langue, utilisez la traduction cloud.

Pour un public multilingue, traduisez avec Gemini vers 78 langues ou avec OpenAI vers 13. Affichez deux langues à la fois, ou placez le texte original sous la traduction en mode Adapter à la fenêtre ou Compact lorsque le moteur le fournit. Une seconde langue ouvre une session cloud supplémentaire par source audio et augmente l'utilisation facturée par le fournisseur. Gemini et Mistral proposent aussi des sous-titres cloud dans la langue parlée, avec votre propre clé API ; pour Mistral, une liste de noms et termes l'aide à écrire correctement les noms d'intervenants, les institutions et les sigles.

Adaptez les sous-titres à la salle : police, taille, couleurs, fond, durée d'affichage et rythme des mises à jour. Choisissez Adapter à la fenêtre, Compact ou Lecture stable, avec prise en charge des écritures de droite à gauche. Le filtre d'hésitations facultatif les masque dans la surimpression, et vous pouvez modifier sa liste de mots sans toucher à la transcription originale. Enregistrez des profils de réunion pour vos salles habituelles et faites une pause sans scinder la session.

Pendant une session, l'onglet Transcription affiche tout ce qui a été dit, et Copier place les 5, 10 ou 30 dernières minutes, ou toute la transcription, dans le presse-papiers en texte brut. Exportez les transcriptions en texte, Markdown, SRT ou WebVTT. Activez l'historique local pour enregistrer les sous-titres finalisés au fil de la session, puis nommez, recherchez, rouvrez, copiez, exportez et supprimez vos sessions. L'historique et la copie de récupération restent désactivés tant que vous ne les activez pas. Avec Whisper, la commande Arrêter laisse l'audio en attente finir son traitement pour que la transcription soit complète.

Avec Whisper, l'audio reste sur votre PC ; la parole en attente de traitement est placée dans un fichier temporaire supprimé automatiquement. Avec un moteur cloud, l'audio est envoyé directement au fournisseur choisi. Le partage des sous-titres avec Zoom, qui ne démarre que si vous le lancez, envoie directement à Zoom le texte des sous-titres, jamais l'audio. Les clés API sont conservées dans le Gestionnaire d'informations d'identification Windows. Le développeur ne reçoit ni audio, ni clé, ni transcription, ni télémétrie, et l'application ne vend ni abonnement, ni crédits, ni accès API. Un lien facultatif vers Buy Me a Coffee permet de soutenir le développement et ne débloque rien.

Utilisez l'interface en français, anglais ou allemand, quelle que soit la langue des sous-titres. La fenêtre de contrôle s'utilise au clavier et avec le Narrateur, et suit la taille du texte et les thèmes de contraste de Windows. Une démo facultative en français et en anglais permet d'essayer l'affichage sans configuration ; elle déroule un script préparé et ne reconnaît pas la parole.

Nécessite Windows 11 et Microsoft Edge WebView2 Runtime, avec des paquets x64 et ARM64 natifs. Sur x64, Whisper nécessite un processeur compatible AVX2 ; les moteurs cloud fonctionnent sans. Téléchargez un modèle Whisper avant de l'utiliser hors ligne et autorisez l'accès à la source audio choisie. La capture d'une application inclut aussi ses processus enfants et ses notifications, et peut inclure plusieurs onglets d'un navigateur. Le partage avec Zoom nécessite une réunion dont l'organisateur a activé le sous-titrage manuel et autorisé le jeton d'API de sous-titrage pour les services tiers.

Zoom est une marque de Zoom Communications, Inc. Live Translation & Subtitles n'est ni affiliée à Zoom ni approuvée par Zoom.
```

### Nouveautés de cette version — 1.12.0

```text
Sous-titres dans Zoom : affichez une langue de sous-titres dans la barre de sous-titres d'une réunion Zoom ; les participants les activent dans Zoom, sans rien installer. Choisissez Sous-titres Zoom dans la barre d'outils, collez le lien de sous-titrage (jeton d'API) que l'organisateur copie dans Zoom, puis lancez le partage ; Envoyer un sous-titre de test confirme qu'il arrive. Le partage reste désactivé tant que vous ne le lancez pas, continue après Arrêter et Démarrer, et n'envoie que le texte des sous-titres, jamais l'audio, même quand Whisper les produit sur votre PC. Le lien n'est conservé que jusqu'à la fermeture de l'application.

Corrections : Démarrer indique désormais qu'il attend le choix d'une application à capter, et la configuration et la transcription ne clignotent ni ne sautent plus quand les listes se chargent, quand les clés sont vérifiées ou quand vous faites défiler.
```

### Fonctionnalités — une fonctionnalité par ligne

```text
Sous-titres Whisper hors ligne dans 99 langues parlées, ou détection automatique de la langue
Téléchargez Tiny, Base ou Small, puis transcrivez sans compte, clé API ni connexion Internet
Traduction hors ligne vers l'anglais avec Whisper, depuis ses 99 langues, sans clé API
Traduction cloud avec votre clé Gemini ou OpenAI ; des frais d'utilisation peuvent s'appliquer
Sous-titres cloud avec votre clé Gemini ou Mistral, et une liste de noms et termes à privilégier pour Mistral
Deux langues de traduction à la fois et affichage facultatif du texte original disponible
Captez un microphone, l'audio système, une application précise, ou le microphone et l'audio système ensemble
Sous-titres transparents, toujours visibles et traversables par les clics, sur vos diapositives et appels
Suivez le chat vocal des jeux en ligne dans une autre langue, dans une fenêtre de sous-titres distincte, sans rien injecter dans le jeu
La surimpression se rouvre là où vous l'avez placée, pour chaque configuration d'écrans
Adapter à la fenêtre, Compact et Lecture stable ; apparence réglable et écritures de droite à gauche
Pause et reprise, profils de réunion, recherche de langues et favoris
Liste d'hésitations modifiable pour la surimpression, avec conservation de la transcription originale
Relecture de toute la transcription en cours de session et copie des 5, 10 ou 30 dernières minutes, ou de l'ensemble
Export texte, Markdown, SRT et WebVTT, avec texte original lorsqu'il est disponible
Historique local facultatif avec titres, recherche, réouverture et suppression
Raccourcis clavier et prise en charge du Narrateur ; suit la taille du texte et les thèmes de contraste de Windows
Fenêtre de contrôle claire ou sombre, ou qui suit le mode d'application clair ou sombre de Windows
Interface en français, anglais et allemand ; versions Windows x64 et ARM64 natives
Partage d'une langue de sous-titres dans la barre de sous-titres d'une réunion Zoom, avec le lien de l'organisateur
```

### Description courte

```text
Sous-titres en direct gratuits sur vos réunions, appels et présentations. Transcrivez hors ligne avec Whisper en 99 langues ou traduisez vers l'anglais, et vers d'autres langues avec votre clé API cloud. Captez un micro ou une application et exportez la transcription.
```

### Mots clés — 7 au maximum

```text
sous-titres en direct
transcription Whisper
traduction vocale en jeu
traduction instantanée
transcription vocale
réunions accessibles
reconnaissance vocale hors ligne
```

## Deutsch (Deutschland)

### Beschreibung — vollständiger Text

```text
Verfolgen Sie Besprechungen, Vorträge und Videoanrufe mit Untertiteln direkt über Ihren Inhalten. Live Translation & Subtitles transkribiert Sprache lokal mit Whisper nach einem einmaligen Modelldownload und übersetzt sie auf Wunsch offline ins Englische. Optionale Cloud-Übersetzungen und Cloud-Untertitel benötigen Ihr eigenes Konto mit API-Schlüssel bei Google Gemini, OpenAI oder Mistral; dabei können nutzungsabhängige Kosten entstehen.

Platzieren Sie ein transparentes Untertitel-Overlay über Folien, einem Anruf oder einer anderen Windows-App. Erfassen Sie ein Mikrofon, das Windows-Systemaudio, eine Anwendung mit ihren Unterprozessen oder Mikrofon und Systemaudio gemeinsam. In einem Zoom-Meeting können Sie außerdem eine Untertitelsprache in Zooms eigener Untertitelleiste teilen, mit dem Untertitel-Link, den der Host in Zoom kopiert; die Teilnehmenden schalten die Untertitel in Zoom ein und installieren nichts. Das Overlay bleibt im Vordergrund und lässt Mausklicks zur darunterliegenden Anwendung durch. Es öffnet sich wieder dort, wo Sie es zuletzt platziert haben, gespeichert für jede Bildschirmanordnung. Auch beim Spielen lässt sich der Sprachchat in einer anderen Sprache verfolgen: Das Overlay ist ein eigenes Fenster, in das Spiel wird nichts eingeschleust. Starten Sie das Spiel daher im Fenstermodus oder randlosen Fenstermodus.

Für Offline-Untertitel wählen Sie ein mehrsprachiges Whisper-Modell und eine von 99 gesprochenen Sprachen oder die automatische Erkennung. Tiny, Base und Small werden separat heruntergeladen (etwa 31, 57 und 182 MiB). Danach läuft die Erkennung auf der CPU Ihres PCs, ohne Konto, API-Schlüssel oder Internetverbindung. Geschwindigkeit und Genauigkeit hängen von Computer, Modell und Sprache ab. Whisper transkribiert in der gesprochenen Sprache oder übersetzt ins Englische; zum Übersetzen wählen Sie Small, da Tiny und Base schlecht übersetzen. Für andere Sprachen nutzen Sie die Cloud-Übersetzung.

Für ein mehrsprachiges Publikum übersetzen Sie mit Gemini oder OpenAI. Die App bietet 78 Zielsprachen für Gemini und 13 für OpenAI. Zeigen Sie zwei Zielsprachen gleichzeitig oder den Originaltext unter der Übersetzung in An Fenster anpassen oder Kompakt, sofern die Engine ihn liefert. Eine zweite Zielsprache öffnet für jede Audioquelle eine zusätzliche Cloud-Sitzung und erhöht die Anbieternutzung. Gemini und Mistral bieten auch Cloud-Untertitel in der gesprochenen Sprache; für Mistral hilft eine Liste mit Namen und Begriffen bei der Schreibweise von Namen der Vortragenden, Institutionen und Abkürzungen.

Passen Sie Schrift, Größe, Farbe, Kontrast, Anzeigedauer und Aktualisierungstempo an. Wählen Sie An Fenster anpassen, Kompakt oder Ruhiger Lesemodus, auch für Text von rechts nach links. Bearbeiten Sie die Wörter des optionalen Füllwortfilters, während das Originaltranskript erhalten bleibt. Nutzen Sie gespeicherte Besprechungsprofile und pausieren Sie, ohne die Sitzung aufzuteilen.

Während einer Sitzung zeigt die Ansicht Transkript alles bisher Gesagte, und Kopieren legt die letzten 5, 10 oder 30 Minuten oder das ganze Transkript als reinen Text in die Zwischenablage. Exportieren Sie Transkripte als Text, Markdown, SRT oder WebVTT. Der optionale lokale Verlauf speichert abgeschlossene Untertitel fortlaufend; Sie können Sitzungen benennen, durchsuchen, erneut öffnen, kopieren, exportieren und löschen. Verlauf und Wiederherstellungskopie sind standardmäßig deaktiviert. Bei Whisper verarbeitet Stopp noch ausstehendes Audio, bevor das Transkript vollständig ist.

Whisper-Audio bleibt auf dem PC; ausstehende Sprache wird in einer automatisch gelöschten temporären Datei gepuffert. Cloud-Audio geht direkt an den gewählten Anbieter. Das Teilen von Untertiteln mit Zoom, das erst startet, wenn Sie es wählen, sendet Untertiteltext direkt an Zoom, nie Audio. API-Schlüssel liegen in der Windows-Anmeldeinformationsverwaltung. Der Entwickler erhält weder Audio noch Schlüssel, Transkripte oder Telemetrie. Die App verkauft keine Abonnements, Guthaben oder API-Zugänge. Ein freiwilliger Buy-Me-a-Coffee-Link unterstützt die Entwicklung und schaltet nichts frei.

Die Oberfläche ist auf Deutsch, Englisch und Französisch verfügbar, unabhängig von der Untertitelsprache. Das Bedienfenster lässt sich per Tastatur und mit der Sprachausgabe bedienen und folgt der Windows-Textgröße und den Kontrastdesigns. Eine optionale skriptbasierte Demo auf Englisch und Französisch zeigt die Anzeigefunktionen ohne Einrichtung; sie erkennt keine Sprache.

Erfordert Windows 11 und die Microsoft Edge WebView2-Runtime. Native x64- und ARM64-Pakete sind verfügbar. Auf x64 benötigt Whisper einen Prozessor mit AVX2; die Cloud-Engines funktionieren auch ohne. Laden Sie vor der Offline-Nutzung ein Whisper-Modell herunter und erlauben Sie den Zugriff auf die ausgewählte Audioquelle. Die Aufnahme einer Anwendung umfasst ihre Unterprozesse und Benachrichtigungen und kann mehrere Browser-Tabs einschließen. Das Teilen mit Zoom erfordert ein Meeting, dessen Host manuelle Untertitel aktiviert und das Untertitel-API-Token für Drittanbieterdienste erlaubt hat.

Zoom ist eine Marke von Zoom Communications, Inc. Live Translation & Subtitles ist nicht mit Zoom verbunden und wird nicht von Zoom unterstützt.
```

### Neu in dieser Version — 1.12.0

```text
Untertitel in Zoom: Zeigen Sie eine Untertitelsprache in der eigenen Untertitelleiste eines Zoom-Meetings; die Teilnehmenden schalten sie in Zoom ein und installieren nichts. Wählen Sie Zoom-Untertitel in der Symbolleiste, fügen Sie den Untertitel-Link (API-Token) ein, den der Host in Zoom kopiert, und starten Sie das Teilen; Testuntertitel senden bestätigt, dass er ankommt. Das Teilen bleibt aus, bis Sie es starten, läuft über Stopp und Start hinweg weiter und sendet nur Untertiteltext, nie Audio, auch wenn Whisper die Untertitel auf Ihrem PC erzeugt. Der Link wird nur bis zum Schließen der App aufbewahrt.

Korrekturen: Start sagt jetzt, wenn es auf die Auswahl einer aufzunehmenden Anwendung wartet, und Einrichtung und Transkript flackern oder springen nicht mehr, wenn Listen laden, Schlüssel geprüft werden oder Sie scrollen.
```

### Funktionen — eine Funktion pro Zeile

```text
Offline-Untertitel mit Whisper: 99 gesprochene Sprachen oder automatische Erkennung
Tiny, Base oder Small herunterladen und danach ohne Konto, API-Schlüssel oder Internet transkribieren
Offline-Übersetzung ins Englische mit Whisper aus allen 99 Sprachen, ohne API-Schlüssel
Cloud-Übersetzung mit eigenem Gemini- oder OpenAI-Schlüssel; nutzungsabhängige Kosten möglich
Cloud-Untertitel mit eigenem Gemini- oder Mistral-Schlüssel; für Mistral eine Liste mit Namen und Begriffen
Zwei Übersetzungssprachen gleichzeitig und optional der verfügbare Originaltext
Mikrofon, Systemaudio, eine Anwendung mit Unterprozessen oder Mikrofon und Systemaudio gemeinsam
Transparente, durchklickbare Untertitel im Vordergrund über Folien und Videoanrufen
Sprachchat in Onlinespielen in einer anderen Sprache verfolgen, in einem eigenen Untertitelfenster, ohne Eingriff in das Spiel
Das Overlay öffnet sich dort, wo Sie es zuletzt platziert haben, für jede Bildschirmanordnung
An Fenster anpassen, Kompakt und Ruhiger Lesemodus; anpassbare Darstellung und Text von rechts nach links
Pause und Fortsetzen, Besprechungsprofile, Sprachsuche und Favoriten
Bearbeitbarer Füllwortfilter für das Overlay; das Originaltranskript bleibt erhalten
Transkript während der Sitzung lesen und die letzten 5, 10 oder 30 Minuten oder alles kopieren
Export als Text, Markdown, SRT und WebVTT, einschließlich verfügbarem Originaltext
Optionaler lokaler Transkriptverlauf mit Titeln, Suche, erneutem Öffnen und Löschen
Tastenkombinationen und Unterstützung der Sprachausgabe; folgt Windows-Textgröße und Kontrastdesigns
Bedienfenster hell oder dunkel, oder passend zum hellen oder dunklen App-Modus von Windows
Oberfläche auf Deutsch, Englisch und Französisch; native Windows-Versionen für x64 und ARM64
Eine Untertitelsprache in der Untertitelleiste eines Zoom-Meetings teilen, mit dem Link des Hosts
```

### Kurzbeschreibung

```text
Untertitel über Meetings, Anrufen und Folien. Mit Whisper offline in 99 Sprachen transkribieren oder ins Englische übersetzen, andere Sprachen per Cloud mit eigenem API-Schlüssel. Mikrofon oder App-Audio erfassen, zwei Sprachen anzeigen und Transkripte exportieren.
```

### Suchbegriffe — höchstens 7

```text
Live-Untertitel
Offline-Transkription mit Whisper
Sprachchat-Übersetzung für Spiele
Echtzeit-Übersetzung
Sprache zu Text
barrierefreie Meetings
Offline-Spracherkennung
```

## Adding the German listing

The interface supports German. Before publishing a German listing, review the German copy.
The German screenshots are captured from the 1.7.1 MSIX with the interface set to German; see
the [screenshot notes](store-screenshots/README.md).

## Additional system requirements

Enter each line as a separate requirement (under 200 characters):

```text
Windows 11 with Microsoft Edge WebView2 Runtime; native x64 and ARM64 packages
Whisper: internet for the initial model download (31–182 MiB), then offline transcription or translation into English on the CPU
Whisper on x64 needs a processor with AVX2, which some budget Celeron and Pentium Silver PCs lack
Cloud engines: internet, your own compatible provider account and API key; usage charges may apply
Audio-source access required for speech recognition; a microphone is needed only for microphone capture
```

Recommended memory: 8 GB. Whisper speed depends on the CPU and model; this is not a guarantee
of real-time processing on all supported hardware. Localized requirements:

### Configuration supplémentaire — français

```text
Windows 11 et Microsoft Edge WebView2 Runtime ; paquets x64 et ARM64 natifs
Whisper : Internet pour télécharger le modèle (31–182 Mio), puis transcription ou traduction vers l'anglais hors ligne sur le processeur
Whisper sur x64 nécessite un processeur compatible AVX2, absent de certains PC Celeron et Pentium Silver d'entrée de gamme
Moteurs cloud : Internet, votre compte et clé API compatibles ; des frais d'utilisation peuvent s'appliquer
Accès à la source audio pour la reconnaissance ; microphone nécessaire uniquement pour la capture du micro
```

### Zusätzliche Systemanforderungen — deutsch

```text
Windows 11 mit Microsoft Edge WebView2-Runtime; native Pakete für x64 und ARM64
Whisper: Internet für den Modelldownload (31–182 MiB), danach Offline-Transkription oder Übersetzung ins Englische auf der CPU
Whisper auf x64 erfordert einen Prozessor mit AVX2; manche günstigen Celeron- und Pentium-Silver-PCs haben keinen
Cloud-Engines: Internet, eigenes kompatibles Konto und API-Schlüssel; nutzungsabhängige Kosten möglich
Zugriff auf die Audioquelle für Spracherkennung; ein Mikrofon ist nur für Mikrofonaufnahme erforderlich
```

## Notes for certification

```text
Product: Live Translation & Subtitles. Product ID: 9PFB8LR3RR9X. Version: 1.12.0.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Since 1.11.0, the version in the Store, this update adds optional Captions in Zoom, described in the next paragraph; makes the pre-flight say why Start waits when one application is to be captured and none is chosen; and removes flashes and layout shifts in the setup and transcript. It adds no capability and no product declaration. The test routes below are those of 1.11.0.

Optional Captions in Zoom (off until the user starts it): the Zoom captions button in the operator window's toolbar opens a dialog where the user pastes the caption link (API token) that a Zoom meeting's host copies with Show Captions > Set up manual captioner > Copy the API token, and chooses Start sharing. From then until Stop sharing or the app quits, the app posts finished caption text in one caption language over HTTPS directly to that zoom.us or zoomgov.com address, and Zoom shows it in the meeting's caption bar. This includes captions made offline with Whisper or by the built-in demo; no audio, API key or device information is sent, and nothing passes through a developer server. The dialog says that caption text is sent to Zoom above its Start sharing button, and the privacy policy describes the flow. The link is held in memory only and never saved. To exercise it, host a Zoom meeting (a free Basic account is enough) on an account whose settings have Manual captions on and the third-party caption API token allowed, copy the API token in the meeting, paste it, start sharing, choose Send test caption, then run the built-in demo or Whisper: attendees see the captions after choosing Show Captions. Without a Zoom meeting, open the dialog and paste any https address outside Zoom: it is refused locally with an explanation and nothing is sent. The rest of the app works the same whether or not Zoom is used. Zoom is named descriptively only; the app uses no Zoom logo and is not affiliated with Zoom Communications, Inc.

Voluntary support link (Store Policy 10.8.2): the Support development section at the end of Settings › App, and the credit line at the foot of the start screen while no session runs, open https://buymeacoffee.com/fmadore in the user's default browser. Buy Me a Coffee, a third-party service, identifies itself, authenticates the supporter and takes any payment on its own site. The app has no in-app purchase and collects no payment or financial information. A donation gives nothing in return: every feature stays free, and nothing is unlocked, removed or changed. The link is never shown during a session and the app never prompts for support. The Product declaration "This product allows users to make purchases, but does not use the Microsoft Store commerce system" is ticked for this link.

Real speech recognition without a paid account or API key:
1. A clean install opens on Subtitles / Whisper / Base / Detect automatically, with the default microphone selected. No download or capture starts automatically. On an x64 processor without AVX2, Whisper is listed as unavailable with the reason and a clean install opens on Built-in demo instead; use the setup-free check below. Choose the microphone or system audio source. Grant microphone access if using a microphone. Existing saved setups are preserved on upgrade.
2. Choose the spoken language or Detect automatically. Download Tiny (about 31 MiB) for the lightest setup, or Base (about 57 MiB). The initial HTTPS model download needs internet access; the model is not bundled. No account or key is needed.
3. Start captioning and speak a few sentences or play speech on the selected output. Allow for CPU processing time. Verify captions in the operator window and overlay, elapsed time and input levels. Then select the Transcript tab to read the session so far, choose Last 5 minutes beside Copy, click Copy and paste into Notepad: the captions arrive as plain text.
4. Use Move overlay to place it, lock it back to click-through, and try Fit window, Compact and Stable reading. After a restart, the overlay reopens where it was placed. Pause blocks new speech entering recognition; existing pending audio can still finish.
5. Stop and let Finishing the transcript complete. Export text or Markdown, then SRT or WebVTT. Disconnect the network and repeat with the installed model to verify offline operation.
6. In Settings, optionally enable transcript history; run a disposable session and reopen, rename, export and delete it. History and recovery are disabled by default. Test the editable filler-word list: it changes only the overlay.
7. Optional offline translation: choose Translation, then Local Whisper, keep English as the caption language, choose the spoken language or Detect automatically, and download Small (about 182 MiB). Speak or play non-English speech, or click Rehearse to play a bundled French recording with no microphone: English captions appear with no account or key. Tiny and Base also work but translate poorly.

Setup-free display check if capture hardware or model downloads are unavailable:
In Subtitles, select Built-in demo in the engine list, choose English and click Start demo subtitles. The bundled English/French script exercises status, meter animation, timer, overlay, Stop and export without devices, network or credentials. Repeat with French. It is explicitly a scripted demo and does not recognize speech; Whisper above is the first-launch local speech engine.

Optional cloud features:
Gemini/OpenAI translation and Gemini/Mistral subtitles require the user's compatible third-party account and API key and may incur provider charges. The app supplies no cloud account or credits. Keys are entered in the app and kept in Windows Credential Manager. Cloud testing requires suitable test credentials to be arranged separately before submission. Never place a personal key in public listing fields or screenshots. Choose two supported translation targets to check bilingual output; this creates a second provider session per audio source. Original speech is shown only when the provider supplies it. Mistral subtitles now use Voxtral Realtime 3 (voxtral-mini-transcribe-realtime-3-0), which Mistral lists as a public preview. Entries in the optional Names and terms list, shown in the setup when Mistral is selected, are sent to Mistral with the session.

The full-trust desktop process performs WASAPI/microphone capture, CPU Whisper transcription and translation, user-requested verified model downloads, Credential Manager access, window/overlay management and explicit transcript export. Whisper buffers pending audio in an automatically deleted local temporary file. Cloud engines send audio directly to the selected provider. Captions in Zoom, when the user starts it, sends caption text directly to the Zoom address the user pasted. There is no developer relay, telemetry, driver, service, auto-start task, Windows AI component or required speech language pack.
```

Resolve cloud test credentials or Microsoft reviewer access requirements before submission;
the credential-free Whisper and demo routes do not establish acceptance of cloud features.

## Screenshots

One caption per image in [`store-screenshots/`](store-screenshots/README.md), captured from
the 1.7.1 MSIX in each listing language. Each caption is under 200 characters. The images
predate the Live and Transcript tabs of 1.8.0, so none of them shows the tabs; the captions are
unchanged.

| Image                            | English                                                                                                       | Français                                                                                                                      | Deutsch                                                                                                                        |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 1 — Whisper                      | Offline captions with Whisper: choose a model, spoken language and audio source.                              | Sous-titres hors ligne avec Whisper : choisissez un modèle, une langue et une source audio.                                   | Offline-Untertitel mit Whisper: Modell, gesprochene Sprache und Audioquelle auswählen.                                         |
| 2 — Overlay                      | Keep captions visible over your slides or video call in a transparent, click-through overlay.                 | Gardez les sous-titres visibles sur vos diapositives ou appels, dans une surimpression traversable par les clics.             | Untertitel bleiben über Folien oder Videoanrufen sichtbar, in einem transparenten, durchklickbaren Overlay.                    |
| 3 — Bilingual                    | Show two translation languages and include the original speech when the engine supplies it.                   | Affichez deux langues de traduction et le texte original lorsque le moteur le fournit.                                        | Zwei Übersetzungssprachen anzeigen und den Originaltext einblenden, wenn die Engine ihn liefert.                               |
| 4 — Appearance                   | Choose caption size, typeface and colours, with presets previewed on a bright and a dark slide.               | Choisissez la taille, la police et les couleurs, avec des préréglages prévisualisés sur une diapositive claire et une sombre. | Größe, Schrift und Farben wählen, mit Voreinstellungen in der Vorschau auf einer hellen und einer dunklen Folie.               |
| 5 — Reading                      | Set the reading pace: Stable reading holds captions still, and your own filler-word list hides “um” and “uh”. | Réglez le rythme de lecture : la lecture stable fige les sous-titres, et votre liste masque les « euh ».                      | Lesetempo einstellen: Der ruhige Lesemodus hält Untertitel still, und eine eigene Füllwortliste blendet Verzögerungslaute aus. |
| 6 — History                      | Find a saved session, read its transcript and export text or timed subtitles.                                 | Retrouvez une session, relisez sa transcription et exportez du texte ou des sous-titres minutés.                              | Gespeicherte Sitzungen finden, Transkripte lesen und Text oder zeitbasierte Untertitel exportieren.                            |
| 7 — Optional demo (not captured) | Try the display controls with a scripted English/French demo, without audio capture or setup.                 | Essayez l'affichage avec une démo scénarisée en français ou en anglais, sans capture audio ni configuration.                  | Anzeige mit einer englischen oder französischen Skript-Demo ausprobieren, ohne Audioaufnahme oder Einrichtung.                 |
