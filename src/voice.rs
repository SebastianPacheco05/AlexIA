use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Converts Markdown to bounded plain text, intended for Alexa PlainText.
/// This is not an SSML serializer. Code blocks, image descriptions and raw
/// HTML events are discarded. Inline code is retained as ordinary text.
/// The limit counts Unicode scalar values, never slices UTF-8 bytes.
pub fn normalize_for_voice(input: &str, max_chars: usize) -> String {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut text = String::new();
    let mut skipped_depth = 0usize;
    for event in Parser::new_ext(input, options) {
        if skipped_depth > 0 {
            match event {
                Event::Start(_) => skipped_depth += 1,
                Event::End(_) => {
                    skipped_depth -= 1;
                    if skipped_depth == 0 {
                        text.push(' ');
                    }
                }
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(Tag::CodeBlock(_) | Tag::Image { .. } | Tag::HtmlBlock) => {
                skipped_depth = 1;
                text.push(' ');
            }
            Event::Text(value) | Event::Code(value) => text.push_str(&value),
            Event::SoftBreak | Event::HardBreak | Event::Rule => text.push(' '),
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item
                | TagEnd::TableCell | TagEnd::TableRow | TagEnd::TableHead) => text.push(' '),
            _ => {}
        }
    }
    let cleaned: String = text.chars().map(|character| {
        if character.is_control() || matches!(character, '<' | '>') {
            ' '
        } else {
            character
        }
    }).collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.chars().take(max_chars).collect::<String>().trim().to_owned()
}
