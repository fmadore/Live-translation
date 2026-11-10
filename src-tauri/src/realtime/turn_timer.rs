//! When the pump finalizes the open turn on a timer, as pure decisions: it is told the time and
//! what the provider said, and answers when to look next and whether the turn ends there. The
//! socket pump owns the sleep; this owns the reasons, so they are unit-tested without a server.

use std::time::Duration;

use tokio::time::Instant;

/// The deadline while nothing is pending: a day, so the timer is effectively off.
pub(super) const IDLE: Duration = Duration::from_secs(24 * 60 * 60);
/// The longest the idle boundary leaves a turn open at the provider's word that the speaker is
/// mid-sentence (`keeps_turn_open`), counted from the turn's last text. Mistral ends a turn
/// after three seconds of silence whatever its model thinks, so a turn still held past this
/// has lost its end, not merely paused.
pub(super) const MAX_HOLD: Duration = Duration::from_secs(4);

/// The finalize timer of one connection.
///
/// Two things end a turn on a timer. The idle boundary (`finalize_after`, for providers that
/// ask for one) follows the last target text; the provider may hold it open while the speaker
/// is mid-sentence, for up to `MAX_HOLD`. An end-of-turn signal sets a deadline of its own,
/// the grace for the last words' text to arrive, which later text moves neither back nor
/// forward.
#[derive(Debug)]
pub(super) struct TurnTimer {
    finalize_after: Option<Duration>,
    /// Set by an end of turn until the turn is finalized, and from then on the finalize
    /// deadline itself: later text pushes it neither back, as it pushes back the idle
    /// boundary, nor forward to an idle boundary that falls inside the grace.
    turn_end: Option<Instant>,
    /// When the turn last grew, for `MAX_HOLD`.
    last_activity: Instant,
    deadline: Instant,
}

impl TurnTimer {
    pub fn new(finalize_after: Option<Duration>, now: Instant) -> Self {
        Self {
            finalize_after,
            turn_end: None,
            last_activity: now,
            deadline: now + IDLE,
        }
    }

    /// When the pump should look next.
    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    /// New target text. It restarts the idle boundary, unless an end of turn has already set
    /// the deadline: text inside a turn's grace is its last words arriving, and the end already
    /// set is when they have had time to.
    pub fn activity(&mut self, now: Instant) {
        self.last_activity = now;
        if let (Some(after), None) = (self.finalize_after, self.turn_end) {
            self.deadline = now + after;
        }
    }

    /// The provider says the speaker finished: finalize once `grace` has passed, or at an
    /// earlier end already set. Even when the idle boundary would come first: closing there is
    /// what made the final "." a line of its own.
    pub fn end_of_turn(&mut self, now: Instant, grace: Duration) {
        let end = now + grace;
        let end = self.turn_end.map_or(end, |earlier| earlier.min(end));
        self.turn_end = Some(end);
        self.deadline = end;
    }

    /// The deadline has come: whether to finalize the turn now. When the provider says the
    /// speaker is mid-sentence (`keeps_open`, asked only when it could matter), the idle
    /// boundary holds the turn open and looks again a boundary later, for as long as its text
    /// has grown within `MAX_HOLD`. An end of turn whose grace has passed is never held.
    pub fn fire(&mut self, now: Instant, keeps_open: impl FnOnce() -> bool) -> bool {
        let ended = self.turn_end.is_some_and(|end| end <= now);
        let hold = self.finalize_after.filter(|_| {
            !ended && now.duration_since(self.last_activity) < MAX_HOLD && keeps_open()
        });
        if let Some(after) = hold {
            self.deadline = self.turn_end.unwrap_or(now + after);
            false
        } else {
            self.turn_end = None;
            self.deadline = now + IDLE;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AFTER: Duration = Duration::from_millis(900);
    const GRACE: Duration = Duration::from_millis(480);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn nothing_pending_is_a_day_away() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        assert_eq!(timer.deadline(), t0 + IDLE);
        // A provider with no idle boundary: text alone never sets one.
        let mut none = TurnTimer::new(None, t0);
        none.activity(t0 + ms(10));
        assert_eq!(none.deadline(), t0 + IDLE);
        // Firing anyway finalizes (whatever is open) and goes back to idle.
        assert!(timer.fire(t0 + IDLE, || unreachable!("nothing to hold")));
        assert_eq!(timer.deadline(), t0 + IDLE + IDLE);
    }

    #[test]
    fn text_pushes_the_idle_boundary_back() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.activity(t0 + ms(100));
        assert_eq!(timer.deadline(), t0 + ms(1000));
        timer.activity(t0 + ms(600));
        assert_eq!(timer.deadline(), t0 + ms(1500));
        assert!(timer.fire(t0 + ms(1500), || false));
        assert_eq!(timer.deadline(), t0 + ms(1500) + IDLE);
    }

    #[test]
    fn an_end_of_turn_sets_a_deadline_text_does_not_move() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.activity(t0);
        timer.end_of_turn(t0 + ms(100), GRACE);
        assert_eq!(timer.deadline(), t0 + ms(580));
        // Its last words arrive: the deadline stays.
        timer.activity(t0 + ms(400));
        assert_eq!(timer.deadline(), t0 + ms(580));
        // A second end of turn cannot push it later, only earlier.
        timer.end_of_turn(t0 + ms(450), GRACE);
        assert_eq!(timer.deadline(), t0 + ms(580));
        timer.end_of_turn(t0 + ms(460), ms(50));
        assert_eq!(timer.deadline(), t0 + ms(510));
        // Its grace has passed, so a speaker "mid-sentence" does not hold it.
        assert!(timer.fire(t0 + ms(510), || true));
        // And the next turn's text is back on the idle boundary.
        timer.activity(t0 + ms(600));
        assert_eq!(timer.deadline(), t0 + ms(1500));
    }

    #[test]
    fn mid_sentence_holds_the_idle_boundary_until_the_text_has_been_still_too_long() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.activity(t0);
        let mut now = timer.deadline();
        let mut holds = 0;
        while !timer.fire(now, || true) {
            holds += 1;
            assert_eq!(
                timer.deadline(),
                now + AFTER,
                "looks again a boundary later"
            );
            now = timer.deadline();
        }
        // 0.9, 1.8, 2.7 and 3.6 s are held; at 4.5 s the text has been still past MAX_HOLD.
        assert_eq!(holds, 4);
        assert!(now.duration_since(t0) >= MAX_HOLD);
    }

    #[test]
    fn a_released_hold_finalizes_at_the_next_boundary() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.activity(t0);
        assert!(!timer.fire(t0 + AFTER, || true));
        assert!(timer.fire(t0 + AFTER * 2, || false));
    }

    #[test]
    fn a_hold_inside_a_grace_waits_for_the_grace() {
        let t0 = Instant::now();
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.activity(t0);
        // A long grace: the deadline is its end, and a fire before it (as the pump's would be
        // were the timer late to reset) holds until then rather than a boundary later.
        timer.end_of_turn(t0, ms(3000));
        assert!(!timer.fire(t0 + AFTER, || true));
        assert_eq!(timer.deadline(), t0 + ms(3000));
        assert!(timer.fire(t0 + ms(3000), || true));
    }

    #[test]
    fn the_provider_is_asked_only_when_its_answer_could_matter() {
        let t0 = Instant::now();
        let mut none = TurnTimer::new(None, t0);
        assert!(none.fire(t0, || unreachable!("no idle boundary to hold")));
        let mut timer = TurnTimer::new(Some(AFTER), t0);
        timer.end_of_turn(t0, GRACE);
        assert!(timer.fire(t0 + GRACE, || unreachable!("the grace has passed")));
    }
}
