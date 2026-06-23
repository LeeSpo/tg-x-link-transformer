use std::sync::OnceLock;

use regex::Regex;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformedLink {
    pub original: String,
    pub converted: String,
}

pub fn transform_links(text: &str) -> Vec<TransformedLink> {
    url_regex()
        .find_iter(text)
        .filter_map(|matched| transform_url_candidate(matched.as_str()))
        .collect()
}

fn url_regex() -> &'static Regex {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    URL_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\bhttps?://[^\s<>()]+").expect("URL extraction regex must compile")
    })
}

fn transform_url_candidate(candidate: &str) -> Option<TransformedLink> {
    let original = trim_trailing_punctuation(candidate);
    let url = Url::parse(original).ok()?;
    let target_host = target_host(url.host_str()?)?;

    if !is_status_url(&url) {
        return None;
    }

    let mut converted = url;
    converted
        .set_scheme("https")
        .expect("setting https scheme must succeed");
    converted
        .set_host(Some(target_host))
        .expect("setting target host must succeed");
    converted.set_query(None);

    Some(TransformedLink {
        original: original.to_string(),
        converted: converted.to_string(),
    })
}

fn target_host(host: &str) -> Option<&'static str> {
    match host.to_ascii_lowercase().trim_start_matches("www.") {
        "x.com" => Some("fixupx.com"),
        "twitter.com" => Some("fxtwitter.com"),
        _ => None,
    }
}

fn is_status_url(url: &Url) -> bool {
    let mut segments = match url.path_segments() {
        Some(segments) => segments,
        None => return false,
    };

    let Some(_user) = segments.next() else {
        return false;
    };

    matches!(segments.next(), Some("status")) && segments.next().is_some()
}

fn trim_trailing_punctuation(candidate: &str) -> &str {
    candidate.trim_end_matches(['.', ',', '!', '?', ':', ';'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_x_status_link_to_fixupx() {
        let links =
            transform_links("https://x.com/PlusMinusKettle/status/2051551662750232727?s=20");

        assert_eq!(
            links,
            vec![TransformedLink {
                original: "https://x.com/PlusMinusKettle/status/2051551662750232727?s=20"
                    .to_string(),
                converted: "https://fixupx.com/PlusMinusKettle/status/2051551662750232727"
                    .to_string(),
            }]
        );
    }

    #[test]
    fn converts_twitter_status_link_to_fxtwitter() {
        let links = transform_links("https://twitter.com/alice/status/12345");

        assert_eq!(
            links,
            vec![TransformedLink {
                original: "https://twitter.com/alice/status/12345".to_string(),
                converted: "https://fxtwitter.com/alice/status/12345".to_string(),
            }]
        );
    }

    #[test]
    fn strips_query_params_from_converted_link() {
        let links = transform_links("https://www.x.com/bob/status/67890?foo=bar&s=20#reply");

        assert_eq!(
            links[0].converted,
            "https://fixupx.com/bob/status/67890#reply"
        );
    }

    #[test]
    fn converts_multiple_links_in_one_message() {
        let links =
            transform_links("one https://x.com/a/status/1?s=20 two https://twitter.com/b/status/2");

        assert_eq!(
            links
                .iter()
                .map(|link| link.converted.as_str())
                .collect::<Vec<_>>(),
            vec![
                "https://fixupx.com/a/status/1",
                "https://fxtwitter.com/b/status/2",
            ]
        );
    }

    #[test]
    fn ignores_already_transformed_links() {
        let links =
            transform_links("https://fixupx.com/a/status/1 https://fxtwitter.com/b/status/2");

        assert!(links.is_empty());
    }

    #[test]
    fn ignores_plain_text_and_non_status_links() {
        let links = transform_links("hello https://x.com/alice https://twitter.com/search?q=rust");

        assert!(links.is_empty());
    }

    #[test]
    fn allows_http_and_www_prefixes() {
        let links = transform_links("http://www.twitter.com/alice/status/42");

        assert_eq!(links[0].converted, "https://fxtwitter.com/alice/status/42");
    }
}
