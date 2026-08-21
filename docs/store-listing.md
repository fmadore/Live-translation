# Microsoft Store listing material

Everything Phase E of [`microsoft-store.md`](microsoft-store.md) needs typed into Partner
Center: the two store descriptions, the certification notes, guidance for the IARC
questionnaire, and the screenshot plan. Store ID `9PFB8LR3RR9X`; the listing goes live at
<https://apps.microsoft.com/detail/9PFB8LR3RR9X> once the first submission passes. The
screen-by-screen sequence, and the shorter fields, are in
[`partner-center-walkthrough.md`](partner-center-walkthrough.md).

Blocks in fenced `text` boxes are ready to paste as-is. Partner Center descriptions are
**plain text** and do not render Markdown, so those blocks carry no formatting characters:
line breaks and hyphen bullets only. The description field allows 10,000 characters; both
languages below sit around 4,000.

**Each paragraph in those blocks is one long unwrapped line, deliberately.** Partner Center
preserves the line breaks you paste, so a paragraph hard-wrapped for readability here would
reach the product page broken into ragged short lines. Copy a whole block, breaks included,
and change nothing. For the same reason the price rows are hyphen bullets rather than a
space-aligned table: the Store renders the description in a proportional font, where column
alignment made of spaces falls apart.

The disclosure in the opening lines is deliberate and satisfies policy 10.2.4, which requires
a dependency on non-integrated software to be disclosed *at the beginning of the description*.
It is worded to be accurate now that the on-device engine ships: captioning itself needs no
key, and the paid dependency applies to translation and to the higher-accuracy subtitle
engines. Do not soften it further, and do not move it below the feature list.

---

## 1. Store description (EN)

```text
Live Translation & Subtitles puts real-time captions on screen for bilingual meetings, lectures and conference sessions. It captions out of the box, free, with no account and no API key: the bundled on-device speech recognizer runs entirely on your own computer and never sends audio anywhere. Live translation between English and French, and higher-accuracy subtitles, are optional upgrades that require your own API key from Google Gemini, OpenAI or Mistral. Those are paid third-party services. They bill your own account with that provider per minute of streamed audio, roughly $0.36 to $3.06 per hour at the list prices below. This app sells nothing: no keys, no credits, no subscription, no in-app purchase.

WHAT IT DOES

- Live subtitles with no key. The bundled on-device recognizer (whisper) turns speech into same-language captions using only your PC. No network connection, no credential, nothing billed per minute. This is the default on a fresh install.
- Live translation, English to French and French to English. Google Gemini or OpenAI, with your own key. The spoken language is detected automatically and the caption appears in the target language you chose. Gemini also captions speech that is already in that language, so a mixed-language room does not go blank during same-language passages.
- Higher-accuracy subtitles. Mistral Voxtral realtime transcription, with your own key, when the on-device engine is not accurate enough for the room.
- A transparent, always-on-top caption overlay. It is click-through, so it sits over a slide deck or a video call without intercepting a single click. Move mode lets you drag, nudge and resize it on the projector, then locks it back into place. Captions can be hidden mid-session for a video clip or a break without stopping anything.
- Capture from the room microphone, from Windows system audio (whatever is playing: Teams, Zoom, a browser tab, a media player), or from both at once, captioned side by side.
- Transcript export to plain text or Markdown, written to your Documents folder only when you ask for it.
- A running cost meter. Streamed minutes and an estimate in dollars for the engine you picked, visible while the session runs, so nothing arrives as a surprise on a provider bill.
- Rehearsal mode. A short bundled French and English speech sample plays through the real pipeline, so you can test the overlay, its placement and the transcript export before an event without a microphone, a key or a network.

WHAT IT COSTS TO RUN

The on-device engine is free. The cloud engines are billed by the provider that issued your key, per minute of audio streamed, for as long as a session is open. Per hour, per audio source:

- Subtitles, on-device recognizer (whisper, bundled): free
- Subtitles, Mistral voxtral-mini-transcribe-realtime: $0.36 per hour
- Translation, Google gemini-3.5-live-translate-preview: $1.25 to $2.21 per hour
- Translation, OpenAI gpt-realtime-translate: $3.06 per hour

Gemini is a range because its expensive output leg accrues only while the model is generating, so pauses and slide changes lower the bill. The OpenAI figure includes the separate source transcription that feeds the operator's monitor. Captioning the room microphone and system audio at the same time doubles these figures, because one stream is opened per source.

These are provider list prices, verified on 10 August 2026, and they are set by the provider, not by this app. They change. Check Google, OpenAI and Mistral pricing pages before budgeting a long event.

SYSTEM REQUIREMENTS

- Windows 10 or Windows 11, 64-bit. ARM64 devices install and run the x64 package under Windows emulation; only the on-device engine is slower there.
- Microsoft Edge WebView2 Runtime, which is already present on Windows 11 and on any Windows 10 machine kept current through Edge.
- Microphone access allowed for this app under Settings, Privacy & security, Microphone, if you want to caption a room microphone. Capturing system audio needs no permission.
- An internet connection only for the cloud engines. The on-device engine works offline.

PRIVACY

No account, no sign-in, no telemetry, no analytics, no advertising. The developer runs no server and receives nothing. Audio is captured only while a session is running, is never written to disk, and leaves your computer only when you have selected a cloud engine, in which case it goes to that provider and to nobody else. API keys are stored in Windows Credential Manager and are sent only to the provider that issued them. Transcripts are saved only when you click Save.

Full privacy policy: https://fmadore.github.io/Live-translation/privacy

Open source under the MIT licence: https://github.com/fmadore/Live-translation
```

---

## 2. Store description (FR)

```text
Live Translation & Subtitles affiche des sous-titres en temps réel pour les réunions, les cours et les sessions de conférence bilingues. L'application sous-titre dès l'installation, gratuitement, sans compte et sans clé d'API : le moteur de reconnaissance vocale embarqué fonctionne entièrement sur votre ordinateur et n'envoie aucun son à l'extérieur. La traduction en direct entre le français et l'anglais, ainsi que des sous-titres plus précis, sont des options qui exigent votre propre clé d'API Google Gemini, OpenAI ou Mistral. Ces services tiers sont payants : ils facturent votre propre compte à la minute d'audio transmis, soit environ 0,36 à 3,06 USD l'heure aux tarifs publics repris ci-dessous. L'application ne vend rien : ni clé, ni crédit, ni abonnement, ni achat intégré.

CE QUE FAIT L'APPLICATION

- Des sous-titres en direct sans aucune clé. Le moteur embarqué (whisper) transforme la parole en sous-titres dans la même langue en n'utilisant que votre PC : aucune connexion, aucun identifiant, rien de facturé à la minute. C'est le réglage par défaut à la première ouverture.
- La traduction en direct, du français vers l'anglais et de l'anglais vers le français. Google Gemini ou OpenAI, avec votre propre clé. La langue parlée est détectée automatiquement et le sous-titre s'affiche dans la langue que vous avez choisie. Gemini sous-titre également les passages déjà prononcés dans cette langue, de sorte qu'une salle bilingue ne se retrouve jamais sans texte.
- Des sous-titres plus précis. La transcription temps réel Mistral Voxtral, avec votre propre clé, lorsque le moteur embarqué ne suffit pas à l'acoustique de la salle.
- Une incrustation transparente, toujours au premier plan. Elle laisse passer les clics et se superpose donc à une présentation ou à une visioconférence sans jamais en intercepter un seul. Un mode déplacement permet de la positionner et de la redimensionner sur le vidéoprojecteur, puis de la verrouiller. Les sous-titres peuvent être masqués en cours de session, le temps d'un extrait vidéo ou d'une pause, sans rien interrompre.
- La capture du micro de la salle, du son système de Windows (ce qui joue sur la machine : Teams, Zoom, un onglet du navigateur, un lecteur multimédia), ou des deux à la fois, sous-titrés côte à côte.
- L'export de la transcription en texte brut ou en Markdown, écrit dans votre dossier Documents uniquement lorsque vous le demandez.
- Un compteur de coût. Les minutes transmises et une estimation en dollars pour le moteur choisi restent visibles pendant la session : aucune mauvaise surprise sur la facture du fournisseur.
- Un mode répétition. Un court extrait sonore français et anglais fourni avec l'application traverse toute la chaîne de traitement : vous pouvez donc essayer l'incrustation, son positionnement et l'export avant un événement, sans micro, sans clé et sans réseau.

CE QUE COÛTE UNE SESSION

Le moteur embarqué est gratuit. Les moteurs en ligne sont facturés par le fournisseur qui a émis votre clé, à la minute d'audio transmis, tant que la session reste ouverte. Par heure et par source audio :

- Sous-titres, moteur embarqué (whisper, fourni) : gratuit
- Sous-titres, Mistral voxtral-mini-transcribe-realtime : 0,36 USD l'heure
- Traduction, Google gemini-3.5-live-translate-preview : 1,25 à 2,21 USD l'heure
- Traduction, OpenAI gpt-realtime-translate : 3,06 USD l'heure

Le tarif de Gemini est une fourchette parce que son poste de sortie, le plus cher, n'est facturé que pendant la génération : les silences et les changements de diapositive font baisser la note. Le chiffre d'OpenAI inclut la transcription de la source qui alimente le moniteur de l'opérateur. Sous-titrer en même temps le micro de la salle et le son système double ces montants, puisqu'un flux est ouvert par source.

Ce sont les tarifs publics des fournisseurs, vérifiés le 10 août 2026. Ils sont fixés par eux et non par cette application, et ils évoluent. Consultez les pages tarifaires de Google, d'OpenAI et de Mistral avant de budgéter un événement long.

CONFIGURATION REQUISE

- Windows 10 ou Windows 11, 64 bits. Les machines ARM64 installent et exécutent le paquet x64 par émulation ; seul le moteur embarqué y est plus lent.
- Le runtime Microsoft Edge WebView2, déjà présent sous Windows 11 et sur toute machine Windows 10 tenue à jour par Edge.
- L'accès au micro autorisé pour cette application dans Paramètres, Confidentialité et sécurité, Microphone, si vous souhaitez sous-titrer un micro de salle. La capture du son système ne demande aucune autorisation.
- Une connexion Internet pour les seuls moteurs en ligne. Le moteur embarqué fonctionne hors ligne.

CONFIDENTIALITÉ

Aucun compte, aucune inscription, aucune télémétrie, aucune analyse d'usage, aucune publicité. Le développeur n'exploite aucun serveur et ne reçoit rien. L'audio n'est capté que pendant une session, n'est jamais écrit sur le disque, et ne quitte votre ordinateur que si vous avez choisi un moteur en ligne : il va alors à ce fournisseur et à personne d'autre. Les clés d'API sont conservées dans le Gestionnaire d'identifiants de Windows et ne sont transmises qu'au fournisseur qui les a émises. Les transcriptions ne sont enregistrées que sur votre demande explicite.

Politique de confidentialité complète : https://fmadore.github.io/Live-translation/privacy

Code source ouvert, licence MIT : https://github.com/fmadore/Live-translation
```

---

## 3. Notes for certification

Paste into the *Notes for certification* field. It answers policy 10.3.1 (testability) and
pre-empts policy 10.8.3 (API keys as financial information), per gates 1 and 2 of
[`microsoft-store.md`](microsoft-store.md).

```text
HOW TO SEE THE PRIMARY FUNCTIONALITY WITH ZERO SETUP

There is no account, no sign-in and no demo credential to hand over, because none is needed. Captioning works on a clean machine with no key and no network connection.

1. Install and launch. The operator window opens on its pre-flight screen with the first-run defaults already selected: mode "Live subtitles", source "System audio", engine "On-device". The checklist shows "No key needed / Runs entirely on this machine" and the cost row reads "Free".
2. Click "Rehearse". A short bundled speech sample in French and English is played through the full capture-to-caption pipeline. Captions appear in the operator window and in the transparent overlay. With the on-device engine selected, which is the default described in step 1, this uses no microphone, no API key and no network connection at all.
3. Or, equivalently, play any audio on the machine (a video in a browser tab, a media player, a Teams call) and click "Start subtitles". The app captures Windows system audio through WASAPI loopback, which requires no permission prompt, and captions it locally.

Either path exercises the overlay, its move mode (drag, arrow-key nudge, resize, Enter to lock, Esc to cancel) and the transcript export to text or Markdown.

Please allow a few seconds for the first caption. The on-device recognizer is whisper.cpp running on the CPU; on a virtual machine, or on an ARM64 device running the x64 package under emulation, it is noticeably slower than on native x64 hardware, but it does produce captions.

POLICY 10.8.3 AND THE OPTIONAL PROVIDER KEYS

This product does not require financial information for its primary functionality. The primary functionality is captioning, and captioning runs on a fresh install with no credential of any kind, through the bundled on-device engine described above.

Section 10.8 applies to products that include in-product purchase, subscriptions, virtual currency or billing functionality, or that capture financial information. This product has none of the first four. It sells nothing, offers no in-app purchase and no subscription, handles no payment, and has no billing functionality of its own.

The API key the app optionally accepts is a developer credential that the user has already created on their own account with Google, OpenAI or Mistral, for a third-party service the user has chosen to use. The user types it into the app themselves. It is stored in Windows Credential Manager under the service name org.stias.live-translation, read only by the app's Rust core, and used only to authenticate a wss:// connection to the provider that issued it. It is never collected by the developer, who operates no server, no relay and no proxy, and receives no data of any kind: there is no telemetry, no analytics and no crash reporting. The key grants no access to an account balance and buys nothing inside the app. Removing it does not disable the app; the app returns to the keyless on-device engine.

What the optional key unlocks is translation between English and French (there is no on-device translation API on Windows) and two higher-accuracy cloud subtitle engines. The dependency, and the fact that those providers bill the user's own account per minute, is disclosed in the opening lines of the store description together with a per-hour price table, per policy 10.2.4.

MICROPHONE PERMISSION

The package declares the microphone device capability. Capturing a room microphone therefore depends on Settings > Privacy & security > Microphone being enabled for this app; if it is off, the app reports that and names the settings path rather than failing silently. This does not affect the test above: the first-run default source is Windows system audio, which needs no permission, so the keyless primary functionality can be verified without changing any privacy setting.

IF YOU WISH TO EXERCISE THE PAID PATHS

A free key can be created in a few minutes at any of the three providers. Mistral is the cheapest at $0.006 per minute of audio.

- Google Gemini (translation): https://aistudio.google.com/apikey
- OpenAI (translation): https://platform.openai.com/api-keys
- Mistral Voxtral (subtitles): https://console.mistral.ai/api-keys

Paste the key into the key panel that appears on the pre-flight screen once a cloud engine is selected, then start a session.

PRIVACY POLICY

https://fmadore.github.io/Live-translation/privacy

The application is open source: https://github.com/fmadore/Live-translation
```

---

## 4. IARC questionnaire guidance

The rating is generated from the answers, and the answers are a compliance statement, not
listing copy. **Answer the live questionnaire from the app's actual behaviour at the moment of
submission, not from this sheet.** IARC revises its questions and their wording regularly, and
a wrong answer is a certification and legal problem rather than a cosmetic one. Use the table
below only to recognise what each question is asking about and to spot an answer that would
contradict the app.

**Category.** Utility / productivity tool. Not a game, and not a reference or social product.
When the questionnaire offers a general-purpose or utility application route, take it: the
game-specific branches do not apply.

Expected answers, given the app as it stands:

| Question area | Expected answer | Why |
| --- | --- | --- |
| Violence, injury, blood, weapons | No | No such content anywhere in the product. |
| Sexual content, nudity, suggestive themes | No | None. |
| Profanity, crude humour | No | The app authors no text of its own. |
| Controlled substances, alcohol, tobacco | No | No references. |
| Gambling, simulated gambling, real-money wagering | No | None, and no virtual currency. |
| Fear, horror, disturbing content | No | None. |
| In-app purchases, paid random items | No | The app sells nothing. Provider keys are bought from the provider, outside the app. |
| Advertising, advertising identifiers | No | No ads, no ad ID, no ad SDK. |
| User-generated content shared with other users | No | Captions and transcripts stay on the local machine and are shown only in this room. Nothing is uploaded, published, or exchanged between users. |
| Users can interact or communicate with each other | No | There is no messaging, no chat, no lobby, no account and no user directory. |
| Location collected or shared | No | The app requests no location capability and reads no location. |
| Personal information collected or shared by the developer | No | No telemetry, no analytics, no crash reporting, no server. See the privacy policy. |
| Digital purchases / unrestricted web access | No | No browser surface, no store front, no external navigation beyond static links to the privacy policy and the providers' key pages. |
| Connects to third-party services / shares data with third parties | **Yes** | When the user selects a cloud engine and supplies their own key, audio is streamed to Google, OpenAI or Mistral under that provider's own terms. Declare this. Point to the privacy policy URL if a free-text field is offered. |

Two points that are easy to get wrong:

- The **third-party connection question is a yes**, and answering it honestly costs nothing.
  Understating it is the kind of discrepancy a reviewer can check against the privacy policy
  in one click.
- Captions reproduce whatever is said in the room, so the *displayed* text is not authored by
  the developer and cannot be guaranteed free of strong language. That is not
  user-generated content in IARC's sense, because nothing is shared, published or transmitted
  between users. If a question asks specifically about unmoderated content that users can see
  from other users, the answer is still no.

Expected outcome for a utility of this shape: the lowest rating available from each board
(PEGI 3, ESRB Everyone, USK 0, and equivalents). If the questionnaire returns anything
higher, an answer has been misread; go back through it rather than accepting the rating.

Re-run the questionnaire whenever the product gains anything with content of its own, notably
if the bundled rehearsal fixture is ever replaced with different recorded speech.

---

## 5. Screenshot shot-list

Partner Center requires screenshots of **at least 1366×768 pixels**, in PNG. Four are enough;
the field accepts up to ten. Order them as listed, because the first one is what appears in
search results and it is the shot that has to say "this works without a key".

The operator window defaults to 1200×820, which is **below the minimum width**. Either widen
the window past 1366 before capturing, or capture the whole screen on a display running at
1920×1080 or better. Never upscale a smaller capture to reach the minimum.

The app is dark-themed throughout, so a straight capture reads well against both the light and
dark Store backgrounds. Do not add drop shadows, gradients, device frames or marketing text
over the shots: Store screenshots are meant to be the product. Capture at 100% display scaling
so the type is crisp, and keep real bilingual French and English caption text in every shot
rather than placeholder strings.

**1. Operator pre-flight, idle, checklist green.** Launch on the first-run defaults (Live
subtitles / System audio / On-device). Play some audio so the "Audio arriving" check flips
green, and place the overlay once so "Overlay placement" reads "Placed". The shot should show:
the numbered setup rail on the left (01 What to show, 02 Where the audio comes from, 03
language, 04 Engine), the "No key needed / Runs entirely on this machine" row with its green
tick, the "Running cost" row reading "Free", and the primary button reading "Start subtitles".
This is the 10.8.3 argument in one image.

**2. Operator running, captions on the stage.** Start a session with bilingual speech (the
rehearsal fixture is the easiest source) and capture mid-sentence, so a live unfinalized
caption with its caret is visible alongside a finished one. The shot should show the origin
chip on each turn block, the source-language line above the caption, the "Audio arriving"
meters showing signal, and the cost card. Use a cloud engine for this one if you want the
"Est. cost" figure to appear, since the on-device engine shows elapsed time only; a Mistral
session is the cheap way to get that. Two turn blocks at once, from "Both" sources, sells the
dual-capture feature better than one.

**3. The audience overlay over a real slide.** Full-screen a genuine presentation slide, run a
session, and capture the entire screen. The overlay should be in its locked, click-through
state, sitting where an operator would actually have placed it, showing one complete caption
line. Keep the operator window off this shot: it is the audience view. Use your own slide, and
check the frame for anything identifiable, since a projected Zoom or Teams roster will show
participants' names.

**4. Overlay move mode with the placement chrome.** Click "Place it" (or "Adjust", or "Move"
during a session) and capture the overlay with its full move-mode furniture visible: the "Drag
to place" bar, the "Move mode / Captions are paused on the overlay" panel, the keyboard hint
row (Enter locks, Esc cancels, Arrows nudge), the size stepper and "Snap to bottom". Frame it
over the same slide as shot 3 so the two read as a sequence.

One difference the screenshots will show, settled deliberately: the operator window's own title
is **Live Translation & Subtitles**, matching the reserved Store name, so that is what appears
in the taskbar and in Alt-Tab. The in-app header keeps the shorter wordmark **Live Captions**
above the line *Realtime translation & subtitles*. A short in-product name alongside the full
name in the window title is ordinary and accurate, so shots 1 and 2 can be taken as they are.
