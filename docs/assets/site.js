// Links to headings, the sticky page bar and the table of contents, on the home page and on every
// page docs/_layouts/default.html renders. Without script the pages read the same, with no #
// links, no "On this page" list and the page title alone in the bar.
(() => {
	const root = document.documentElement;
	const masthead = document.querySelector('.sticky-top');
	const main = document.querySelector('main');
	if (!main) return;

	// How far the sticky header reaches down the window, so a heading reached by its link is not
	// hidden under it (the CSS reads --sticky in scroll-margin-top).
	const measure = () => {
		if (masthead) root.style.setProperty('--sticky', `${masthead.offsetHeight}px`);
	};
	measure();
	window.addEventListener('resize', measure);

	const toast = document.createElement('div');
	toast.className = 'toast';
	toast.setAttribute('role', 'status');
	document.body.append(toast);
	let toastTimer;
	const announce = (text) => {
		toast.textContent = text;
		toast.classList.add('is-shown');
		clearTimeout(toastTimer);
		toastTimer = setTimeout(() => toast.classList.remove('is-shown'), 1800);
	};

	const addressOf = (id) => {
		const url = new URL(window.location.href);
		url.hash = id;
		return url.href;
	};
	const copy = async (id) => {
		try {
			await navigator.clipboard.writeText(addressOf(id));
			announce('Link copied');
		} catch {
			// The address bar already shows the link: copying is a convenience.
		}
	};

	// The home page labels each section with its heading; its link is the section's own id
	// (#engines rather than #engines-title), the one the navigation uses.
	const targetOf = (heading) => {
		const section = heading.closest('section[id]');
		if (section && section.getAttribute('aria-labelledby') === heading.id) return section.id;
		return heading.id;
	};

	const headings = [...main.querySelectorAll('h2[id], h3[id], h4[id]')].filter(
		(heading) => !heading.closest('.toc, .more, .card, .sources, .steps')
	);
	for (const heading of headings) {
		const id = targetOf(heading);
		const anchor = document.createElement('a');
		anchor.className = 'anchor';
		anchor.href = `#${id}`;
		anchor.textContent = '#';
		anchor.setAttribute('aria-label', `Link to “${heading.textContent.trim()}”`);
		anchor.addEventListener('click', () => copy(id));
		heading.append(anchor);
	}

	// Guide pages: the bar under the masthead names the section being read, and the table of
	// contents lists the page's sections.
	const prose = main.querySelector('.prose');
	if (!prose) return;
	const sections = headings.filter((h) => h.tagName === 'H2' || h.tagName === 'H3');
	const label = (heading) =>
		[...heading.childNodes]
			.filter((node) => !(node instanceof Element && node.classList.contains('anchor')))
			.map((node) => node.textContent)
			.join('')
			.trim();

	const toc = document.querySelector('.toc');
	const tocLinks = new Map();
	if (toc && sections.length > 1) {
		const list = document.createElement('ol');
		let sub = null;
		for (const heading of sections) {
			const item = document.createElement('li');
			const link = document.createElement('a');
			link.href = `#${heading.id}`;
			link.textContent = label(heading);
			item.append(link);
			tocLinks.set(heading, link);
			if (heading.tagName === 'H2' || !list.lastElementChild) {
				list.append(item);
				sub = null;
			} else {
				if (!sub) {
					sub = document.createElement('ol');
					list.lastElementChild.append(sub);
				}
				sub.append(item);
			}
		}
		toc.append(list);
		toc.hidden = false;
		const wide = window.matchMedia('(min-width: 1040px)');
		const openWhenWide = () => {
			toc.open = wide.matches;
		};
		openWhenWide();
		wide.addEventListener('change', openWhenWide);
		toc.querySelector('summary').addEventListener('click', (event) => {
			if (wide.matches) event.preventDefault();
		});
		toc.addEventListener('click', (event) => {
			if (event.target instanceof HTMLAnchorElement && !wide.matches) toc.open = false;
		});
	}

	const sectionLink = document.querySelector('.pagebar-section');
	const separator = document.querySelector('.pagebar-sep');
	const copyButton = document.querySelector('.pagebar .copy');
	let current = null;

	const update = () => {
		const line = (masthead ? masthead.offsetHeight : 0) + 24;
		let found = null;
		for (const heading of sections) {
			if (heading.getBoundingClientRect().top - line > 0) break;
			found = heading;
		}
		if (found === current) return;
		current = found;
		for (const [heading, link] of tocLinks) {
			if (heading === found) link.setAttribute('aria-current', 'location');
			else link.removeAttribute('aria-current');
		}
		if (sectionLink && separator) {
			if (found) {
				sectionLink.textContent = label(found);
				sectionLink.href = `#${found.id}`;
			}
			sectionLink.hidden = !found;
			separator.hidden = !found;
		}
	};
	let queued = false;
	window.addEventListener(
		'scroll',
		() => {
			if (queued) return;
			queued = true;
			requestAnimationFrame(() => {
				queued = false;
				update();
			});
		},
		{ passive: true }
	);
	update();

	if (copyButton) {
		copyButton.hidden = false;
		copyButton.addEventListener('click', () => copy(current ? current.id : ''));
	}
})();
