import { get } from 'svelte/store';
import { api } from './tauri';
import { asStatus, describeError } from './errors';
import { t } from './i18n';
import { closeToTray, trayHideExplained, statusMessage } from './stores';
import { closeAction, type CloseChoice } from './document';
import { acknowledgeClose, endsLiveSession, prepareClose, resolveClose } from './quit';
import type { TrayCommand } from './types';

/** Prompts and tray actions share one shutdown sequence; rendering only observes it. */
export function createQuitController(actions: {
	stop: () => Promise<void>;
	toggleOverlayVisible: () => Promise<void>;
}) {
	let closePrompt = $state(false);
	let closeEndedSession = $state(false);
	let closeSaving = $state(false);
	let closeError = $state('');
	let answeringClose = false;

	// Leaving is asked about in stages, and each stage is one question: may this session end,
	// then what about the unsaved text. Never more than one is on screen.
	let sessionPrompt = $state(false);
	let sessionPromptFromTray = $state(false);
	let hidePrompt = $state(false);

	async function onCloseRequested() {
		// Claimed first, unconditionally. A second click on the window's X while a prompt is
		// already up is still an interception the core is counting down on, and letting that
		// one lapse would release the window with the transcript still unsaved.
		await acknowledgeClose();
		switch (closeAction(get(closeToTray), get(trayHideExplained))) {
			case 'hide':
				await hideWindow();
				return;
			case 'explain-then-hide':
				// Said in the window they are looking at, not as a toast: an app that vanishes
				// from the taskbar while holding a microphone has to be sure the message landed.
				hidePrompt = true;
				return;
			case 'quit':
				await beginQuit(false);
		}
	}

	async function hideWindow() {
		try {
			await api.hideToTray();
		} catch (e) {
			statusMessage.set(asStatus(e));
		}
	}

	async function onHideChoice(choice: 'hide' | 'quit') {
		hidePrompt = false;
		if (choice === 'quit') {
			// They meant to leave. The explanation is deliberately *not* marked as given: the
			// app never actually hid, so the first hide still deserves it.
			await beginQuit(false);
			return;
		}
		trayHideExplained.set(true);
		await hideWindow();
	}

	/** One shutdown, whichever way it was asked for. */
	async function beginQuit(fromTray: boolean) {
		if (closePrompt || sessionPrompt || answeringClose) return;
		answeringClose = true;
		try {
			// Ending an event's captions is the operator's decision, never a consequence of a
			// mis-aimed click — so this is asked before anything is stopped.
			if (endsLiveSession()) {
				sessionPromptFromTray = fromTray;
				await showWindowForQuestion();
				sessionPrompt = true;
				return;
			}
			await finishQuit();
		} catch (e) {
			statusMessage.set(asStatus(e));
		} finally {
			answeringClose = false;
		}
	}

	/** Stop, drain, finalize — then either ask about unsaved text or leave. */
	async function finishQuit() {
		const outcome = await prepareClose(actions.stop);
		closeEndedSession = outcome.endedSession;
		closeError = '';
		if (outcome.prompt) {
			await showWindowForQuestion();
			closePrompt = true;
		}
	}

	/** A question the operator cannot answer from the tray, so the window comes back first.
	 *  Failure is not fatal: the prompt still renders if the window was already visible. */
	async function showWindowForQuestion() {
		try {
			await api.showOperator();
		} catch {
			// Ignored deliberately; see above.
		}
	}

	async function onSessionChoice(stopIt: boolean) {
		sessionPrompt = false;
		if (!stopIt) return;
		// Held across the stop and drain as well: with both prompts down, nothing else would
		// stop a second close request from starting the whole sequence again underneath it.
		answeringClose = true;
		try {
			await finishQuit();
		} catch (e) {
			statusMessage.set(asStatus(e));
		} finally {
			answeringClose = false;
		}
	}

	async function onTrayCommand(command: TrayCommand) {
		switch (command) {
			case 'toggle-overlay':
				await actions.toggleOverlayVisible();
				return;
			case 'stop-session':
				await actions.stop();
				return;
			case 'quit':
				// Quit from the tray is counted down on just like a window close.
				await acknowledgeClose();
				await beginQuit(true);
		}
	}

	async function onCloseChoice(choice: CloseChoice) {
		if (choice === 'cancel') {
			closePrompt = false;
			closeEndedSession = false;
			closeError = '';
			return;
		}
		closeSaving = choice === 'save';
		closeError = '';
		try {
			await resolveClose(choice);
			closePrompt = false;
		} catch (e) {
			// Stay open on a failed write: quitting here would lose exactly what the operator
			// just asked to keep.
			closeError = describeError(e, get(t));
		} finally {
			closeSaving = false;
		}
	}

	return {
		get closePrompt() {
			return closePrompt;
		},
		get closeEndedSession() {
			return closeEndedSession;
		},
		get closeSaving() {
			return closeSaving;
		},
		get closeError() {
			return closeError;
		},
		get sessionPrompt() {
			return sessionPrompt;
		},
		get sessionPromptFromTray() {
			return sessionPromptFromTray;
		},
		get hidePrompt() {
			return hidePrompt;
		},
		onCloseRequested,
		onHideChoice,
		onSessionChoice,
		onTrayCommand,
		onCloseChoice,
		hideWindow
	};
}
