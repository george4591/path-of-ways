use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};

pub fn render_markdown(input: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, opts);

    let mut events: Vec<Event> = Vec::new();
    let mut in_code_block = false;
    for event in parser {
        match event {
            Event::Html(_) | Event::InlineHtml(_) => continue,
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                events.push(Event::Start(Tag::CodeBlock(kind)));
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                events.push(Event::End(TagEnd::CodeBlock));
            }
            Event::Text(text) if !in_code_block => {
                let mut intermediate: Vec<Event> = Vec::new();
                expand_poe_tags(&text, &mut intermediate);
                for ev in intermediate {
                    if let Event::Text(t) = ev {
                        expand_note_links(&t, &mut events);
                    } else {
                        events.push(ev);
                    }
                }
            }
            other => events.push(other),
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    out
}

pub fn render_inline_md(text: &str) -> String {
    let html = render_markdown(text);
    let trimmed = html.trim();
    trimmed
        .strip_prefix("<p>")
        .and_then(|inner| inner.strip_suffix("</p>"))
        .map(|inner| inner.to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

fn expand_poe_tags<'a>(text: &str, out: &mut Vec<Event<'a>>) {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    let mut last_emit = 0usize;

    let push_text = |out: &mut Vec<Event<'a>>, slice: &str| {
        if !slice.is_empty() {
            out.push(Event::Text(CowStr::Boxed(
                slice.to_string().into_boxed_str(),
            )));
        }
    };

    while cursor < bytes.len() {
        if bytes[cursor] == b'#' {
            let name_start = cursor + 1;
            let mut name_end = name_start;
            while name_end < bytes.len()
                && (bytes[name_end].is_ascii_alphanumeric() || bytes[name_end] == b'_')
            {
                name_end += 1;
            }
            if name_end > name_start && name_end < bytes.len() && bytes[name_end] == b'(' {
                let name = &text[name_start..name_end];
                if let Some(class) = poe_class_for(name) {
                    let content_start = name_end + 1;
                    if let Some(close_offset) = text[content_start..].find(')') {
                        let close = content_start + close_offset;
                        let content = &text[content_start..close];

                        push_text(out, &text[last_emit..cursor]);
                        out.push(Event::Html(CowStr::Boxed(
                            format!("<span class=\"{}\">", class).into_boxed_str(),
                        )));
                        push_text(out, content);
                        out.push(Event::Html(CowStr::Boxed(
                            "</span>".to_string().into_boxed_str(),
                        )));

                        cursor = close + 1;
                        last_emit = cursor;
                        continue;
                    }
                }
            }
        }
        cursor += 1;
    }
    push_text(out, &text[last_emit..]);
}

fn expand_note_links<'a>(text: &str, out: &mut Vec<Event<'a>>) {
    let bytes = text.as_bytes();
    let mut cursor = 0usize;
    let mut last_emit = 0usize;

    let push_text = |out: &mut Vec<Event<'a>>, slice: &str| {
        if !slice.is_empty() {
            out.push(Event::Text(CowStr::Boxed(
                slice.to_string().into_boxed_str(),
            )));
        }
    };

    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'[' && bytes[cursor + 1] == b'[' {
            let after = &text[cursor + 2..];
            if let Some(rel_end) = after.find("]]") {
                let title = &after[..rel_end];
                if !title.is_empty() && !title.contains('\n') && !title.contains('[') {
                    push_text(out, &text[last_emit..cursor]);
                    let href_encoded = title.replace(' ', "%20");
                    out.push(Event::Html(CowStr::Boxed(
                        format!(r#"<a href="note:{}" class="note-link">"#, href_encoded)
                            .into_boxed_str(),
                    )));
                    push_text(out, title);
                    out.push(Event::Html(CowStr::Boxed(
                        "</a>".to_string().into_boxed_str(),
                    )));
                    cursor = cursor + 2 + rel_end + 2;
                    last_emit = cursor;
                    continue;
                }
            }
        }
        cursor += 1;
    }
    push_text(out, &text[last_emit..]);
}

fn poe_class_for(name: &str) -> Option<&'static str> {
    Some(match name {
        "str" => "poe-str",
        "dex" => "poe-dex",
        "int" => "poe-int",
        "normal" => "poe-normal",
        "magic" => "poe-magic",
        "rare" => "poe-rare",
        "unique" => "poe-unique",
        "gem" => "poe-gem",
        "currency" => "poe-currency",
        "quest" => "poe-quest",
        _ => return None,
    })
}
