//! The caption URL a Zoom host copies with **Copy the API token**, checked before anything is
//! sent to it.
//!
//! The URL is a credential: its query (`id`, `ns`, `expire`, `signature`) is what lets anyone
//! holding it put text in front of every attendee. So a rejection names the rule that failed
//! and never the text, `Link`'s `Debug` shows the host alone, and nothing here logs.

use std::fmt;

use reqwest::Url;

/// The path Zoom serves third-party captions on.
const CAPTION_PATH: &str = "/closedcaption";
/// The same URL's sibling that answers with the last `seq` Zoom accepted.
const LAST_SEQ_PATH: &str = "/closedcaption/seq";

/// A validated Zoom caption URL, without any `seq` of its own.
pub(super) struct Link {
    url: Url,
}

impl Link {
    /// Check a pasted caption URL. The error is the rule that failed, written for the detail
    /// under `error.zoomLink`; it never quotes the text it was given.
    pub(super) fn parse(text: &str) -> Result<Self, &'static str> {
        let mut url = Url::parse(text.trim()).map_err(|_| "not a URL")?;
        if url.scheme() != "https" {
            return Err("not https");
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("has a user name or password");
        }
        // An IP address never ends in a Zoom domain, so it fails here too.
        if !url.host_str().is_some_and(is_zoom) {
            return Err("not a Zoom host");
        }
        if !matches!(url.path(), "/closedcaption" | "/closedcaption/") {
            return Err("not the closedcaption path");
        }
        url.set_path(CAPTION_PATH);
        let present = |key: &str| url.query_pairs().any(|(k, v)| k == key && !v.is_empty());
        if !(present("id") && present("signature")) {
            return Err("missing id or signature");
        }
        // A `seq` pasted along with the rest (from a log or an earlier test) would be sent
        // twice. Removed from the raw query rather than through `query_pairs_mut`, which would
        // re-encode every other parameter: the signature must reach Zoom byte for byte as Zoom
        // wrote it.
        let query = url
            .query()
            .unwrap_or_default()
            .split('&')
            .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some("seq"))
            .collect::<Vec<_>>()
            .join("&");
        url.set_query(Some(&query));
        url.set_fragment(None);
        Ok(Self { url })
    }

    /// A link to a test server, which `parse` would refuse for being plain HTTP on loopback.
    #[cfg(test)]
    pub(super) fn unchecked(url: &str) -> Self {
        Self {
            url: Url::parse(url).expect("a test URL"),
        }
    }

    /// The host, which is all of the URL that may be logged.
    pub(super) fn host(&self) -> &str {
        self.url.host_str().unwrap_or_default()
    }

    /// Where caption number `seq` is posted: the URL as pasted, plus `seq`. Appended to the raw
    /// query for the reason `parse` strips it there.
    pub(super) fn caption(&self, seq: u64) -> Url {
        let mut url = self.url.clone();
        let query = match url.query() {
            Some(query) if !query.is_empty() => format!("{query}&seq={seq}"),
            _ => format!("seq={seq}"),
        };
        url.set_query(Some(&query));
        url
    }

    /// Where Zoom answers with the last `seq` it accepted: the same query on `/closedcaption/seq`.
    pub(super) fn last_seq(&self) -> Url {
        let mut url = self.url.clone();
        url.set_path(LAST_SEQ_PATH);
        url
    }
}

/// The host alone: a `{:?}` in a log line or a panic message must not carry the token.
impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Link").field("host", &self.host()).finish()
    }
}

/// Zoom's own domain and its government cloud. A suffix with its dot, so `evilzoom.us` and
/// `zoom.us.example.com` are not mistaken for either.
fn is_zoom(host: &str) -> bool {
    host == "zoom.us" || host.ends_with(".zoom.us") || host.ends_with(".zoomgov.com")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped like the URL Zoom's Copy the API token produces: `ns` is base64 with its padding,
    /// `spparams` is percent-encoded, and the signature is the secret part.
    const SIGNATURE: &str = "nYtXJqRKCWsecret%2Bpart";
    fn realistic() -> String {
        format!(
            "https://wmcc.zoom.us/closedcaption?id=200610693&ns=GZHkEAQGbDBmb3Jtb3IucmVyZXo=\
             &expire=86400&spparams=id%2Cns%2Cexpire&signature={SIGNATURE}"
        )
    }

    #[test]
    fn accepts_the_url_zoom_hands_out_and_posts_to_it_unchanged() {
        let pasted = realistic();
        // Pasted with the stray whitespace a chat message or an e-mail adds.
        let link = Link::parse(&format!("  {pasted}\n")).unwrap();
        assert_eq!(link.host(), "wmcc.zoom.us");
        assert_eq!(link.caption(42).as_str(), format!("{pasted}&seq=42"));
        assert_eq!(
            link.last_seq().as_str(),
            pasted.replace("/closedcaption?", "/closedcaption/seq?")
        );
    }

    #[test]
    fn accepts_zoom_for_government_and_a_trailing_slash() {
        let link =
            Link::parse("https://wmcc.zoomgov.com/closedcaption/?id=7&signature=abc").unwrap();
        assert_eq!(link.host(), "wmcc.zoomgov.com");
        assert_eq!(
            link.caption(1).as_str(),
            "https://wmcc.zoomgov.com/closedcaption?id=7&signature=abc&seq=1"
        );
        // Breakout rooms add `subconfid`, which goes along like every other parameter.
        let breakout =
            Link::parse("https://us02wmcc.zoom.us/closedcaption?id=7&subconfid=9&signature=abc")
                .unwrap();
        assert!(breakout
            .caption(3)
            .as_str()
            .ends_with("subconfid=9&signature=abc&seq=3"));
    }

    #[test]
    fn a_pasted_seq_is_dropped_and_the_rest_kept_as_written() {
        let link =
            Link::parse("https://wmcc.zoom.us/closedcaption?id=7&seq=12&signature=a%2Bb&seq=13")
                .unwrap();
        assert_eq!(
            link.caption(5).as_str(),
            "https://wmcc.zoom.us/closedcaption?id=7&signature=a%2Bb&seq=5"
        );
    }

    #[test]
    fn refuses_anything_that_is_not_a_zoom_caption_url_and_says_why() {
        let cases = [
            ("", "not a URL"),
            ("paste the API token here", "not a URL"),
            (
                "http://wmcc.zoom.us/closedcaption?id=7&signature=s",
                "not https",
            ),
            (
                "https://me:pw@wmcc.zoom.us/closedcaption?id=7&signature=s",
                "has a user name or password",
            ),
            (
                "https://zoom.us.evil.com/closedcaption?id=7&signature=s",
                "not a Zoom host",
            ),
            (
                "https://evilzoom.us/closedcaption?id=7&signature=s",
                "not a Zoom host",
            ),
            (
                "https://203.0.113.9/closedcaption?id=7&signature=s",
                "not a Zoom host",
            ),
            (
                "https://wmcc.zoom.us/closedcaptions?id=7&signature=s",
                "not the closedcaption path",
            ),
            (
                "https://wmcc.zoom.us/closedcaption/seq?id=7&signature=s",
                "not the closedcaption path",
            ),
            (
                "https://wmcc.zoom.us/closedcaption?id=7",
                "missing id or signature",
            ),
            (
                "https://wmcc.zoom.us/closedcaption?id=&signature=s",
                "missing id or signature",
            ),
        ];
        for (pasted, rule) in cases {
            assert_eq!(Link::parse(pasted).unwrap_err(), rule, "{pasted}");
        }
    }

    /// The detail reaches the operator's screen, which may be projected, and any report they
    /// send. Whatever was wrong with a link, the token stays out of both.
    #[test]
    fn no_rejection_or_debug_output_carries_the_token() {
        let good = realistic();
        for pasted in [
            good.replace("https:", "http:"),
            good.replace("wmcc.zoom.us", "wmcc.zoom.us.example.com"),
            good.replace("/closedcaption", "/captions"),
            good.replace("id=200610693&", ""),
        ] {
            let rule = Link::parse(&pasted).unwrap_err();
            assert!(!rule.contains(SIGNATURE) && !rule.contains("200610693"));
        }
        let debug = format!("{:?}", Link::parse(&good).unwrap());
        assert_eq!(debug, r#"Link { host: "wmcc.zoom.us" }"#);
    }
}
