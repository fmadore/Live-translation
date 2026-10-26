# Captions in Zoom

**Status:** on `main`, not yet released. It ships in the first release after 1.11.0, once the
[live checks](#live-verification) below have been made in a real meeting
([#187](https://github.com/fmadore/Live-translation/issues/187)).

The app can send its captions to Zoom's own caption bar, so attendees turn them on in Zoom like
any other captions and nobody installs anything. It uses Zoom's third-party caption interface,
documented in [Zoom's support article](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0060368).

Sharing sends **caption text only**. Which audio is captured, and which engine captions it, is
set up as before: the microphone for the room, system or application audio for the remote
participants, local Whisper or a cloud engine. Sharing never opens another provider session,
so it adds no provider cost.

## Setting it up

The first two steps happen in Zoom and are done by the meeting's host, who may not be the
person running this app. The **Step-by-step setup guide** link in the app's Zoom dialog opens
this page.

1. **Once, in the Zoom web portal** (sign in at zoom.us), as the host or an account admin:
   1. Open **Settings**, then the **Meeting** tab.
   2. Under **In Meeting (Advanced)**, turn on **Manual captions** (also labelled *Manual
      captioning*).
   3. Tick **Allow use of caption API Token to integrate with 3rd-party Closed Captioning
      services** beneath it, and save.

   If the option is greyed out, an account or group admin has locked it and has to change it.
   Zoom moves its settings from time to time, so the labels may differ slightly; searching the
   settings page for "caption" finds them.
2. **In each meeting**, the host or a co-host clicks the small arrow beside **Show Captions** in
   the meeting controls, chooses **Set up manual captioner**, then **Copy the API token**. The
   host can send it to the person running this app in the meeting chat. A token belongs to one
   meeting: copy a new one for the next.
3. **In the app**, choose **Zoom** in the toolbar, paste the link into **Caption link (API
   token)**, choose the caption language (with two caption languages) and the sources (with the
   microphone and system audio both captured), and **Start sharing**. Starting asks Zoom for the
   meeting's last caption number, which checks the link without showing anything to attendees.
4. **Send test caption** shows one line to everyone in the meeting, to confirm it arrives.
5. **Attendees** turn captions on with **Show Captions** (or **CC**) in their own meeting
   controls; nothing needs installing on their side.

Sharing can start before or during a caption session, and it carries on across Stop and Start:
it ends with **Stop sharing** or when the app quits. To change the language or the sources,
stop sharing and start again.

### What the status means

| Status | Meaning |
| --- | --- |
| Sharing | Zoom accepted the link or the last caption. The count is the captions Zoom accepted. |
| Waiting for the meeting | Zoom answered that the meeting has not started (HTTP 400). Captions flow once it has. |
| Refused | Zoom answered 401, 403 or 404: the link has expired, been revoked or was copied incompletely. Copy a new API token. |
| Cannot reach Zoom | No connection, a timeout or a Zoom server error. Each new caption tries again. |

Captions that could not be delivered in time are counted as skipped.

## What is sent, and when

The link is a credential for the meeting's captions. The app keeps it in memory until it quits,
so stopping and starting again needs no new paste. It is never written to the saved setup, a
meeting profile, history, the recovery copy or a log, and the field is masked because the
operator's screen may be projected. Errors name what was wrong
with a link, never the link.

Only finished text is sent, so attendees never see a hypothesis that is later revised:

- **Gemini Live Translate, OpenAI and Mistral** build each line by appending text and never
  change what they have sent, so a completed sentence is shared as soon as the next one begins,
  without waiting for the end of a long turn. A run of 160 characters with no sentence end is
  shared up to its last whole word.
- **Gemini Transcribe Live** revises its hypothesis until it finalizes the line, so only final
  lines are shared.
- **Local Whisper** and the **built-in demo** produce final lines only.

Each Zoom caption is at most 120 characters, cut between sentences where possible and otherwise
between words. Newlines become spaces.

Only one caption language goes to a meeting: the one chosen when sharing starts. The app does
not send Zoom's `lang` parameter, whose behaviour is not documented; Zoom reportedly shows every
language posted to a meeting at once rather than letting each attendee choose one.

## Delivery

- Every caption carries Zoom's sequence number `seq`, one higher than the last accepted caption.
  A retry keeps its number. After a caption is given up, the app asks Zoom for the last number
  it accepted before sending the next one.
- Each request times out after 3 seconds. Failures Zoom lists as worth retrying (400, 403, 405,
  408, 500, 502, 503, 504, plus 429 and network errors) are retried with randomized exponential
  backoff for about 5 seconds; anything else is given up at once.
- At most 64 captions wait to be sent, and a caption still waiting after 15 seconds is skipped:
  after an outage the meeting sees current captions, not a replay of the backlog.
- Sending runs on its own task. A slow or failing Zoom never delays capture, the overlay, the
  transcript or exports, and while sharing is off nothing is sent or kept.

## Limits

- Breakout rooms (`subconfid` links), Zoom Rooms and Zoom's own automatic or translated captions
  running at the same time have not been tested.
- Zoom does not document a length limit or how many lines its caption bar shows; the 120-character
  captions follow Microsoft's guidance for Teams and may be tuned after the live checks.
- Users on Zoom's developer forum report a delay of 15 seconds or more before the first caption of
  a meeting appears.
- Whether shared captions reach Zoom's saved transcript or cloud recording is not verified.

## Live verification

Not yet done. Each check records the Zoom client version, the account type and the date.

- [ ] Host setup as described above, on a licensed account and on a basic account.
- [ ] The start check against a meeting that has not started, one that has, and an expired link.
- [ ] Attendees on Windows, macOS, a phone and the web client see the captions, with accents
      and a right-to-left language intact.
- [ ] Reading pace: whether Zoom's caption bar shows one caption at a time or several lines, and
      whether 120 characters reads well.
- [ ] Delay from the overlay to Zoom's caption bar, with local Whisper and with a cloud engine.
- [ ] Pause and resume, Stop and Start while sharing, a Wi-Fi drop and its recovery, and a long
      meeting.
- [ ] Interaction with Zoom's own captions and with another manual captioner.
