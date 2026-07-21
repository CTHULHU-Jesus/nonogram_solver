use essence_math::{CalqAst, Essence, ESSENCE_ORDER};
use std::str::FromStr;
use web_sys::{console::log_1, Event, HtmlInputElement, HtmlTextAreaElement, InputEvent};
use yew::prelude::*;
use yew::{html, html::Scope, Component, Context, Html};

pub struct App {
    log: Vec<Html>,
    input: String,
}

pub enum Msg {
    Input(String),
}

/// Returns true if i ends in a newline
fn complete_input(i: &str) -> bool {
    if i.chars().last() == Some('\n') {
        true
    } else {
        false
    }
}

fn info_window() -> Html {
    let blockquote = |x: &str| html!(<blockquote><pre>{place_linebreaks(x)}</pre></blockquote>);
    let blockquote2 = |x: Html| html!(<blockquote><pre>{x}</pre></blockquote>);
    let example1 = apply_calc("4*fire+air");
    let example2 = apply_calc("moon-yin");
    let mut all_essence = "".to_string();
    for e in ESSENCE_ORDER.iter().rev() {
        let s: String = format!("{e}\n");
        all_essence.push_str(&s);
    }

    html!(
        <div class="helplist">
            <h3>{blockquote("Examples")}</h3>
            <p>{blockquote2(example1)}</p>
            <p>{blockquote2(example2)}</p>
            <h3>{blockquote("All Essence")}</h3>
            <p>{blockquote(&all_essence)}</p>
        </div>
    )
}

fn place_linebreaks(s: &str) -> Vec<Html> {
    let mut v: Vec<Html> = Vec::new();
    for s in s.split("\n") {
        if s != "" {
            v.push(html!(<>{s}<br/></>));
        }
    }
    v
}

fn apply_calc(s: &str) -> Html {
    let e = CalqAst::from_str(s);
    let class = if e.is_ok() { "value" } else { "error" };
    let out = match e {
        Ok(e) => format!(" = {}", e.eval()),
        Err(_) => " ! Parse Error".to_string(),
    };
    let inp = format!("> {}", s);
    html! {
    <blockquote class = {class}>
        <p> {inp} </p>
        <p> {out} </p>
    </blockquote>
        }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &yew::Context<Self>) -> Self {
        App {
            log: Vec::new(),
            input: String::new(),
            // counter: 0
        }
    }
    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        // todo, do calc
        match msg {
            Msg::Input(s) => {
                if complete_input(&s) {
                    self.log.push(apply_calc(&s));
                    self.input = "".to_string();
                } else {
                    self.input = s;
                }
            }
        }
        true
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let oninput = ctx.link().callback(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            Msg::Input(input.value())
        });
        html! {
            <>
                <div class="infowindow">
            {info_window()}
            </div>
                <div class="term">
            // term input
                <div class="input-area">
                <textarea value={self.input.clone()} oninput={oninput}></textarea>
                <a href="https://github.com/CTHULHU-Jesus/Essence_math">{"Github"}</a>
                </div>
            // term log
                <div class="log">
            {self.log.clone()}
            </div>
                </div>
                </>
        }
    }
}
