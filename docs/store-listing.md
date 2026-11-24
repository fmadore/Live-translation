# Partner Center listing text — 1.16.1

Written for **1.16.1 / MSIX 1.16.1.0**, prepared on 24 November 2026. The Store serves 1.16.0
(MSIX 1.16.0.0), read from the installed package on 24 November 2026
([handoff](store-updates.md#release-1161-handoff)). For 1.16.1 only What's new and the
certification notes' first paragraph change: What's new leads with Gemini's move to verbatim
transcription, which 1.16.0's text had already announced, then sums up 1.16 for users updating
from 1.15; the certification notes say what changed since 1.16.0. The descriptions, features,
short descriptions, keywords, requirements, screenshots and trademark line are 1.16.0's, kept
below; the French and German What's new need the maintainer's review. Paste each block into its
named field after the [release checks](store-updates.md). These descriptions lead with the
actual captioning workflow; the scripted demo is a secondary setup aid.

What 1.16.0 changed in the listing, kept for reference:

For 1.16.0, What's new is replaced in all three languages: a paragraph on OpenAI subtitles and
Continue with offline Whisper, one on the Gemini subtitles fix and the first words kept at Start
or Resume, and a short list of fixes from the 20 November audit. As planned in
[store-listing-next.md](store-listing-next.md#what-1160-adds), each description changes in its
fourth paragraph (cloud translation and subtitles) only: Gemini, Mistral and OpenAI offer cloud
subtitles, OpenAI on the same key as its translation; OpenAI joins the engines that take the
names and terms; and one new sentence says the app offers to carry on with offline Whisper in
the same session when a cloud engine stops. The German opening paragraph already names every
provider and is unchanged. Two feature lines change in place, so the list stays at twenty: the
first (offline Whisper) adds that it can carry on a session when a cloud engine stops, and the
fifth (cloud subtitles) adds OpenAI. The certification notes change in their first paragraph
(the version, and everything since 1.15.0) and in *Optional cloud features* (OpenAI subtitles,
the consent answer both OpenAI engines share, and the names and terms sent to OpenAI). The short
descriptions, the keywords, the additional system requirements, the screenshot captions and the
trademark line are unchanged. The French and German sentences new in 1.16.0 need the
maintainer's review, as each release's do. The listing is for Windows: it does not mention the
Mac app.

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
Follow any meeting, lecture, video call or slide deck with live captions floating right where people look. Live Translation & Subtitles transcribes speech locally on your PC with Whisper: download a model once and run captions completely offline, with no account or subscription. Whisper also translates into English offline. Whenever your audience reads another language, plug in your own Google Gemini, OpenAI or DeepL API key for live cloud translation (provider usage charges may apply; DeepL requires a paid DeepL API plan).

Captions appear in a transparent overlay that stays above your slides, call or any other Windows app while letting clicks pass straight through to the window underneath. The overlay remembers its position for every monitor setup and reopens right where you left it. Pick the exact audio you want: a room microphone, your PC's system audio, a single application, or microphone and system audio together. In a Zoom meeting, you can also send one caption language straight into Zoom's own caption bar using the API token copied by the host, so attendees simply turn captions on in Zoom with nothing to install. For voice chat in online games, run the game in windowed or borderless mode so the separate overlay window sits cleanly on top without injecting into the game.

For offline subtitles, pick a multilingual Whisper model and one of 99 spoken languages, or let the app detect the language automatically. Download Tiny (31 MiB), Base (57 MiB), Small (182 MiB) or Large v3 Turbo (547 MiB) once; after that, recognition runs locally on your graphics processor when available (or your CPU otherwise) with no internet connection or API key. Built-in speech detection skips silence and music so Whisper does not invent captions over a quiet room. Speed and accuracy depend on your PC, model and language. Whisper captions in the spoken language or translates into English; choose Small for English translation, as Tiny and Base translate poorly and Large v3 Turbo only transcribes. For any other target language, switch to cloud translation.

For multilingual rooms, translate live into 78 languages with Gemini, 42 with DeepL or 13 with OpenAI. DeepL is marked experimental because it has not yet been tested in a live session, and free DeepL API keys are refused. Display two languages at once, or show the original speech beneath the translation in Fit window or Compact when the engine supplies it (a second language opens an extra cloud session per audio source and adds to provider usage). Gemini, Mistral and OpenAI also provide cloud subtitles in the spoken language with your own API key, with OpenAI reusing its translation key. Add a custom list of names and terms so Gemini, Mistral, OpenAI and Whisper spell speaker names, institutions and acronyms right. If a cloud engine drops out mid-session and you have a Whisper model installed, one click switches that source to offline Whisper without ending your session.

Shape the captions to fit the room: adjust typeface, size, colours, backing, hold duration and update pace across Fit window, Compact or Stable reading, with right-to-left script support. Hide hesitations on screen with an editable filler-word filter while keeping the raw transcript untouched. Save meeting profiles for regular rooms, pause during breaks without splitting the session, and put up a break notice such as “Back soon” while paused. Optional system-wide shortcuts let you pause captions or hide the overlay while your slides or call stay in front. With cloud engines, an optional spending guard pauses idle sessions automatically and alerts you when estimated cost or duration passes your chosen limit.

During a session, the Transcript tab lets you read back, search for words and drop markers for new talks, speaker names or section breaks, all preserved in exports and history. Copy the last 5, 10 or 30 minutes (or the full transcript) to the clipboard as plain text, or export to text, Markdown, SRT or WebVTT. Turn on optional local history to save finalised captions as you go, then title, search, reopen, copy, export or delete past sessions. Both history and crash recovery stay off until you enable them. With Whisper, Stop lets queued audio finish decoding so your transcript is complete.

With Whisper, audio never leaves your PC; speech waiting to be decoded sits in an automatically deleted temporary file. Cloud engines stream audio straight to the provider you chose, and the app asks for your permission before sending audio to each provider for the first time. Zoom sharing starts only when you turn it on and sends caption text alone, never audio. API keys live in Windows Credential Manager. The developer receives no audio, keys, transcripts or telemetry, and the app sells no subscriptions, credits or API access. An optional Buy Me a Coffee link supports development and unlocks no extra features.

Run the interface in English, French or German regardless of your caption language. The operator window works cleanly from the keyboard and with Narrator, and respects Windows text size and contrast themes. An optional English/French demo lets you test the overlay with zero setup; it plays a prepared script and does not recognise speech.

Requires Windows 11 and the Microsoft Edge WebView2 Runtime, with native x64 and ARM64 packages. On x64, Whisper requires a processor with AVX2; cloud engines work without it. Download a Whisper model before captioning offline and allow access to your chosen audio source. Capturing a single app also captures its child processes and notifications, and may include multiple browser tabs. Zoom sharing requires a meeting whose host has enabled manual captions and allowed third-party caption API tokens.

Zoom is a trademark of Zoom Communications, Inc. Live Translation & Subtitles is not affiliated with or endorsed by Zoom.
```

### What's new in this version — 1.16.1

```text
Gemini subtitles now arrive word for word, about a second after the speaker pauses. Version 1.16.0 stopped them repeating earlier speech, but they could still lag well behind the speaker and drop a repeated sentence; this update switches Gemini to its verbatim transcription, as 1.16.0 announced. A segment Gemini closes without a final transcript is now kept from the last words on screen.

Also in 1.16: OpenAI live subtitles on your existing OpenAI key; Continue with offline Whisper when a cloud engine stops mid-session, in the same session, transcript and history entry; a spoken-language choice for Gemini; the first words after Start or Resume kept with every cloud engine; and fixes to echo filtering, DeepL sentence endings, Whisper in Chinese, Japanese and Thai, unplugged microphones and projectors, and the cost estimate during pauses.
```

### Features — one feature per line

```text
Offline Whisper captions in 99 spoken languages or auto-detect, using your GPU when available; takes over in one click if a cloud engine stops
Download Tiny, Base, Small or Large v3 Turbo once, then transcribe offline with no account, API key or internet connection
Offline translation into English with Whisper from any of its 99 languages, with no API key
Cloud translation with your own Gemini, OpenAI or DeepL key (DeepL: paid API plan, experimental); provider usage charges apply; optional spending guard
Cloud subtitles with your own Gemini, Mistral or OpenAI key (same OpenAI key as translation); custom names and terms list for Gemini, Mistral, OpenAI and Whisper
Show two translation languages at once, plus original speech underneath when the engine supplies it
Capture a microphone, system audio, a single application, or microphone and system audio together
Transparent, always-on-top, click-through caption overlay over slides, calls and any Windows app
Follow voice chat in online games in another language, in a separate desktop window with nothing injected into the game
Overlay remembers its position for each display setup and reopens where you left it
Fit window, Compact and Stable reading modes, adjustable typography and colours, and right-to-left script support
Pause and resume, break notices on the overlay, optional system-wide shortcuts, meeting profiles, language search and favourites
Editable filler-word filter for the overlay while keeping raw transcripts intact
Read back, search and drop markers in the live transcript; copy the last 5, 10 or 30 minutes or the full session
Export transcripts as plain text, Markdown, SRT or WebVTT, including original speech when available
Optional local transcript history with session titles, search, reopening, bulk export and deletion
Full keyboard control and Narrator support; follows Windows text scaling and contrast themes
Light or dark operator window, or automatically match your Windows app theme
English, French and German interface; native Windows x64 and ARM64 packages
Send one caption language directly to a Zoom meeting's built-in caption bar using the host's API token
```

### Short description

```text
Free live captions over meetings, calls and slides. Transcribe offline with Whisper in 99 languages, translate into English locally, or connect your own cloud API key for other languages. Capture any mic or app, display two languages and export transcripts.
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
Suivez chaque réunion, cours, appel vidéo ou diaporama avec des sous-titres en direct, exactement là où porte le regard. Live Translation & Subtitles transcrit la parole localement sur votre PC grâce à Whisper : téléchargez un modèle une seule fois et sous-titrez entièrement hors ligne, sans compte ni abonnement. Whisper traduit aussi vers l'anglais hors ligne. Quand votre public lit une autre langue, branchez votre propre clé API Google Gemini, OpenAI ou DeepL pour la traduction en direct (des frais d'utilisation du fournisseur peuvent s'appliquer ; DeepL exige un forfait API payant).

Les sous-titres flottent dans une surimpression transparente qui reste au-dessus de vos diapositives, de votre appel ou de n'importe quelle application Windows tout en laissant passer les clics vers la fenêtre dessous. Elle mémorise sa position pour chaque configuration d'écrans et se rouvre exactement où vous l'avez laissée. Captez l'audio qui vous intéresse : un microphone de salle, l'audio système de votre PC, une application précise, ou le micro et l'audio système ensemble. En réunion Zoom, vous pouvez aussi envoyer une langue de sous-titres directement dans la barre de sous-titres intégrée de Zoom avec le jeton d'API copié par l'organisateur : les participants activent simplement les sous-titres dans Zoom, sans rien installer. Pour suivre le chat vocal d'un jeu en ligne dans une autre langue, lancez le jeu en mode fenêtré ou sans bordure : la surimpression est une fenêtre Windows indépendante qui n'injecte rien dans le jeu.

Pour les sous-titres hors ligne, choisissez un modèle Whisper multilingue et l'une des 99 langues parlées, ou laissez la détection automatique faire le travail. Téléchargez Tiny (31 Mio), Base (57 Mio), Small (182 Mio) ou Large v3 Turbo (547 Mio) une fois ; la reconnaissance tourne ensuite sur votre PC, sur la carte graphique si elle convient ou sur le processeur sinon, sans connexion Internet ni clé API. La détection de la parole intégrée ignore les silences et la musique pour éviter que Whisper n'invente du texte dans une salle calme. Vitesse et précision dépendent de votre PC, du modèle et de la langue. Whisper sous-titre dans la langue parlée ou traduit vers l'anglais : privilégiez Small pour traduire, car Tiny et Base traduisent mal et Large v3 Turbo se limite à la transcription. Pour toute autre langue cible, passez à la traduction cloud.

Pour un public multilingue, traduisez en direct vers 78 langues avec Gemini, 42 avec DeepL ou 13 avec OpenAI. DeepL porte la mention expérimental car il n'a pas encore été testé en séance réelle, et les clés API DeepL gratuites sont refusées. Affichez deux langues simultanément, ou le texte original sous la traduction en mode Adapter à la fenêtre ou Compact quand le moteur le fournit (une seconde langue ouvre une session cloud supplémentaire par source audio et augmente la consommation facturée). Gemini, Mistral et OpenAI proposent aussi des sous-titres cloud dans la langue parlée avec votre propre clé API, OpenAI réutilisant sa clé de traduction. Une liste personnalisée de noms et termes aide Gemini, Mistral, OpenAI et Whisper à orthographier correctement noms d'intervenants, institutions et sigles. Si un moteur cloud s'arrête en cours de séance et qu'un modèle Whisper est installé, un clic bascule cette source sur Whisper hors ligne sans interrompre votre session.

Adaptez l'affichage à la salle : police, taille, couleurs, fond, durée d'affichage et rythme de mise à jour, en mode Adapter à la fenêtre, Compact ou Lecture stable, avec prise en charge des écritures de droite à gauche. Masquez les hésitations à l'écran grâce à un filtre modifiable tout en conservant la transcription brute intacte. Enregistrez des profils de réunion pour vos salles habituelles, mettez en pause sans scinder la session et affichez un message de pause tel que « Reprise dans un instant » sur la surimpression. Des raccourcis système facultatifs permettent de mettre en pause ou de masquer la surimpression pendant que vos diapositives ou votre appel restent au premier plan. Avec un moteur cloud, un garde-fou de dépenses facultatif met automatiquement en pause une session silencieuse et vous avertit dès que le coût estimé ou la durée dépasse votre limite.

Pendant la séance, l'onglet Transcription permet de relire, de chercher un mot et de poser des repères (début d'intervention, nom d'orateur, changement de partie), conservés dans les exports et l'historique. Copier place les 5, 10 ou 30 dernières minutes, ou toute la transcription, dans le presse-papiers en texte brut ; exportez sinon en texte, Markdown, SRT ou WebVTT. Activez l'historique local facultatif pour enregistrer les sous-titres finalisés au fil de l'eau, puis renommez, cherchez, rouvrez, copiez, exportez ou supprimez vos séances passées. L'historique et la copie de récupération restent désactivés tant que vous ne les activez pas. Avec Whisper, Arrêter laisse l'audio en file d'attente se terminer pour que la transcription soit complète.

Avec Whisper, l'audio ne quitte jamais votre PC ; la parole en attente de décodage passe par un fichier temporaire supprimé automatiquement. Les moteurs cloud envoient l'audio directement au fournisseur choisi, et l'application demande votre accord avant tout premier envoi vers chaque fournisseur. Le partage avec Zoom ne démarre qu'à votre demande et n'envoie que le texte des sous-titres, jamais l'audio. Les clés API sont stockées dans le Gestionnaire d'informations d'identification Windows. Le développeur ne reçoit ni audio, ni clé, ni transcription, ni télémétrie, et l'application ne vend ni abonnement, ni crédits, ni accès API. Un lien facultatif Buy Me a Coffee permet de soutenir le développement sans débloquer aucune fonction supplémentaire.

Utilisez l'interface en français, anglais ou allemand quelle que soit la langue des sous-titres. La fenêtre de contrôle se pilote au clavier et avec le Narrateur, et respecte la taille du texte et les thèmes de contraste de Windows. Une démo facultative en français et en anglais permet d'essayer l'affichage sans configuration ; elle lit un texte préparé et ne reconnaît pas la parole.

Nécessite Windows 11 et Microsoft Edge WebView2 Runtime, avec paquets natifs x64 et ARM64. Sur x64, Whisper exige un processeur compatible AVX2 ; les moteurs cloud fonctionnent sans. Téléchargez un modèle Whisper avant toute utilisation hors ligne et autorisez l'accès à la source audio choisie. La capture d'une application inclut ses processus enfants et ses notifications, et peut englober plusieurs onglets de navigateur. Le partage avec Zoom exige une réunion dont l'organisateur a activé le sous-titrage manuel et autorisé les jetons d'API tiers.

Zoom est une marque de Zoom Communications, Inc. Live Translation & Subtitles n'est ni affiliée à Zoom ni approuvée par Zoom.
```

### Nouveautés de cette version — 1.16.1

```text
Les sous-titres Gemini arrivent désormais mot pour mot, environ une seconde après une pause de l'orateur. La version 1.16.0 avait mis fin aux répétitions, mais les sous-titres pouvaient encore arriver bien après la parole et omettre une phrase répétée ; cette mise à jour fait passer Gemini à la transcription verbatim, comme l'annonçait la 1.16.0. Un segment que Gemini clôt sans transcription finale est désormais conservé à partir des derniers mots affichés.

Aussi dans la 1.16 : sous-titres en direct OpenAI avec votre clé OpenAI existante ; Continuer avec Whisper hors ligne quand un moteur cloud s'arrête en cours de séance, dans la même session, la même transcription et la même entrée d'historique ; le choix de la langue parlée pour Gemini ; les premiers mots après Démarrer ou Reprendre conservés avec tous les moteurs cloud ; et des corrections pour le filtre d'écho, la fin des phrases DeepL, Whisper en chinois, japonais et thaï, les micros et projecteurs débranchés, et le coût estimé pendant les pauses.
```

### Fonctionnalités — une fonctionnalité par ligne

```text
Sous-titres Whisper hors ligne en 99 langues ou détection auto, sur carte graphique si disponible ; prend le relais en un clic si un moteur cloud s'arrête
Téléchargez Tiny, Base, Small ou Large v3 Turbo une fois, puis transcrivez hors ligne sans compte, clé API ni connexion Internet
Traduction hors ligne vers l'anglais avec Whisper depuis ses 99 langues, sans clé API
Traduction cloud avec votre clé Gemini, OpenAI ou DeepL (DeepL : forfait API payant, expérimental) ; frais fournisseur possibles ; garde-fou de dépenses
Sous-titres cloud avec votre clé Gemini, Mistral ou OpenAI (même clé que la traduction) ; liste de noms et termes pour Gemini, Mistral, OpenAI et Whisper
Deux langues de traduction simultanées et affichage facultatif du texte original quand le moteur le fournit
Captez un microphone, l'audio système, une application précise, ou le micro et l'audio système ensemble
Surimpression transparente, toujours au premier plan et traversable par les clics, sur vos diapositives et appels
Suivez le chat vocal des jeux en ligne dans une autre langue, dans une fenêtre indépendante sans rien injecter dans le jeu
La surimpression mémorise sa position pour chaque configuration d'écrans et se rouvre où vous l'avez laissée
Modes Adapter à la fenêtre, Compact et Lecture stable ; apparence réglable et écritures de droite à gauche
Pause et reprise, annonce de pause sur la surimpression, raccourcis système facultatifs, profils de réunion, recherche de langues et favoris
Filtre d'hésitations modifiable pour la surimpression tout en conservant la transcription brute intacte
Relecture, recherche et repères dans la transcription en direct ; copie des 5, 10 ou 30 dernières minutes ou de toute la séance
Export en texte brut, Markdown, SRT et WebVTT, avec le texte original lorsqu'il est disponible
Historique local facultatif avec titres, recherche, réouverture, export groupé et suppression
Pilotage complet au clavier et avec le Narrateur ; suit la taille du texte et les thèmes de contraste de Windows
Fenêtre de contrôle claire ou sombre, ou calée automatiquement sur le thème de Windows
Interface en français, anglais et allemand ; paquets Windows natifs x64 et ARM64
Envoi d'une langue de sous-titres directement dans la barre de sous-titres d'une réunion Zoom avec le jeton de l'organisateur
```

### Description courte

```text
Sous-titres en direct gratuits sur vos réunions, appels et diapositives. Transcrivez hors ligne avec Whisper en 99 langues, traduisez vers l'anglais en local, ou utilisez votre clé API cloud pour d'autres langues. Captez micro ou app et exportez la transcription.
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
Verfolgen Sie Besprechungen, Vorträge, Videoanrufe und Präsentationen mit Live-Untertiteln direkt dort, wo alle hinschauen. Live Translation & Subtitles transkribiert Sprache lokal auf Ihrem PC mit Whisper: Laden Sie einmal ein Modell herunter und nutzen Sie Untertitel komplett offline, ohne Konto und ohne Abo. Whisper übersetzt außerdem offline ins Englische. Wenn Ihr Publikum eine andere Sprache liest, nutzen Sie einfach Ihren eigenen API-Schlüssel von Google Gemini, OpenAI oder DeepL für Live-Übersetzungen (beim Anbieter können nutzungsabhängige Kosten anfallen; DeepL erfordert einen kostenpflichtigen DeepL-API-Tarif).

Die Untertitel erscheinen in einem transparenten Overlay, das über Folien, Videoanrufen oder jeder anderen Windows-App bleibt und Mausklicks direkt zur darunterliegenden Anwendung durchlässt. Das Overlay merkt sich seine Position für jede Bildschirmanordnung und öffnet sich wieder genau dort. Wählen Sie genau die richtige Audioquelle: ein Raummikrofon, das Windows-Systemaudio, eine einzelne Anwendung oder Mikrofon und Systemaudio gemeinsam. In Zoom-Meetings können Sie zudem eine Untertitelsprache direkt in Zooms eigene Untertitelleiste senden – über den Untertitel-Link des Hosts; Teilnehmende schalten die Untertitel einfach in Zoom ein und müssen nichts installieren. Auch für Sprachchat in Onlinespielen eignet sich das Overlay: Starten Sie das Spiel im Fenstermodus oder randlosen Fenstermodus, damit das eigenständige Untertitelfenster sauber darüber liegt, ohne in das Spiel einzugreifen.

Für Offline-Untertitel wählen Sie ein mehrsprachiges Whisper-Modell und eine von 99 gesprochenen Sprachen oder die automatische Erkennung. Laden Sie Tiny (31 MiB), Base (57 MiB), Small (182 MiB) oder Large v3 Turbo (547 MiB) einmal herunter; danach läuft die Erkennung lokal auf Ihrer Grafikkarte (falls geeignet) oder auf der CPU, ohne Internetverbindung und ohne API-Schlüssel. Die eingebaute Spracherkennung überspringt Stille und Musik, damit Whisper in einem ruhigen Raum keine Untertitel erfindet. Tempo und Genauigkeit hängen von PC, Modell und Sprache ab. Whisper untertitelt in der gesprochenen Sprache oder übersetzt ins Englische; wählen Sie zum Übersetzen Small, da Tiny und Base schwach übersetzen und Large v3 Turbo nur transkribiert. Für andere Zielsprachen wechseln Sie zur Cloud-Übersetzung.

Für ein mehrsprachiges Publikum übersetzen Sie live in 78 Sprachen mit Gemini, 42 mit DeepL oder 13 mit OpenAI. DeepL ist als experimentell gekennzeichnet, da es noch nicht in einer echten Sitzung erprobt wurde; kostenlose DeepL-API-Schlüssel werden abgelehnt. Zeigen Sie zwei Sprachen gleichzeitig an oder blenden Sie den Originaltext unter der Übersetzung ein (in An Fenster anpassen oder Kompakt, sofern die Engine ihn liefert; eine zweite Sprache öffnet eine zusätzliche Cloud-Sitzung pro Audioquelle und erhöht die Anbieternutzung). Gemini, Mistral und OpenAI bieten mit eigenem API-Schlüssel auch Cloud-Untertitel in der gesprochenen Sprache – OpenAI nutzt dafür denselben Schlüssel wie für die Übersetzung. Eine eigene Liste mit Namen und Begriffen hilft Gemini, Mistral, OpenAI und Whisper bei Namen von Vortragenden, Institutionen und Abkürzungen. Stoppt eine Cloud-Engine mitten in einer Sitzung und ist ein Whisper-Modell installiert, wechseln Sie mit einem Klick nahtlos zu Offline-Whisper, ohne die Sitzung zu beenden.

Passen Sie die Darstellung an den Raum an: Schriftart, Größe, Farben, Hintergrund, Anzeigedauer und Aktualisierungstempo in den Modi An Fenster anpassen, Kompakt oder Ruhiger Lesemodus, auch für Rechts-nach-links-Schriften. Ein bearbeitbarer Füllwortfilter blendet Zögerlaute im Overlay aus, während das Originaltranskript unberührt bleibt. Speichern Sie Besprechungsprofile für wiederkehrende Räume, pausieren Sie in Pausen ohne Sitzungsbruch und zeigen Sie im Overlay einen Pausenhinweis wie „Gleich geht es weiter“. Optionale systemweite Tastenkombinationen pausieren Untertitel oder blenden das Overlay ein und aus, während Folien oder Meeting-App im Vordergrund bleiben. Bei Cloud-Engines pausiert eine optionale Kostenbremse ruhige Sitzungen automatisch und warnt, sobald geschätzte Kosten oder Sitzungsdauer Ihr Limit überschreiten.

Während der Sitzung zeigt der Tab Transkript den gesamten Verlauf: Durchsuchen Sie ihn und setzen Sie Marken für Vortragsbeginn, Sprecherwechsel oder Abschnitte, die in Exporten und im Verlauf erhalten bleiben. Kopieren legt die letzten 5, 10 oder 30 Minuten oder das ganze Transkript als reinen Text in die Zwischenablage; für Dateien stehen Text, Markdown, SRT und WebVTT bereit. Aktivieren Sie optional den lokalen Verlauf, um abgeschlossene Untertitel fortlaufend zu sichern und Sitzungen später zu benennen, zu durchsuchen, erneut zu öffnen, zu exportieren oder zu löschen. Verlauf und Wiederherstellungskopie bleiben deaktiviert, bis Sie sie einschalten. Bei Whisper verarbeitet Stopp noch wartendes Audio zu Ende, damit nichts fehlt.

Bei Whisper verlässt kein Audio Ihren PC; wartende Sprache liegt nur in einer automatisch gelöschten temporären Datei. Cloud-Engines streamen Audio direkt an den gewählten Anbieter, und vor dem allerersten Versand an jeden Anbieter fragt die App ausdrücklich um Erlaubnis. Das Teilen mit Zoom startet nur auf Ihren Wunsch und überträgt ausschließlich Untertiteltext, niemals Audio. API-Schlüssel bleiben in der Windows-Anmeldeinformationsverwaltung. Der Entwickler erhält weder Audio noch Schlüssel, Transkripte oder Telemetrie, und die App verkauft keine Abos, Guthaben oder API-Zugänge. Ein freiwilliger Buy-Me-a-Coffee-Link unterstützt die Entwicklung, schaltet aber keine Zusatzfunktionen frei.

Nutzen Sie die Oberfläche auf Deutsch, Englisch oder Französisch, unabhängig von der Untertitelsprache. Das Bedienfenster funktioniert vollständig per Tastatur und mit der Sprachausgabe und folgt der Windows-Textgröße sowie Kontrastdesigns. Eine optionale Demo auf Englisch und Französisch zeigt das Overlay ohne Einrichtung; sie spielt einen vorbereiteten Text ab und erkennt keine Sprache.

Erfordert Windows 11 und die Microsoft Edge WebView2-Runtime, mit nativen Paketen für x64 und ARM64. Auf x64 benötigt Whisper einen Prozessor mit AVX2; die Cloud-Engines laufen auch ohne. Laden Sie vor der Offline-Nutzung ein Whisper-Modell herunter und erlauben Sie den Zugriff auf Ihre gewählte Audioquelle. Die Aufnahme einer einzelnen Anwendung umfasst deren Unterprozesse und Benachrichtigungen und kann mehrere Browser-Tabs einschließen. Für das Teilen mit Zoom muss der Host manuelle Untertitel aktivieren und API-Token für Drittanbieter erlauben.

Zoom ist eine Marke von Zoom Communications, Inc. Live Translation & Subtitles ist nicht mit Zoom verbunden und wird nicht von Zoom unterstützt.
```

### Neu in dieser Version — 1.16.1

```text
Gemini-Untertitel erscheinen jetzt wortgetreu, etwa eine Sekunde nach einer Sprechpause. Version 1.16.0 hatte die Wiederholungen beendet, doch die Untertitel konnten dem Gesagten noch deutlich hinterherhinken und einen wiederholten Satz auslassen; dieses Update stellt Gemini auf wortgetreue Transkription um, wie in 1.16.0 angekündigt. Ein Abschnitt, den Gemini ohne endgültige Transkription schließt, bleibt jetzt mit den zuletzt angezeigten Wörtern erhalten.

Außerdem in 1.16: OpenAI-Live-Untertitel mit Ihrem vorhandenen OpenAI-Schlüssel; „Mit Offline-Whisper fortsetzen“, wenn eine Cloud-Engine mitten in einer Sitzung stoppt, in derselben Sitzung, demselben Transkript und Verlaufseintrag; die Wahl der gesprochenen Sprache für Gemini; die ersten Wörter nach Start oder Fortsetzen bleiben bei allen Cloud-Engines erhalten; und Korrekturen am Echofilter, an DeepL-Satzenden, an Whisper auf Chinesisch, Japanisch und Thai, bei ausgesteckten Mikrofonen und Projektoren und an der Kostenschätzung während Pausen.
```

### Funktionen — eine Funktion pro Zeile

```text
Offline-Untertitel mit Whisper in 99 Sprachen oder automatischer Erkennung, per GPU falls verfügbar; übernimmt mit einem Klick, wenn eine Cloud-Engine stoppt
Tiny, Base, Small oder Large v3 Turbo einmal laden und danach offline ohne Konto, API-Schlüssel oder Internet transkribieren
Offline-Übersetzung ins Englische mit Whisper aus allen 99 Sprachen, ganz ohne API-Schlüssel
Cloud-Übersetzung mit eigenem Gemini-, OpenAI- oder DeepL-Schlüssel (DeepL: kostenpflichtiger API-Tarif, experimentell); Kosten möglich; optionale Kostenbremse
Cloud-Untertitel mit eigenem Gemini-, Mistral- oder OpenAI-Schlüssel (wie Übersetzung); Liste mit Namen und Begriffen für Gemini, Mistral, OpenAI und Whisper
Zwei Übersetzungssprachen gleichzeitig und optional den Originaltext darunter, sofern die Engine ihn liefert
Mikrofon, Systemaudio, eine einzelne Anwendung oder Mikrofon und Systemaudio gemeinsam erfassen
Transparentes, durchklickbares Untertitel-Overlay immer im Vordergrund über Folien, Videoanrufen und jeder Windows-App
Sprachchat in Onlinespielen in einer anderen Sprache verfolgen, in einem eigenen Fenster ohne Eingriff ins Spiel
Das Overlay merkt sich seine Position für jede Bildschirmanordnung und öffnet sich wieder genau dort
Modi An Fenster anpassen, Kompakt und Ruhiger Lesemodus; anpassbare Darstellung und Rechts-nach-links-Schriften
Pause und Fortsetzen, Pausenhinweis im Overlay, optionale systemweite Tasten, Besprechungsprofile, Sprachsuche und Favoriten
Bearbeitbarer Füllwortfilter für das Overlay, während das Originaltranskript unverändert bleibt
Transkript live lesen, durchsuchen und mit Marken versehen; die letzten 5, 10 oder 30 Minuten oder alles kopieren
Export als reiner Text, Markdown, SRT und WebVTT, auf Wunsch mit Originaltext
Optionaler lokaler Transkriptverlauf mit Titeln, Suche, erneutem Öffnen, Sammelexport und Löschen
Vollständige Tastaturbedienung und Sprachausgabe; folgt Windows-Textgröße und Kontrastdesigns
Bedienfenster hell oder dunkel oder automatisch passend zum Windows-App-Modus
Oberfläche auf Deutsch, Englisch und Französisch; native Windows-Pakete für x64 und ARM64
Eine Untertitelsprache direkt in der Untertitelleiste eines Zoom-Meetings teilen, über den API-Link des Hosts
```

### Kurzbeschreibung

```text
Kostenlose Live-Untertitel über Meetings, Anrufen und Folien. Mit Whisper offline in 99 Sprachen transkribieren oder ins Englische übersetzen, andere Sprachen per Cloud mit eigenem API-Schlüssel. Mikrofon oder App erfassen, zwei Sprachen anzeigen und exportieren.
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

The interface supports German. Before publishing a German listing, review the German copy,
including the sentences new in 1.13.0, 1.13.1, 1.14.0, 1.15.0 and 1.16.0.
The German screenshots are captured from the 1.7.1 MSIX with the interface set to German; see
the [screenshot notes](store-screenshots/README.md).

## Additional system requirements

Enter each line as a separate requirement (under 200 characters):

```text
Windows 11 with Microsoft Edge WebView2 Runtime; native x64 and ARM64 packages
Whisper: internet for the initial model download (31–547 MiB), then offline transcription or translation into English on the processor or a suitable graphics processor
Whisper on x64 needs a processor with AVX2, which some budget Celeron and Pentium Silver PCs lack
Cloud engines: internet, your own compatible provider account and API key; usage charges may apply
Audio-source access required for speech recognition; a microphone is needed only for microphone capture
```

Recommended memory: 8 GB; Large v3 Turbo takes about 1 GB more while it runs. Whisper speed
depends on the processor, the graphics processor and the model; this is not a guarantee of
real-time processing on all supported hardware. Localized requirements:

### Configuration supplémentaire — français

```text
Windows 11 et Microsoft Edge WebView2 Runtime ; paquets x64 et ARM64 natifs
Whisper : Internet pour télécharger le modèle (31–547 Mio), puis transcription ou traduction vers l'anglais hors ligne sur le processeur ou un processeur graphique adapté
Whisper sur x64 nécessite un processeur compatible AVX2, absent de certains PC Celeron et Pentium Silver d'entrée de gamme
Moteurs cloud : Internet, votre compte et clé API compatibles ; des frais d'utilisation peuvent s'appliquer
Accès à la source audio pour la reconnaissance ; microphone nécessaire uniquement pour la capture du micro
```

### Zusätzliche Systemanforderungen — deutsch

```text
Windows 11 mit Microsoft Edge WebView2-Runtime; native Pakete für x64 und ARM64
Whisper: Internet für den Modelldownload (31–547 MiB), danach Offline-Transkription oder Übersetzung ins Englische auf der CPU oder einem geeigneten Grafikprozessor
Whisper auf x64 erfordert einen Prozessor mit AVX2; manche günstigen Celeron- und Pentium-Silver-PCs haben keinen
Cloud-Engines: Internet, eigenes kompatibles Konto und API-Schlüssel; nutzungsabhängige Kosten möglich
Zugriff auf die Audioquelle für Spracherkennung; ein Mikrofon ist nur für Mikrofonaufnahme erforderlich
```

## Notes for certification

```text
Product: Live Translation & Subtitles. Product ID: 9PFB8LR3RR9X. Version: 1.16.1.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Since 1.16.0, this update changes one thing: Gemini subtitles (the user's own Gemini key) ask Google's Gemini Transcribe for its VERBATIM transcription instead of SMART, so captions arrive word for word about a second after a pause, and a segment the service closes without a final transcript keeps its last interim text. What is sent (audio, the optional names and terms and spoken language) and to whom is unchanged, and the privacy policy is unchanged. 1.16.0 added OpenAI subtitles (OpenAI's gpt-live-transcribe on the user's own OpenAI key, covered by the same one-time question before audio first goes to OpenAI) and Continue with offline Whisper: when a cloud engine ends a source after a key, quota or billing error, or has been reconnecting for 30 seconds, a banner offers to continue the same session with an installed Whisper model; it never switches on its own and sends nothing new anywhere. It adds no capability and no product declaration. The test routes below are unchanged since 1.15.0.

Optional Captions in Zoom (off until the user starts it): the Zoom captions button in the operator window's toolbar opens a dialog where the user pastes the caption link (API token) that a Zoom meeting's host copies with Show Captions > Set up manual captioner > Copy the API token, and chooses Start sharing. From then until Stop sharing or the app quits, the app posts finished caption text in one caption language over HTTPS directly to that zoom.us or zoomgov.com address, and Zoom shows it in the meeting's caption bar. This includes captions made offline with Whisper or by the built-in demo; no audio, API key or device information is sent, and nothing passes through a developer server. The dialog says that caption text is sent to Zoom above its Start sharing button, and the privacy policy describes the flow. The link is held in memory only and never saved. To exercise it, host a Zoom meeting (a free Basic account is enough) on an account whose settings have Manual captions on and the third-party caption API token allowed, copy the API token in the meeting, paste it, start sharing, choose Send test caption, then run the built-in demo or Whisper: attendees see the captions after choosing Show Captions. Without a Zoom meeting, open the dialog and paste any https address outside Zoom: it is refused locally with an explanation and nothing is sent. The rest of the app works the same whether or not Zoom is used. Zoom is named descriptively only; the app uses no Zoom logo and is not affiliated with Zoom Communications, Inc.

Voluntary support link (Store Policy 10.8.2): the Support development section at the end of Settings › App, and the credit line at the foot of the start screen while no session runs, open https://buymeacoffee.com/fmadore in the user's default browser. Buy Me a Coffee, a third-party service, identifies itself, authenticates the supporter and takes any payment on its own site. The app has no in-app purchase and collects no payment or financial information. A donation gives nothing in return: every feature stays free, and nothing is unlocked, removed or changed. The link is never shown during a session and the app never prompts for support. The Product declaration "This product allows users to make purchases, but does not use the Microsoft Store commerce system" is ticked for this link.

Real speech recognition without a paid account or API key:
1. A clean install opens on Subtitles / Whisper / Base / Detect automatically, with the default microphone selected. No download or capture starts automatically. On an x64 processor without AVX2, Whisper is listed as unavailable with the reason and a clean install opens on Built-in demo instead; use the setup-free check below. Choose the microphone or system audio source. Grant microphone access if using a microphone. Existing saved setups are preserved on upgrade.
2. Choose the spoken language or Detect automatically. Download Tiny (about 31 MiB) for the lightest setup, or Base (about 57 MiB). The initial HTTPS model download needs internet access; the model is not bundled. No account or key is needed.
3. Start captioning and speak a few sentences or play speech on the selected output. Allow for processing time; on a PC with a suitable graphics processor, captions arrive sooner. The Use the graphics processor switch under the model turns this off. Verify captions in the operator window and overlay, elapsed time and input levels. Then select the Transcript tab to read the session so far, choose Last 5 minutes beside Copy, click Copy and paste into Notepad: the captions arrive as plain text.
4. Use Move overlay to place it, lock it back to click-through, and try Fit window, Compact and Stable reading. After a restart, the overlay reopens where it was placed. Pause blocks new speech entering recognition; existing pending audio can still finish.
5. Stop and let Finishing the transcript complete. Export text or Markdown, then SRT or WebVTT. Disconnect the network and repeat with the installed model to verify offline operation.
6. In Settings, optionally enable transcript history; run a disposable session and reopen, rename, export and delete it. History and recovery are disabled by default. Test the editable filler-word list: it changes only the overlay.
7. Optional offline translation: choose Translation, then Local Whisper, keep English as the caption language, choose the spoken language or Detect automatically, and download Small (about 182 MiB). Speak or play non-English speech, or click Rehearse to play a bundled French recording with no microphone: English captions appear with no account or key. Tiny and Base also work but translate poorly.

Setup-free display check if capture hardware or model downloads are unavailable:
In Subtitles, select Built-in demo in the engine list, choose English and click Start demo subtitles. The bundled English/French script exercises status, meter animation, timer, overlay, Stop and export without devices, network or credentials. Repeat with French. It is explicitly a scripted demo and does not recognize speech; Whisper above is the first-launch local speech engine.

Optional cloud features:
Gemini/OpenAI/DeepL translation and Gemini/Mistral/OpenAI subtitles require the user's compatible third-party account and API key and may incur provider charges. The app supplies no cloud account or credits. Keys are entered in the app and kept in Windows Credential Manager. Cloud testing requires suitable test credentials to be arranged separately before submission. Never place a personal key in public listing fields or screenshots. Choose two supported translation targets to check bilingual output; this creates a second provider session per audio source. Original speech is shown only when the provider supplies it. Mistral subtitles now use Voxtral Realtime 3 (voxtral-mini-transcribe-realtime-3-0), which Mistral lists as a public preview. DeepL translation (DeepL Voice API, by DeepL SE), which the app marks as experimental because it has not yet run in a live session, needs a key from a paid DeepL API plan: the Voice API is not on DeepL's free API. Without one, saving a free DeepL API key (one ending in :fx) for DeepL shows the app's refusal, with its reason, and no request is made. Before the first session or rehearsal with Gemini, OpenAI, Mistral or DeepL, the app asks once for that provider: it names the provider, says that the captured audio goes to it under the user's own account, key and terms, and gives the privacy policy's address. Choose Send to (the provider's name) to continue; Cancel, which has the focus, starts nothing. Both Gemini engines share one answer, and so do both OpenAI engines. Entries in the optional Names and terms list, shown in the setup for Mistral, Gemini subtitles, OpenAI subtitles and Whisper, are sent to Mistral, Gemini or OpenAI with the session; Whisper keeps them on the PC, and the translation engines are not sent them.

The full-trust desktop process performs WASAPI/microphone capture, Whisper transcription and translation (on the processor, or on the graphics processor through the bundled whisper-vulkan.exe helper it starts), speech detection with a bundled model, user-requested verified model downloads, Credential Manager access, window/overlay management and explicit transcript export. Whisper buffers pending audio in an automatically deleted local temporary file. Cloud engines send audio directly to the selected provider. Captions in Zoom, when the user starts it, sends caption text directly to the Zoom address the user pasted. There is no developer relay, telemetry, driver, service, auto-start task, Windows AI component or required speech language pack.
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
