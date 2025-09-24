use eframe::egui;
use egui::{Color32, RichText, FontId};
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel, Options};

use crate::config::Config;

#[derive(Debug, Clone)]
struct MarkdownContext {
    current_heading: Option<HeadingLevel>,
    in_list: bool,
    list_depth: usize,
    list_item_number: usize,
    is_ordered_list: bool,
}

impl MarkdownContext {
    fn new() -> Self {
        Self {
            current_heading: None,
            in_list: false,
            list_depth: 0,
            list_item_number: 0,
            is_ordered_list: false,
        }
    }
}

pub struct RenderedView {
    current_markdown_text: String,
    config: Config,
}

impl RenderedView {
    pub fn new(config: &Config) -> Self {
        Self {
            current_markdown_text: String::new(),
            config: config.clone(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, markdown_text: &str) -> Option<Vec<usize>> {
        self.current_markdown_text = markdown_text.to_string();
        let inner = ui.available_size();
        let mut result = None;
        ui.allocate_ui_with_layout(inner, egui::Layout::top_down(egui::Align::LEFT), |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_salt("rendered_scroll")
                .show(ui, |ui| {
                    if markdown_text.trim().is_empty() {
                        ui.label(
                            egui::RichText::new("Start typing to see your rendered notes (markdown)...")
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .font(egui::FontId::proportional(14.0)),
                        );
                        result = Some(Vec::new());
                    } else {
                        let checkbox_toggles = self.render_markdown(ui, markdown_text);
                        result = Some(checkbox_toggles);
                    }
                });
        });
        result
    }

    fn render_markdown(&self, ui: &mut egui::Ui, markdown_text: &str) -> Vec<usize> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(markdown_text, options);
        let events: Vec<Event> = parser.collect();

        let mut context = MarkdownContext::new();
        let mut checkbox_toggles = Vec::new();
        let mut i = 0;

        while i < events.len() {
            i = self.render_markdown_events(ui, &events, i, &mut context, &mut checkbox_toggles);
        }

        checkbox_toggles
    }

    fn render_markdown_events(&self, ui: &mut egui::Ui, events: &[Event], start: usize, context: &mut MarkdownContext, checkbox_toggles: &mut Vec<usize>) -> usize {
        if start >= events.len() {
            return start;
        }

        match &events[start] {
            Event::Start(Tag::Heading { level, .. }) => {
                context.current_heading = Some(*level);
                self.render_heading_inline(ui, events, start + 1, context)
            }
            Event::Start(Tag::Paragraph) => {
                self.render_paragraph_with_spacing(ui, events, start, context)
            }
            Event::Start(Tag::List(first_item_number)) => {
                self.handle_list_start(context, *first_item_number);
                ui.add_space(4.0);
                start + 1
            }
            Event::End(TagEnd::List(_)) => {
                self.handle_list_end(context);
                ui.add_space(4.0);
                start + 1
            }
            Event::Start(Tag::Item) => {
                self.render_list_item_inline(ui, events, start + 1, context, checkbox_toggles)
            }
            Event::Start(Tag::CodeBlock(_)) => {
                self.render_code_block(ui, events, start + 1)
            }
            Event::Start(Tag::BlockQuote { .. }) => {
                self.render_blockquote(ui, events, start + 1, context, checkbox_toggles)
            }
            _ => start + 1,
        }
    }

    fn handle_list_start(&self, context: &mut MarkdownContext, first_item_number: Option<u64>) {
        context.in_list = true;
        context.list_depth += 1;
        context.is_ordered_list = first_item_number.is_some();
        context.list_item_number = first_item_number.unwrap_or(1) as usize;
    }

    fn handle_list_end(&self, context: &mut MarkdownContext) {
        context.list_depth = context.list_depth.saturating_sub(1);
        if context.list_depth == 0 {
            context.in_list = false;
        }
    }

    fn render_paragraph_with_spacing(&self, ui: &mut egui::Ui, events: &[Event], start: usize, context: &MarkdownContext) -> usize {
        if !context.in_list {
            ui.add_space(4.0);
        }
        self.render_paragraph_inline(ui, events, start + 1, context)
    }

    fn render_heading_inline(&self, ui: &mut egui::Ui, events: &[Event], start: usize, context: &MarkdownContext) -> usize {
        let mut i = start;
        let mut heading_text = String::new();

        while i < events.len() {
            match &events[i] {
                Event::End(TagEnd::Heading(_)) => break,
                Event::Text(text) => heading_text.push_str(text),
                _ => {}
            }
            i += 1;
        }

        let (font_size, color) = match context.current_heading {
            Some(HeadingLevel::H1) => (self.config.markdown_styles.h1.font_size, self.config.markdown_styles.h1.to_color32()),
            Some(HeadingLevel::H2) => (self.config.markdown_styles.h2.font_size, self.config.markdown_styles.h2.to_color32()),
            Some(HeadingLevel::H3) => (self.config.markdown_styles.h3.font_size, self.config.markdown_styles.h3.to_color32()),
            Some(HeadingLevel::H4) => (self.config.markdown_styles.h4.font_size, self.config.markdown_styles.h4.to_color32()),
            Some(HeadingLevel::H5) => (self.config.markdown_styles.h5.font_size, self.config.markdown_styles.h5.to_color32()),
            Some(HeadingLevel::H6) => (self.config.markdown_styles.h6.font_size, self.config.markdown_styles.h6.to_color32()),
            None => (self.config.markdown_styles.paragraph.font_size, Color32::WHITE),
        };

        ui.add_space(8.0);
        ui.label(RichText::new(&heading_text)
            .font(FontId::proportional(font_size))
            .strong()
            .color(color));
        ui.add_space(4.0);

        i + 1
    }

    fn render_paragraph_inline(&self, ui: &mut egui::Ui, events: &[Event], start: usize, _context: &MarkdownContext) -> usize {
        let mut i = start;
        ui.horizontal_wrapped(|ui| {
            let mut in_strong = false;
            let mut in_emphasis = false;
            let mut in_strikethrough = false;

            let mut current_i = i;
            while current_i < events.len() {
                match &events[current_i] {
                    Event::End(TagEnd::Paragraph) => break,
                    Event::Start(Tag::Strong) => { in_strong = true; current_i += 1; }
                    Event::End(TagEnd::Strong) => { in_strong = false; current_i += 1; }
                    Event::Start(Tag::Emphasis) => { in_emphasis = true; current_i += 1; }
                    Event::End(TagEnd::Emphasis) => { in_emphasis = false; current_i += 1; }
                    Event::Start(Tag::Strikethrough) => { in_strikethrough = true; current_i += 1; }
                    Event::End(TagEnd::Strikethrough) => { in_strikethrough = false; current_i += 1; }
                    Event::Start(Tag::Link { link_type: _, dest_url, title: _, id: _ }) => {

                        let mut link_text = String::new();
                        let mut temp_i = current_i;
                        while temp_i < events.len() {
                            match &events[temp_i] {
                                Event::End(TagEnd::Link) => break,
                                Event::Text(text) => {
                                    link_text.push_str(text.as_ref());
                                }
                                _ => {}
                            }
                            temp_i += 1;
                        }

                        if ui.add(egui::Hyperlink::from_label_and_url(&link_text, dest_url.as_ref())).clicked()
                            && let Err(e) = webbrowser::open(dest_url.as_ref()) {
                                eprintln!("Failed to open link: {}", e);
                            }

                        current_i = temp_i + 1;
                    }
                    Event::End(TagEnd::Link) => {
                        current_i += 1;
                    }
                    Event::Text(text) => {
                        let mut rich_text = RichText::new(text.as_ref())
                            .font(FontId::proportional(self.config.rendered_font_size));

                        if in_strikethrough {
                            rich_text = rich_text.strikethrough().color(self.config.markdown_styles.strikethrough.to_color32());
                        } else if in_strong {
                            rich_text = rich_text.strong().color(self.config.markdown_styles.strong.to_color32());
                        } else if in_emphasis {
                            rich_text = rich_text.italics().color(self.config.markdown_styles.emphasis.to_color32());
                        } else {
                            rich_text = rich_text.color(self.config.markdown_styles.paragraph.to_color32());
                        }

                        if in_strong && !in_strikethrough {
                            rich_text = rich_text.strong();
                        }
                        if in_emphasis && !in_strikethrough {
                            rich_text = rich_text.italics();
                        }
                        if in_strikethrough {
                            rich_text = rich_text.strikethrough();
                        }

                        ui.label(rich_text);
                        current_i += 1;
                    }
                    Event::Code(code) => {
                        ui.label(RichText::new(code.as_ref())
                            .monospace()
                            .background_color(Color32::from_rgb(255, 245, 235))
                            .color(self.config.markdown_styles.code_inline.to_color32()));
                        current_i += 1;
                    }
                    Event::SoftBreak => {
                        ui.label(" ");
                        current_i += 1;
                    }
                    _ => {
                        current_i += 1;
                    }
                }
            }
            i = current_i;
        });

        i + 1
    }

    fn render_list_item_inline(&self, ui: &mut egui::Ui, events: &[Event], start: usize, context: &mut MarkdownContext, checkbox_toggles: &mut Vec<usize>) -> usize {
        let indent = 16.0 * context.list_depth.saturating_sub(1) as f32;
        let mut i = start;

        let mut is_task_item = false;
        let mut is_checked = false;

        for event in events.iter().take(events.len().min(start + 5)).skip(start) {
            match event {
                Event::TaskListMarker(checked) => {
                    is_task_item = true;
                    is_checked = *checked;
                    break;
                }
                Event::End(TagEnd::Item) => break,
                _ => {}
            }
        }

        ui.horizontal_wrapped(|ui| {
            ui.add_space(indent);

            if is_task_item {
                let mut checkbox_checked = is_checked;
                if ui.checkbox(&mut checkbox_checked, "").clicked() && checkbox_checked != is_checked {
                    let line_number = self.find_task_line_number(start, context);
                    checkbox_toggles.push(line_number);
                }
            } else {
                let bullet = if context.is_ordered_list {
                    format!("{}. ", context.list_item_number)
                } else {
                    "• ".to_string()
                };
                ui.label(RichText::new(bullet)
                    .color(self.config.markdown_styles.list_bullet.to_color32())
                    .font(self.config.markdown_styles.list_bullet.to_font_id()));
            }

            let mut in_strong = false;
            let mut in_emphasis = false;
            let mut in_strikethrough = false;

            let mut current_i = i;
            while current_i < events.len() {
                match &events[current_i] {
                    Event::End(TagEnd::Item) => break,
                    Event::TaskListMarker(_) => {
                        current_i += 1;
                    }
                    Event::Start(Tag::Strong) => { in_strong = true; current_i += 1; }
                    Event::End(TagEnd::Strong) => { in_strong = false; current_i += 1; }
                    Event::Start(Tag::Emphasis) => { in_emphasis = true; current_i += 1; }
                    Event::End(TagEnd::Emphasis) => { in_emphasis = false; current_i += 1; }
                    Event::Start(Tag::Link { link_type: _, dest_url, title: _, id: _ }) => {

                        let mut link_text = String::new();
                        let mut temp_i = current_i;
                        while temp_i < events.len() {
                            match &events[temp_i] {
                                Event::End(TagEnd::Link) => break,
                                Event::Text(text) => {
                                    link_text.push_str(text.as_ref());
                                }
                                _ => {}
                            }
                            temp_i += 1;
                        }

                        if ui.add(egui::Hyperlink::from_label_and_url(&link_text, dest_url.as_ref())).clicked()
                            && let Err(e) = webbrowser::open(dest_url.as_ref()) {
                                eprintln!("Failed to open link: {}", e);
                            }

                        current_i = temp_i + 1;
                    }
                    Event::End(TagEnd::Link) => {
                        current_i += 1;
                    }
                    Event::Start(Tag::Strikethrough) => { in_strikethrough = true; current_i += 1; }
                    Event::End(TagEnd::Strikethrough) => { in_strikethrough = false; current_i += 1; }
                    Event::Text(text) => {
                        let mut rich_text = RichText::new(text.as_ref())
                            .font(FontId::proportional(self.config.rendered_font_size));

                        if (is_task_item && is_checked) || in_strikethrough {
                            rich_text = rich_text.strikethrough().color(self.config.markdown_styles.strikethrough.to_color32());
                        } else if in_strong {
                            rich_text = rich_text.strong().color(self.config.markdown_styles.strong.to_color32());
                        } else if in_emphasis {
                            rich_text = rich_text.italics().color(self.config.markdown_styles.emphasis.to_color32());
                        } else {
                            rich_text = rich_text.color(self.config.markdown_styles.paragraph.to_color32());
                        }

                        if !is_checked || !is_task_item {
                            if in_strong && !in_strikethrough {
                                rich_text = rich_text.strong();
                            }
                            if in_emphasis && !in_strikethrough {
                                rich_text = rich_text.italics();
                            }
                            if in_strikethrough {
                                rich_text = rich_text.strikethrough();
                            }
                        }

                        ui.label(rich_text);
                        current_i += 1;
                    }
                    Event::Code(code) => {
                        ui.label(RichText::new(code.as_ref())
                            .monospace()
                            .background_color(Color32::from_rgb(255, 245, 235))
                            .color(self.config.markdown_styles.code_inline.to_color32()));
                        current_i += 1;
                    }
                    Event::SoftBreak => {
                        ui.label(" ");
                        current_i += 1;
                    }
                    _ => {
                        current_i += 1;
                    }
                }
            }
            i = current_i;
        });

        if context.is_ordered_list {
            context.list_item_number += 1;
        }

        i + 1
    }

    fn render_code_block(&self, ui: &mut egui::Ui, events: &[Event], start: usize) -> usize {
        let mut i = start;
        let mut code_text = String::new();

        while i < events.len() {
            match &events[i] {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(text) => code_text.push_str(text),
                _ => {}
            }
            i += 1;
        }

        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.label(RichText::new(&code_text)
                .monospace()
                .font(FontId::monospace(self.config.markdown_styles.code_block.font_size))
                .background_color(Color32::from_rgb(
                    self.config.markdown_styles.code_block_background[0],
                    self.config.markdown_styles.code_block_background[1],
                    self.config.markdown_styles.code_block_background[2]
                ))
                .color(self.config.markdown_styles.code_block.to_color32()));
        });
        ui.add_space(8.0);

        i + 1
    }

    fn render_blockquote(&self, ui: &mut egui::Ui, events: &[Event], start: usize, context: &mut MarkdownContext, checkbox_toggles: &mut Vec<usize>) -> usize {
        let mut i = start;

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("▎").color(Color32::from_rgb(120, 120, 120)).font(FontId::proportional(20.0)));
            ui.vertical(|ui| {
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::BlockQuote(_)) => break,
                        _ => {
                            i = self.render_markdown_events(ui, events, i, context, checkbox_toggles);
                        }
                    }
                }
            });
        });
        ui.add_space(4.0);

        i + 1
    }

    fn find_task_line_number(&self, _event_index: usize, _context: &MarkdownContext) -> usize {
        let lines: Vec<&str> = self.current_markdown_text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("- [ ]") || line.contains("- [x]") {
                return i;
            }
        }
        0
    }
}