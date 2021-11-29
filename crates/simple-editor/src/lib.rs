use novel_archives_text::parser::token::{ParsedSpan, ParsedToken};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use yew::virtual_dom::VNode;

use novel_archives_text::parser::token;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::timeout::*;

struct Model {
    link: ComponentLink<Self>,
    version: usize,
    viewer_text: String,
    viewer_nodes: Vec<VNode>,
    change_text_timeout: Option<TimeoutTask>,
    ctx: token::ParseContext,
}

enum Msg {
    ChangeText(String),
    ParseRequestText(usize),
}

impl Model {
    fn view_token(&self, token: ParsedToken) -> VNode {
        match token {
            ParsedToken::Plaintext(span) => self.view_plaintext(span),
            ParsedToken::Space(span) => self.view_space(span),
            ParsedToken::NewLine(span) => self.view_newline(span),
            ParsedToken::EmphasisMark(span) => self.view_emphasis_mark(span),
            ParsedToken::Ignore(_) => {
                panic!("unexpected token")
            }
            ParsedToken::Term { body, term_id } => self.view_term(body, term_id),
            ParsedToken::Ruby { body, ruby } => self.view_ruby(body, ruby),
            ParsedToken::KanjiRuby { body, ruby } => self.view_ruby(body, ruby),
            ParsedToken::Annotation { body, description } => {
                self.view_annotation(body, description)
            }
        }
    }
    fn view_emphasis_mark(&self, span: ParsedSpan) -> VNode {
        html! {
            <span class="simple-editor__viewer__emphasis_mark">{span.fragment()}</span>
        }
    }
    fn view_plaintext(&self, span: ParsedSpan) -> VNode {
        html! {
            {span.fragment()}
        }
    }
    fn view_space(&self, span: ParsedSpan) -> VNode {
        html! {
            {span.fragment()}
        }
    }
    fn view_newline(&self, _: ParsedSpan) -> VNode {
        html! {
            <br/>
        }
    }
    fn view_term(
        &self,
        body: ParsedSpan,
        _term_id: novel_archives_text::Id<novel_archives_text::term::Term>,
    ) -> VNode {
        html! {
            <a>{body.fragment()}</a>
        }
    }
    fn view_ruby(&self, body: ParsedSpan, ruby: ParsedSpan) -> VNode {
        html! {
            <ruby>
                <rb>{body.fragment()}</rb>
                <rp>{"("}</rp>
                <rt>{ruby.fragment()}</rt>
                <rp>{")"}</rp>
            </ruby>
        }
    }
    fn view_annotation(
        &self,
        body: ParsedSpan,
        _description: token::iterator::TextIterator,
    ) -> VNode {
        html! {
            <a>{body.fragment()}</a>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            version: 0,
            viewer_text: String::new(),
            viewer_nodes: vec![],
            change_text_timeout: None,
            ctx: token::ParseContext::new(Arc::new(BTreeMap::new())),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeText(text) => {
                let (version, _) = self.version.overflowing_add(1);
                self.version = version;
                self.viewer_text = text;
                self.change_text_timeout = Some(TimeoutService::spawn(
                    Duration::from_millis(300),
                    self.link.callback(move |_| Msg::ParseRequestText(version)),
                ));
                false
            }
            Msg::ParseRequestText(version) => {
                if self.version == version {
                    let iter = token::iterator::TextIterator::new(
                        self.ctx.clone(),
                        token::ParsedSpan::new(&self.viewer_text),
                    );
                    let nodes: Vec<_> = iter
                        .filter(|token| !matches!(token, ParsedToken::Ignore(_)))
                        .map(|token| self.view_token(token))
                        .collect();
                    self.viewer_nodes = nodes;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="simple-editor">
                <div class="simple-editor__editor">
                    <textarea class="simple-editor__editor__textarea"  oninput={self.link.callback(|e:InputData| Msg::ChangeText(e.value))} />
                </div>
                <div class="simple-editor__viewer">
                    {self.viewer_nodes.clone()}
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
