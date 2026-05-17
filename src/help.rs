use leptos::prelude::*;

use crate::modal::ModalShell;
use crate::notes::render_markdown;

const MD_EXAMPLES: &[(&str, &str)] = &[
    ("Heading", "# Heading 1\n## Heading 2\n### Heading 3"),
    ("Bold", "**bold text**"),
    ("Italic", "*italic text*"),
    ("Strikethrough", "~~strikethrough~~"),
    ("Bullet list", "- item one\n- item two"),
    ("Numbered list", "1. first\n2. second"),
    ("Task list", "- [ ] todo\n- [x] done"),
    ("Inline code", "Use `cargo build`"),
    ("Code block", "```\nfn main() {}\n```"),
    ("Blockquote", "> a quote"),
    ("Link", "[Path of Exile 2](https://pathofexile.com)"),
    ("Horizontal rule", "---"),
    ("Table", "| Item | Cost |\n|------|------|\n| Map | 1 ex |"),
];

const POE_EXAMPLES: &[(&str, &str)] = &[
    ("Strength", "#str(strength)"),
    ("Dexterity", "#dex(dexterity)"),
    ("Intelligence", "#int(intelligence)"),
    ("Normal item", "#normal(normal item)"),
    ("Magic item", "#magic(magic item)"),
    ("Rare item", "#rare(rare item)"),
    ("Unique item", "#unique(unique item)"),
    ("Skill gem", "#gem(Lightning Arrow)"),
    ("Currency", "#currency(Exalted Orb)"),
    ("Quest", "#quest(quest item)"),
];

#[component]
pub fn HelpModal<C>(close: C) -> impl IntoView
where
    C: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <ModalShell cancel=close confirm=close panel_class="max-w-3xl max-h-[85vh] overflow-auto">
            <div class="flex items-center justify-between mb-4">
                    <h3 class="text-xl font-semibold text-fg m-0">"Formatting & shortcuts"</h3>
                    <button
                        class="w-8 h-8 rounded-md text-fg-muted hover:bg-fg hover:text-bg text-xl leading-none transition"
                        on:click=move |_| close()
                        title="Close"
                    >
                        "×"
                    </button>
                </div>

                <h4 class="text-base font-medium text-fg mt-2 mb-2">"Markdown"</h4>
                <ExampleGrid examples=MD_EXAMPLES/>

                <h4 class="text-base font-medium text-fg mt-6 mb-2">"PoE color tags"</h4>
                <ExampleGrid examples=POE_EXAMPLES/>

                <h4 class="text-base font-medium text-fg mt-6 mb-2">"Note linking"</h4>
                <div class="grid grid-cols-[8rem_1fr_1fr] gap-x-3 gap-y-2 text-sm items-start">
                    <div class="text-fg py-1 font-medium">"Wiki link"</div>
                    <pre class="text-fg-muted text-xs bg-bg p-2 rounded whitespace-pre-wrap break-words font-mono m-0">"See [[Build: Lightning Arrow]]"</pre>
                    <div class="text-sm text-fg-muted bg-bg p-2 rounded">
                        "Becomes a clickable link. Clicking opens the note with that title, or creates one if it doesn't exist (case-insensitive match)."
                    </div>
                </div>

                <h4 class="text-base font-medium text-fg mt-6 mb-2">"Keyboard shortcuts"</h4>
                <div class="grid grid-cols-[10rem_1fr] gap-x-3 gap-y-2 text-sm items-baseline">
                    <div class="flex flex-wrap gap-1">
                        <Kbd label="1"/>
                        <Kbd label="2"/>
                        <Kbd label="3"/>
                        <Kbd label="4"/>
                        <Kbd label="5"/>
                    </div>
                    <div class="text-fg-muted">"Switch tabs: Notes · Campaign · Bosses · Recipes · Links (when not typing in a field)"</div>

                    <div class="flex flex-wrap gap-1">
                        <Kbd label="Ctrl"/>
                        <span class="text-fg-muted">"+"</span>
                        <Kbd label="N"/>
                    </div>
                    <div class="text-fg-muted">"New blank note (anywhere; drops you into edit mode)"</div>

                    <div class="flex flex-wrap gap-1">
                        <Kbd label="Ctrl"/>
                        <span class="text-fg-muted">"+"</span>
                        <Kbd label="K"/>
                    </div>
                    <div class="text-fg-muted">"Open / close the global quick switcher (search across everything)"</div>

                    <div class="flex flex-wrap gap-1">
                        <Kbd label="Esc"/>
                    </div>
                    <div class="text-fg-muted">"Close the quick switcher (when focused in its input)"</div>
                </div>

                <h4 class="text-base font-medium text-fg mt-6 mb-2">"Tips"</h4>
                <ul class="text-sm text-fg-muted m-0 pl-5 list-disc space-y-1">
                    <li>
                        "Wrap markdown around a PoE tag for combined formatting, e.g. "
                        <code class="bg-bg px-1 rounded">"**#dex(+30 Dex)**"</code>
                    </li>
                    <li>
                        "Code blocks support syntax highlighting. Add a language hint after the opening fence: "
                        <code class="bg-bg px-1 rounded">"```rust"</code>
                        ", "
                        <code class="bg-bg px-1 rounded">"```xml"</code>
                        ", "
                        <code class="bg-bg px-1 rounded">"```json"</code>
                        ", "
                        <code class="bg-bg px-1 rounded">"```regex"</code>
                        ", "
                        <code class="bg-bg px-1 rounded">"```filter"</code>
                        " (Path of Exile loot filters)."
                    </li>
                    <li>
                        "Paste an image directly while editing a note ("<Kbd label="Ctrl"/>" + "<Kbd label="V"/>") — it's stored locally in IndexedDB and embedded inline."
                    </li>
                </ul>
        </ModalShell>
    }
}

#[component]
fn Kbd(#[prop(into)] label: String) -> impl IntoView {
    view! {
        <kbd class="inline-block px-1.5 py-0.5 rounded border border-border bg-bg text-fg text-xs font-mono shadow-sm">
            {label}
        </kbd>
    }
}

#[component]
fn ExampleGrid(examples: &'static [(&'static str, &'static str)]) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[8rem_1fr_1fr] gap-x-3 gap-y-2 text-sm items-start">
            {examples.iter().map(|(label, syntax)| {
                let rendered = render_markdown(syntax);
                view! {
                    <div class="text-fg py-1 font-medium">{*label}</div>
                    <pre class="text-fg-muted text-xs bg-bg p-2 rounded whitespace-pre-wrap break-words font-mono m-0">{*syntax}</pre>
                    <div class="markdown-preview bg-bg p-2 rounded" inner_html=rendered></div>
                }
            }).collect_view()}
        </div>
    }
}
