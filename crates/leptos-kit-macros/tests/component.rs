use leptos::prelude::*;
use leptos_kit_macros::component;

#[component(element = "button")]
fn Button(
    #[prop(optional)] variant: Option<&'static str>,
    #[prop(optional, into)] class: Option<Signal<String>>,
    children: Children,
) -> impl IntoView {
    let variant = variant.unwrap_or("default");
    let classes = move || match class.map(|c| c.get()) {
        Some(extra) => format!("cn-button cn-button-variant-{variant} {extra}"),
        None => format!("cn-button cn-button-variant-{variant}"),
    };
    view! { <button {..attrs} data-slot="button" class=classes>{children()}</button> }
}

/// input has more than 26 attributes of its own
#[component(element = "input")]
fn Input() -> impl IntoView {
    view! { <input {..attrs} data-slot="input" /> }
}

#[component(element = "div", skip(role))]
fn Group(#[prop(optional, into)] id: Option<Signal<String>>, children: Children) -> impl IntoView {
    let id = move || id.map(|id| id.get()).unwrap_or_else(|| "generated".into());
    view! { <div {..attrs} role="group" id=id>{children()}</div> }
}

fn html(view: impl FnOnce() -> AnyView) -> String {
    Owner::new().with(|| view().to_html())
}

#[test]
fn unset_attributes_render_nothing() {
    let out = html(|| view! { <Button>"Save"</Button> }.into_any());
    assert!(
        out.starts_with(
            r#"<button data-slot="button" class="cn-button cn-button-variant-default">"#
        ),
        "{out}"
    );
}

#[test]
fn attributes_reach_the_root() {
    let out = html(|| {
        view! {
            <Button variant="outline" class="w-full" r#type="submit" disabled=true tabindex=-1 id="save" form="f">
                "Save"
            </Button>
        }
        .into_any()
    });
    assert!(out.starts_with("<button"), "{out}");
    assert!(
        out.contains(r#"class="cn-button cn-button-variant-outline w-full""#),
        "{out}"
    );
    assert!(out.contains(r#"type="submit""#), "{out}");
    assert!(out.contains(" disabled"), "{out}");
    assert!(out.contains(r#"tabindex="-1""#), "{out}");
    assert!(out.contains(r#"id="save""#), "{out}");
    assert!(out.contains(r#"form="f""#), "{out}");
}

#[test]
fn false_booleans_are_absent() {
    let out = html(|| view! { <Button disabled=false>"Save"</Button> }.into_any());
    assert!(!out.contains("disabled"), "{out}");
}

#[test]
fn reactive_values() {
    let out = html(|| {
        let busy = RwSignal::new(true);
        let label = RwSignal::new("Saving".to_string());
        view! { <Button disabled=busy title=move || label.get()>"Save"</Button> }.into_any()
    });
    assert!(out.contains(" disabled"), "{out}");
    assert!(out.contains(r#"title="Saving""#), "{out}");
}

#[test]
fn dashed_keys_and_spreads_still_work() {
    let out = html(|| {
        view! {
            <Button aria-label="Save" data-testid="save" on:click=|_| {} attr:lang="it">"Save"</Button>
        }
        .into_any()
    });
    assert!(out.contains(r#"aria-label="Save""#), "{out}");
    assert!(out.contains(r#"data-testid="save""#), "{out}");
    assert!(out.contains(r#"lang="it""#), "{out}");
}

#[test]
fn more_than_26_attributes() {
    let out = html(|| {
        view! {
            <Input
                r#type="number" name="qty" value="3" min="1" max="9" step="1" required=true
                placeholder="qty" autocomplete="off" maxlength=3u32 readonly=false class="w-20"
                aria-label="Quantity"
            />
        }
        .into_any()
    });
    for expected in [
        r#"type="number""#,
        r#"name="qty""#,
        r#"min="1""#,
        r#"maxlength="3""#,
        " required",
        r#"class="w-20""#,
        r#"aria-label="Quantity""#,
    ] {
        assert!(out.contains(expected), "{expected} missing from {out}");
    }
    assert!(!out.contains("readonly"), "{out}");
}

#[test]
fn declared_and_skipped_attributes_stay_with_the_body() {
    let out = html(|| view! { <Group id="g" title="t">"x"</Group> }.into_any());
    assert_eq!(out.matches("id=").count(), 1, "{out}");
    assert!(out.contains(r#"id="g""#), "{out}");
    assert_eq!(out.matches("role=").count(), 1, "{out}");
    assert!(out.contains(r#"title="t""#), "{out}");
}
