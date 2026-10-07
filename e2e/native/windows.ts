// The two things the suite needs from Windows itself rather than from the pages: closing the
// operator window the way its title-bar X does, and finding e2e instances a previous run left
// behind. Windows PowerShell rather than a native module, so nothing new has to be installed;
// it ships with every Windows, the runner included.

import { execFile } from 'node:child_process';
import path from 'node:path';
import { promisify } from 'node:util';

const run = promisify(execFile);

async function powershell(script: string, env: Record<string, string>): Promise<string> {
	const encoded = Buffer.from(script, 'utf16le').toString('base64');
	const { stdout } = await run(
		'powershell.exe',
		['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', encoded],
		// Values travel as variables, not spliced into the script: the title contains `&`.
		{ env: { ...process.env, ...env }, windowsHide: true, timeout: 60_000 }
	);
	return stdout;
}

// WM_SYSCOMMAND with SC_CLOSE is what the frame's X button and Alt+F4 send. Windows turns it
// into WM_CLOSE, which Tauri reports as `CloseRequested` — the event `lifecycle::CloseGuard`
// intercepts. Posted, not sent, so this returns at once and the app answers in its own time.
const CLOSE_WINDOW = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class E2eWindows {
	delegate bool EnumProc(IntPtr hwnd, IntPtr param);
	[DllImport("user32.dll")] static extern bool EnumWindows(EnumProc proc, IntPtr param);
	[DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
	[DllImport("user32.dll", CharSet = CharSet.Unicode)]
	static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int max);
	[DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
	[DllImport("user32.dll")] static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr w, IntPtr l);
	public static int Close(uint pid, string title) {
		int posted = 0;
		EnumWindows((hwnd, param) => {
			uint owner;
			GetWindowThreadProcessId(hwnd, out owner);
			if (owner != pid || !IsWindowVisible(hwnd)) return true;
			var text = new StringBuilder(512);
			GetWindowText(hwnd, text, text.Capacity);
			if (text.ToString() == title && PostMessage(hwnd, 0x0112, (IntPtr)0xF060, IntPtr.Zero)) posted++;
			return true;
		}, IntPtr.Zero);
		return posted;
	}
}
'@
$posted = [E2eWindows]::Close([uint32]$env:E2E_PID, $env:E2E_TITLE)
if ($posted -ne 1) { throw "Expected one visible window titled '$env:E2E_TITLE', closed $posted" }
`;

/** Ask a window of process `pid` to close, as a click on its X would. */
export async function requestWindowClose(pid: number, title: string): Promise<void> {
	await powershell(CLOSE_WINDOW, { E2E_PID: String(pid), E2E_TITLE: title });
}

// The WebView2 browser process carries the profile on its command line, so it names its host
// even when nothing else does: renderers have `--type=`, the browser process does not.
const FIND_INSTANCES = String.raw`
$ErrorActionPreference = 'Stop'
$found = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" |
	Where-Object { $_.CommandLine -and $_.CommandLine.Contains($env:E2E_PROFILE) -and -not $_.CommandLine.Contains('--type=') } |
	ForEach-Object {
		$parent = Get-CimInstance Win32_Process -Filter "ProcessId = $($_.ParentProcessId)"
		[pscustomobject]@{ browser = $_.ProcessId; host = $_.ParentProcessId; hostPath = $parent.ExecutablePath }
	})
ConvertTo-Json -InputObject $found -Compress
`;

interface Instance {
	browser: number;
	host: number;
	hostPath: string | null;
}

export async function killTree(pid: number): Promise<void> {
	await run('taskkill', ['/PID', String(pid), '/T', '/F'], { windowsHide: true }).catch(() => {});
}

/**
 * Stop e2e instances an interrupted run left behind. Two reasons: the single-instance lock
 * makes a second launch hand over to the first and exit, and a running WebView2 holds the
 * profile folder that the next test has to delete. Only what is provably the e2e app goes: a
 * WebView2 on the e2e profile, and its host only when that is the e2e exe itself, since a host
 * that died long ago may have handed its process id to something unrelated.
 */
export async function stopStrayInstances(profile: string, exe: string): Promise<void> {
	// The query below costs a second on an idle machine and much more on a busy one, so first
	// the cheap question: is any process running this exe's image at all? With no match,
	// tasklist prints a localised note instead of a row.
	const image = path.basename(exe);
	const { stdout } = await run('tasklist', ['/FI', `IMAGENAME eq ${image}`, '/FO', 'CSV', '/NH'], {
		windowsHide: true
	});
	if (!stdout.toLowerCase().includes(`"${image.toLowerCase()}"`)) return;

	const output = await powershell(FIND_INSTANCES, { E2E_PROFILE: profile + path.sep });
	const instances = JSON.parse(output.trim() || '[]') as Instance[];
	for (const { browser, host, hostPath } of instances) {
		if (hostPath && path.resolve(hostPath).toLowerCase() === path.resolve(exe).toLowerCase()) {
			await killTree(host);
		}
		await killTree(browser);
	}
}
