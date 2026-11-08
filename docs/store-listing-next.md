# Next Store submission — 1.12.0

**Tagged 27 October 2026; not yet submitted.** The Store serves **1.11.0.0**, accepted as
reported by the maintainer on 26 October 2026. The next submission is **1.12.0.0**: one
unsigned bundle with native x64 and ARM64 packages, at
`https://github.com/fmadore/Live-translation/releases/download/v1.12.0/Live.Translation_1.12.0.msixbundle`.

The positioning is unchanged. Lead the listing with offline Whisper captions, the transparent
overlay and optional cloud translation; the scripted demo is a secondary way to try the display
controls. New installations open on Whisper / Base / automatic detection, and existing saved
setups are preserved. Model download and capture remain explicit user actions.

1.12.0 adds **Captions in Zoom**
([#189](https://github.com/fmadore/Live-translation/pull/189), part of
[#187](https://github.com/fmadore/Live-translation/issues/187)): the operator pastes the caption
link (API token) the meeting's host copied in Zoom, and the app posts one caption language
directly to Zoom's caption bar. It also fixes flashes and layout shifts in setup and the
transcript ([#188](https://github.com/fmadore/Live-translation/pull/188)) and says why Start
waits for an application. It is the first version that can send **user content to a third
party other than the speech provider the user chose**, including captions made offline with
Whisper, and the first whose listing names another company's product. So it needs things the
earlier submissions did not:

- **Privacy policy (Store Policy 10.5, personal information).** Caption text can contain
  personal information, such as the names of people speaking, and sharing sends it to Zoom. The
  published [privacy policy](privacy.md) must describe that flow: its *Optional captions in Zoom*
  section and the Zoom line under *Third-party processing* do, with the effective date
  27 October 2026 for 1.12.0. Sharing is opt-in by construction: nothing is sent until the user
  pastes a link and chooses **Start sharing**, and the dialog says, before that button, that
  caption text goes to Zoom even when Whisper makes it. Check that the published page shows the
  new date and the Zoom section before submitting.
- **Notes for certification.** The new second paragraph in [store-listing.md](store-listing.md)
  says what the feature sends and where, that it is off until the user starts it, and how a
  tester can exercise it (a Zoom meeting they host, with manual captioning and the third-party
  caption API token allowed) or skip it (opening the dialog and pasting a non-Zoom address,
  which the app refuses without sending anything). A free Basic Zoom account is enough (confirmed
  by the maintainer on 27 October 2026), but the certification team cannot be expected to have
  one; the credential-free Whisper and demo routes still cover the rest of the app.
- **Trademark (Policies 10.1 and 11.2).** Zoom is named only descriptively, to say what the
  feature works with; the app shows no Zoom logo (its toolbar mark is a generic captions
  symbol, deliberately, because Zoom's logo needs Zoom's written licence), and neither the
  screenshots nor the logo use Zoom's artwork. Each description ends with "Zoom is a trademark of
  Zoom Communications, Inc. Live Translation & Subtitles is not affiliated with or endorsed by
  Zoom." in its language, so the line sits wherever the listing names Zoom. Zoom is not added
  to the keywords.
- **Descriptions, features and What's new.** One sentence per description on Zoom captions,
  one on the caption text it sends in the privacy paragraph, a twentieth feature line (the most
  Partner Center allows), and a new What's new in all three languages.

No product declaration changes: the purchases-outside-Microsoft-commerce box ticked for 1.11.0
stays ticked, and Zoom captions involve no payment. The MSIX capabilities are unchanged
(`runFullTrust` and `microphone`).

- [EN/FR/DE descriptions, What's new and certification notes](store-listing.md)
- [Release notes](release-1.12.0.md)
- [Build evidence and acceptance checklist](store-updates.md#release-1120-handoff)
- [Captions in Zoom guide](zoom-captions.md), which the app's dialog opens
- [Screenshot plan](store-screenshots/README.md)
- [Partner Center walkthrough](partner-center-walkthrough.md)

Before submission, complete:
- the maintainer's test of the Local Test build of the release merge: Captions in Zoom already
  passed in a real meeting on a free account (27 October 2026), so this is a final pass over
  the merge, including the *Show captions on this screen too* switch, the pre-flight asking for
  an application, and the setup and transcript no longer jumping;
- the final-package checks, and the checks still open from earlier releases;
- the screenshots: the toolbar gains a *Zoom captions* button, which the current images (from
  1.7.1) do not show; decide whether a retake is wanted, and if one is taken, keep any Zoom
  window and Zoom's artwork out of it;
- the German copy review, now including the Zoom sentences;
- Windows App Certification Kit.

Use the normal Store identity: the side-by-side Local Test installation is for feedback, not the
Store submission package.

## The version after 1.12.0

**Draft, not yet tagged.** Whisper on the graphics processor, bundled speech detection, loop and
credit filters and Large v3 Turbo
([local Whisper notes](local-whisper.md#graphics-processor)). Apply these once 1.12.0 has been
submitted and [store-listing.md](store-listing.md) is rolled forward to the new version; until
then that file stays the 1.12.0 copy. Every block below replaces the named paragraph or line
in [store-listing.md](store-listing.md) word for word; nothing else in the listing changes (short
descriptions, keywords, screenshots, trademark line). The French and German sentences need the
maintainer's review, as each release's do.

What the submission needs beyond the copy:

- **A second executable in the package.** `whisper-vulkan.exe` sits beside the app's executable.
  It is part of the same full-trust app, started and stopped only by it, opens no network
  connection, and needs no new capability: it reaches the graphics processor through the
  graphics driver already installed. The certification notes say so (below).
- **A bundled model.** `models/ggml-silero-v5.1.2.bin` (864 KiB, Silero VAD, MIT) ships in the
  package. The Whisper speech models are still downloaded only at the user's request.
- **No privacy-policy change in substance.** Audio still never leaves the PC with Whisper; the
  [privacy policy](privacy.md) gained a sentence naming the helper, without a new effective
  date, since nothing new is sent or kept.
- **Screenshots** need no retake: the setup gains one switch under the model, *Use the graphics
  processor*.

### Description: the offline-subtitles paragraph (third paragraph)

English:

```text
For offline subtitles, pick a multilingual Whisper model and one of 99 spoken languages, or let the app detect the language. The Tiny, Base, Small and Large v3 Turbo models download separately, at about 31, 57, 182 and 547 MiB. Once a model is installed, recognition runs on your PC with no account, API key or internet connection: on its graphics processor when it has a suitable one, otherwise on its processor. Built-in speech detection skips silence and music, so Whisper does not invent captions over a quiet room. Speed and accuracy depend on the computer, the model and the language. Whisper captions in the language being spoken, or translates it into English; for translation, choose Small, as Tiny and Base translate poorly and Large v3 Turbo only transcribes. For any other language, use cloud translation.
```

Français :

```text
Pour les sous-titres hors ligne, choisissez un modèle Whisper multilingue et l'une des 99 langues parlées, ou laissez l'application détecter la langue. Les modèles Tiny, Base, Small et Large v3 Turbo se téléchargent séparément et pèsent environ 31, 57, 182 et 547 Mio. Une fois le modèle installé, la reconnaissance s'effectue sur votre PC, sans compte, clé API ni connexion Internet : sur son processeur graphique s'il en a un qui convient, sinon sur son processeur. La détection de la parole intégrée ignore les silences et la musique, pour que Whisper n'invente pas de sous-titres dans une salle silencieuse. La vitesse et la précision dépendent de l'ordinateur, du modèle et de la langue. Whisper sous-titre dans la langue parlée ou traduit la parole vers l'anglais ; pour traduire, choisissez Small, car Tiny et Base traduisent mal et Large v3 Turbo ne fait que transcrire. Pour toute autre langue, utilisez la traduction cloud.
```

Deutsch:

```text
Für Offline-Untertitel wählen Sie ein mehrsprachiges Whisper-Modell und eine von 99 gesprochenen Sprachen oder die automatische Erkennung. Tiny, Base, Small und Large v3 Turbo werden separat heruntergeladen (etwa 31, 57, 182 und 547 MiB). Danach läuft die Erkennung auf Ihrem PC, ohne Konto, API-Schlüssel oder Internetverbindung: auf dem Grafikprozessor, wenn ein geeigneter vorhanden ist, sonst auf der CPU. Die eingebaute Spracherkennung überspringt Stille und Musik, damit Whisper in einem stillen Raum keine Untertitel erfindet. Geschwindigkeit und Genauigkeit hängen von Computer, Modell und Sprache ab. Whisper transkribiert in der gesprochenen Sprache oder übersetzt ins Englische; zum Übersetzen wählen Sie Small, da Tiny und Base schlecht übersetzen und Large v3 Turbo nur transkribiert. Für andere Sprachen nutzen Sie die Cloud-Übersetzung.
```

### What's new in this version

English:

```text
Faster offline captions: Whisper now runs on your PC's graphics processor when it has a suitable one, and returns to the processor by itself if anything goes wrong; a switch under the model turns it off. Built-in speech detection skips silence and music, so a quiet room no longer produces invented captions, and words repeated over and over or stray subtitle credits are removed.

A new model, Large v3 Turbo (547 MiB), gives the most accurate offline subtitles. It transcribes only: keep Small for translation into English.
```

Français :

```text
Des sous-titres hors ligne plus rapides : Whisper s'exécute désormais sur le processeur graphique de votre PC s'il en a un qui convient, et revient de lui-même au processeur en cas de problème ; un interrupteur sous le modèle le désactive. La détection de la parole intégrée ignore les silences et la musique : une salle silencieuse ne produit plus de sous-titres inventés, et les mots répétés en boucle ou les crédits de sous-titrage parasites sont retirés.

Un nouveau modèle, Large v3 Turbo (547 Mio), donne les sous-titres hors ligne les plus précis. Il ne fait que transcrire : gardez Small pour traduire vers l'anglais.
```

Deutsch:

```text
Schnellere Offline-Untertitel: Whisper läuft jetzt auf dem Grafikprozessor Ihres PCs, wenn ein geeigneter vorhanden ist, und wechselt bei Problemen von selbst zurück zur CPU; ein Schalter unter dem Modell schaltet das aus. Die eingebaute Spracherkennung überspringt Stille und Musik: Ein stiller Raum erzeugt keine erfundenen Untertitel mehr, und endlos wiederholte Wörter oder eingestreute Untertitel-Abspanne werden entfernt.

Ein neues Modell, Large v3 Turbo (547 MiB), liefert die genauesten Offline-Untertitel. Es transkribiert nur: Für die Übersetzung ins Englische bleibt Small die Wahl.
```

### Features: the first two lines

English:

```text
Offline Whisper captions in 99 spoken languages or automatic detection, on the graphics processor when available
Download Tiny, Base, Small or Large v3 Turbo once, then transcribe without an account, API key or internet
```

Français :

```text
Sous-titres Whisper hors ligne dans 99 langues parlées ou détection automatique, sur le processeur graphique s'il convient
Téléchargez Tiny, Base, Small ou Large v3 Turbo, puis transcrivez sans compte, clé API ni connexion Internet
```

Deutsch:

```text
Offline-Untertitel mit Whisper: 99 gesprochene Sprachen oder automatische Erkennung, auf dem Grafikprozessor, wenn verfügbar
Tiny, Base, Small oder Large v3 Turbo herunterladen und danach ohne Konto, API-Schlüssel oder Internet transkribieren
```

### Additional system requirements: the second line, and the note under the list

```text
Whisper: internet for the initial model download (31–547 MiB), then offline transcription or translation into English on the processor or a suitable graphics processor
```

Recommended memory: 8 GB; Large v3 Turbo takes about 1 GB more while it runs. Whisper speed
depends on the processor, the graphics processor and the model; this is not a guarantee of
real-time processing on all supported hardware.

```text
Whisper : Internet pour télécharger le modèle (31–547 Mio), puis transcription ou traduction vers l'anglais hors ligne sur le processeur ou un processeur graphique adapté
```

```text
Whisper: Internet für den Modelldownload (31–547 MiB), danach Offline-Transkription oder Übersetzung ins Englische auf der CPU oder einem geeigneten Grafikprozessor
```

### Notes for certification

The first paragraph, with the version filled in:

```text
Product: Live Translation & Subtitles. Product ID: 9PFB8LR3RR9X. Version: <version>.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Since 1.12.0, this update runs offline Whisper on a suitable graphics processor through whisper-vulkan.exe, a helper program in the package that the app alone starts and stops, which opens no network connection and uses the graphics driver already installed; on a PC without a suitable graphics processor, or if the helper fails, Whisper runs on the processor as before. It also adds speech detection, with a 0.9 MB voice-activity model bundled in the package, and an optional Large v3 Turbo model download (about 547 MiB, transcription only). It adds no capability and no product declaration. The test routes below are those of 1.12.0.
```

In step 3 of *Real speech recognition*, replace "Allow for CPU processing time." with:

```text
Allow for processing time; on a PC with a suitable graphics processor, captions arrive sooner. The Use the graphics processor switch under the model turns this off.
```

The last paragraph:

```text
The full-trust desktop process performs WASAPI/microphone capture, Whisper transcription and translation (on the processor, or on the graphics processor through the bundled whisper-vulkan.exe helper it starts), speech detection with a bundled model, user-requested verified model downloads, Credential Manager access, window/overlay management and explicit transcript export. Whisper buffers pending audio in an automatically deleted local temporary file. Cloud engines send audio directly to the selected provider. Captions in Zoom, when the user starts it, sends caption text directly to the Zoom address the user pasted. There is no developer relay, telemetry, driver, service, auto-start task, Windows AI component or required speech language pack.
```
