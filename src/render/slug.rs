//! GitHub-compatible heading slugs with per-page collision counters.

use std::collections::HashMap;

/// GitHub-compatible slug: lowercase, non-alphanumerics to `-`, collisions
/// suffixed `-1`, `-2`, … via the caller-held counter.
pub fn slugify(text: &str, seen: &mut HashMap<String, usize>) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut prev_dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if (c == ' ' || c == '-' || c == '_') && !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug = "section".into();
    }

    let count = seen.entry(slug.clone()).or_insert(0);
    let result = if *count == 0 {
        slug.clone()
    } else {
        format!("{slug}-{count}")
    };
    *count += 1;
    result
}

/// Inject `id` attributes on HTML heading tags that lack them.
pub fn inject_heading_ids(html: &str) -> String {
    let mut seen = HashMap::new();
    let mut result = String::with_capacity(html.len() + 64);
    let mut rest = html;

    while let Some(start) = find_heading_open(rest) {
        result.push_str(&rest[..start]);
        let after_lt = &rest[start..];
        let tag_end = after_lt.find('>').unwrap_or(after_lt.len() - 1);
        let open_tag = &after_lt[..=tag_end];
        let level = open_tag.as_bytes().get(2).copied().unwrap_or(b'1') as char;

        if let Some(existing) = extract_id_attr(open_tag) {
            // Reserve existing IDs so later auto-slugs do not collide
            seen.entry(existing).or_insert(1);
            result.push_str(open_tag);
            rest = &after_lt[tag_end + 1..];
            continue;
        }

        let close = format!("</h{level}>");
        let body_and_rest = &after_lt[tag_end + 1..];
        if let Some(close_at) = body_and_rest.find(&close) {
            let inner = &body_and_rest[..close_at];
            let plain = strip_tags(inner);
            let id = slugify(&plain, &mut seen);
            let insert_at = open_tag.len() - 1;
            result.push_str(&open_tag[..insert_at]);
            result.push_str(&format!(" id=\"{id}\""));
            result.push('>');
            result.push_str(inner);
            result.push_str(&close);
            rest = &body_and_rest[close_at + close.len()..];
        } else {
            result.push_str(open_tag);
            rest = &after_lt[tag_end + 1..];
        }
    }
    result.push_str(rest);
    result
}

fn find_heading_open(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i] == b'<'
            && (bytes[i + 1] == b'h' || bytes[i + 1] == b'H')
            && matches!(bytes[i + 2], b'1'..=b'6')
            && !bytes[i + 3].is_ascii_alphanumeric()
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn extract_id_attr(open_tag: &str) -> Option<String> {
    // very small parser for id="..." or id='...'
    for key in ["id=\"", "id='"] {
        if let Some(pos) = open_tag.find(key) {
            let start = pos + key.len();
            let quote = key.chars().last().unwrap();
            if let Some(end) = open_tag[start..].find(quote) {
                return Some(open_tag[start..start + end].to_string());
            }
        }
    }
    None
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_matches_github() {
        let mut seen = HashMap::new();
        assert_eq!(slugify("Hello World", &mut seen), "hello-world");
        let mut seen = HashMap::new();
        assert_eq!(slugify("Hello, World!", &mut seen), "hello-world");
        let mut seen = HashMap::new();
        assert_eq!(slugify("Foo_bar", &mut seen), "foo-bar");
    }

    #[test]
    fn test_slugify_collisions_suffixed() {
        let mut seen = HashMap::new();
        assert_eq!(slugify("Same", &mut seen), "same");
        assert_eq!(slugify("Same", &mut seen), "same-1");
        assert_eq!(slugify("Same", &mut seen), "same-2");
    }

    #[test]
    fn test_inject_heading_ids() {
        let html = "<h1>Title</h1><p>x</p><h2>Sub</h2>";
        let out = inject_heading_ids(html);
        assert!(out.contains("id=\"title\""), "got: {out}");
        assert!(out.contains("id=\"sub\""), "got: {out}");
    }
}
