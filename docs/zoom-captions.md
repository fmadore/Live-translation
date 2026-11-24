# Captions in Zoom

Send live captions straight into a Zoom meeting's built-in caption bar, so attendees simply turn
captions on in Zoom with nothing to install. This uses Zoom's third-party closed-captioning
interface, documented in
[Zoom's support article](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0060368)
(available since 1.12.0; see [release notes](release-1.12.0.md) and
[live verification](#live-verification)).

Sharing sends **caption text only**. Audio capture and engine setup stay unchanged: use a
microphone for the room, system or application audio for remote participants, and either local
Whisper or a cloud engine. Sharing never opens an extra provider session, so it adds no cloud
cost.

## Setting it up

The first two steps are done in Zoom by the meeting host, who does not need to be the person
running this app. The **Step-by-step setup guide** link inside the app's Zoom dialog opens this
page.

1. **Once, in the Zoom web portal** (sign in at zoom.us), as the host or an account admin:
   1. Open **Settings**, then the **Meeting** tab.
   2. Under **In Meeting (Advanced)**, enable **Manual captions** (also labelled *Manual
      captioning*).
   3. Tick **Allow use of caption API Token to integrate with 3rd-party Closed Captioning
      services** underneath, then save.

   If the toggle is greyed out, an account or group admin has locked it. Zoom occasionally renames
   or moves these settings; searching the page for "caption" finds them quickly.
2. **In each meeting**, the host or co-host clicks the small arrow beside **Show Captions** in the
   meeting controls, chooses **Set up manual captioner**, and clicks **Copy the API token**. The
   host can paste it into the meeting chat for the caption operator. Each token belongs to a
   single meeting; copy a fresh one for the next.
3. **In the app**, click **Zoom captions** in the toolbar, paste the link into **Caption link (API
   token)**, pick the caption language (if running two languages) and the sources (if capturing
   both microphone and system audio), then click **Start sharing**. The app queries Zoom for the
   meeting's latest sequence number to verify the link without displaying anything to attendees.
4. Click **Send test caption** to post one test line to the meeting and confirm delivery.
5. **Attendees** turn captions on with **Show Captions** (or **CC**) in their own Zoom controls.

You can start sharing before or during a caption session, and it stays active across Stop and
Start until you click **Stop sharing** or quit the app. To switch languages or sources, stop
sharing and start again. Rehearsals and the built-in demo are never sent to Zoom, even while
sharing is active.

To caption Zoom without showing the desktop overlay, untick **Show captions on this screen too**
in the dialog. That hides the overlay just like the overlay's Hide button, the tray menu or
Ctrl+Shift+O, without affecting Zoom sharing.

### What the status means

| Status | Meaning |
| --- | --- |
| Sharing | Zoom accepted the link or latest caption. The counter shows how many captions Zoom has accepted. |
| Waiting for the meeting | Zoom reported that the meeting has not started yet (HTTP 400). Captions flow once it begins. |
| Refused | Zoom returned 401, 403 or 404: the link expired, was revoked or was copied incompletely. Copy a new API token. |
| Cannot reach Zoom | No connection, a timeout or a Zoom server error. Each new caption tries again. |

Captions that cannot be delivered in time are counted as skipped.

## What is sent, and when

The caption link acts as a credential for the meeting's captions. The app holds it in memory only
until you quit, so you do not have to paste it again after stopping and restarting a session. It
is never saved to preferences, meeting profiles, history, the recovery copy or logs, and the input
field stays masked in case your screen is projected. Error messages describe what went wrong
without printing the link.

Only finalised text is sent, so attendees never see shifting interim hypotheses:

- **Gemini Live Translate, OpenAI and Mistral** append text without revising earlier words, so
  each completed sentence goes out as soon as the next begins. A stretch of 160 characters without
  sentence-ending punctuation is sent up to its last whole word.
- **Gemini Transcribe Live** and **DeepL** revise tentative text before finalising, so only their
  finalised lines are shared.
- **Local Whisper** emits finalised lines only.

Each Zoom caption is capped at 120 characters, split between sentences where possible and between
words otherwise. Newlines become spaces.

Only one caption language goes to a meeting: the language selected when sharing starts. The app
omits Zoom's undocumented `lang` parameter because Zoom reportedly displays all posted languages
simultaneously rather than letting attendees pick one.

## Delivery

- Every caption carries Zoom's sequence number `seq`, incrementing from the last accepted caption.
  Retries reuse the same number. If a caption is dropped, the app queries Zoom for the latest
  accepted sequence number before sending the next one.
- Requests time out after 3 seconds. Retryable failures (400, 403, 405, 408, 429, 500, 502, 503,
  504 and network errors) retry with randomised exponential backoff for about 5 seconds; all other
  errors give up immediately.
- At most 64 captions queue for delivery, and any caption still waiting after 15 seconds is
  skipped so attendees see current speech after an outage rather than a delayed backlog.
- Delivery runs on its own background task: a slow or unreachable Zoom endpoint never delays
  audio capture, the overlay, the transcript or exports.

## Limits

- Breakout rooms (`subconfid` links), Zoom Rooms and running Zoom's built-in automatic or
  translated captions alongside manual captions have not been tested.
- Zoom documents no line or character limit for its caption bar; the 120-character cap follows
  Microsoft's guidance for Teams and may be adjusted after live testing.
- Users on Zoom's developer forum report a delay of 15 seconds or more before a meeting's first
  caption appears.
- Whether shared captions appear in Zoom's saved transcript or cloud recording has not been
  verified.

## Live verification

Tested in a real meeting on 27 October 2026 on a free (Basic) Zoom account: the caption API token
is available on Basic accounts and captions arrived in Zoom's caption bar. The remaining checks
below are open for recording with the Zoom client version, account type and date:

- [x] Host setup on a Basic (free) account (27 October 2026).
- [ ] Host setup on a licensed account.
- [ ] Start check against a meeting that has not started, one in progress, and an expired link.
- [ ] Attendees on Windows, macOS, mobile and the web client see captions with accents and
      right-to-left scripts intact.
- [ ] Reading pace: how many lines Zoom's caption bar displays and whether 120 characters reads
      comfortably.
- [ ] End-to-end delay from overlay to Zoom's caption bar with local Whisper and a cloud engine.
- [ ] Pause and resume, Stop and Start while sharing, Wi-Fi dropout recovery, and a long meeting.
- [ ] Interaction with Zoom's built-in captions and with another manual captioner.
