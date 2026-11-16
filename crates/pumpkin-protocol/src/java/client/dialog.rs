use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_util::version::JavaMinecraftVersion;

pub struct DialogNBT<'a>(pub DialogNBTSource<'a>);

impl<'a> DialogNBT<'a> {
    #[must_use]
    pub const fn from_dialog(dialog: &'a Dialog) -> Self {
        Self(DialogNBTSource::Struct(dialog))
    }

    #[must_use]
    pub const fn from_nbt(compound: &'a pumpkin_nbt::compound::NbtCompound) -> Self {
        Self(DialogNBTSource::Nbt(compound))
    }

    pub fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        let tag = match &self.0 {
            DialogNBTSource::Struct(dialog) => NbtTag::Compound(dialog.to_nbt_compound(version)?),
            DialogNBTSource::Nbt(nbt) => NbtTag::Compound((*nbt).clone()),
        };
        write.write_nbt_with_version(Some(&tag), version)?;
        Ok(())
    }
}

pub enum DialogNBTSource<'a> {
    Struct(&'a Dialog),
    Nbt(&'a pumpkin_nbt::compound::NbtCompound),
}

/// Vanilla `CommonButtonData` / `ActionButton`: `label`, `tooltip`, `width`, `action`.
/// The client throws (and disconnects the player) on any invalid field, so every
/// refusable value is an `Err` here instead of being sent.
fn button_to_nbt(
    button: &ActionButton,
    version: JavaMinecraftVersion,
) -> Result<NbtCompound, WritingError> {
    let mut c = NbtCompound::new();
    c.put("label", button.text.to_nbt_tag_for_version(&version));
    if let Some(tooltip) = &button.tooltip {
        c.put("tooltip", tooltip.to_nbt_tag_for_version(&version));
    }
    if let Some(width) = button.width {
        // Vanilla `Dialog.WIDTH_CODEC` = intRange(1, 1024) (buttons, inputs and body alike).
        if !(1..=1024).contains(&width) {
            return Err(WritingError::Message(format!(
                "Dialog button width {width} is outside 1..=1024"
            )));
        }
        c.put_int("width", i32::try_from(width).unwrap_or(i32::MAX));
    }
    c.put_compound("action", action_to_nbt(&button.action, version)?);
    Ok(c)
}

fn action_to_nbt(
    action: &DialogAction,
    version: JavaMinecraftVersion,
) -> Result<NbtCompound, WritingError> {
    let mut c = NbtCompound::new();
    match action {
        DialogAction::OpenUrl { url } => {
            c.put_string("type", "minecraft:open_url".to_string());
            c.put_string("url", url.clone());
        }
        DialogAction::Custom { id, payload } => {
            if Identifier::parse(id).is_err() {
                return Err(WritingError::Message(format!(
                    "Invalid custom click id {id:?}"
                )));
            }
            c.put_string("type", "minecraft:dynamic/custom".to_string());
            c.put_string("id", id.clone());
            // `additions` only when the payload is a valid unnamed NBT compound.
            if let Some(mut bytes) = payload.as_deref()
                && let Ok(Some(NbtTag::Compound(additions))) =
                    crate::ser::NetworkReadSliceExt::get_nbt_borrowed(&mut bytes, &version)
            {
                c.put_compound("additions", additions);
            }
        }
    }
    Ok(c)
}

fn input_to_nbt(
    index: usize,
    input: &DialogInput,
    version: JavaMinecraftVersion,
) -> Result<NbtCompound, WritingError> {
    let mut c = NbtCompound::new();
    c.put_string("key", format!("i{index}"));
    match input {
        DialogInput::Boolean {
            label,
            default_value,
        } => {
            c.put_string("type", "minecraft:boolean".to_string());
            c.put("label", label.to_nbt_tag_for_version(&version));
            c.put_bool("initial", *default_value);
        }
        DialogInput::Text {
            label,
            default_value,
            ..
        } => {
            c.put_string("type", "minecraft:text".to_string());
            c.put("label", label.to_nbt_tag_for_version(&version));
            // Vanilla refuses an `initial` longer than `max_length` (default 32).
            let len = default_value.encode_utf16().count().max(32);
            c.put_int("max_length", i32::try_from(len).unwrap_or(i32::MAX));
            c.put_string("initial", default_value.clone());
        }
        DialogInput::NumberRange {
            label,
            min,
            max,
            initial,
            step,
            label_format,
        } => {
            c.put_string("type", "minecraft:number_range".to_string());
            c.put("label", label.to_nbt_tag_for_version(&version));
            c.put_float("start", *min);
            c.put_float("end", *max);
            // Optional in vanilla, which refuses an `initial` outside the real bounds (the
            // range may be decreasing) or a `step` <= 0.
            if (min.min(*max)..=min.max(*max)).contains(initial) {
                c.put_float("initial", *initial);
            }
            if *step > 0.0 {
                c.put_float("step", *step);
            }
            if let Some(format) = label_format {
                c.put_string("label_format", format.clone());
            }
        }
        DialogInput::SingleOption {
            label,
            options,
            initial_index,
        } => {
            if options.is_empty() {
                return Err(WritingError::Message(
                    "single_option input needs at least one option".to_string(),
                ));
            }
            c.put_string("type", "minecraft:single_option".to_string());
            c.put("label", label.to_nbt_tag_for_version(&version));
            let entries = options
                .iter()
                .enumerate()
                .map(|(i, display)| {
                    let mut e = NbtCompound::new();
                    e.put_string("id", i.to_string());
                    e.put("display", display.to_nbt_tag_for_version(&version));
                    if i == *initial_index as usize {
                        e.put_bool("initial", true);
                    }
                    NbtTag::Compound(e)
                })
                .collect();
            c.put_list("options", entries);
        }
    }
    Ok(c)
}

impl Dialog {
    /// Writes this dialog in the vanilla 26.3 `Dialog.DIRECT_CODEC` shape.
    ///
    /// Inputs get the key `i{index}`; `single_option` entries get the id `{index}`.
    ///
    /// # Errors
    /// `dialog_list` (no `dialogs` field available), `item` bodies (unsupported), a
    /// `multi_action` without buttons, or a `confirmation` with fewer than two buttons.
    pub fn to_nbt_compound(
        &self,
        version: &JavaMinecraftVersion,
    ) -> Result<NbtCompound, WritingError> {
        let err = |m: &str| Err(WritingError::Message(m.to_string()));
        let mut c = NbtCompound::new();
        c.put_string("type", self.r#type.clone());
        c.put("title", self.title.to_nbt_tag_for_version(version));
        if let Some(t) = &self.external_title {
            c.put("external_title", t.to_nbt_tag_for_version(version));
        }
        c.put_bool("can_close_with_escape", self.can_close_with_escape);
        // Vanilla only has `close`, `none` and `wait_for_response`; `none` needs `pause: false`
        // and the plugin API never asks for it, so anything but `wait_for_response` closes.
        if let Some(after) = &self.after_action {
            let after = if after == "wait_for_response" {
                "wait_for_response"
            } else {
                "close"
            };
            c.put_string("after_action", after.to_string());
        }

        let mut body = Vec::with_capacity(self.body.len());
        for b in &self.body {
            match b {
                DialogBody::PlainMessage { contents } => {
                    let mut e = NbtCompound::new();
                    e.put_string("type", "minecraft:plain_message".to_string());
                    e.put("contents", contents.to_nbt_tag_for_version(version));
                    body.push(NbtTag::Compound(e));
                }
                DialogBody::Item { .. } => return err("Dialog item bodies are not supported"),
            }
        }
        if !body.is_empty() {
            c.put_list("body", body);
        }
        if !self.inputs.is_empty() {
            let inputs = self
                .inputs
                .iter()
                .enumerate()
                .map(|(i, input)| input_to_nbt(i, input, *version).map(NbtTag::Compound))
                .collect::<Result<_, _>>()?;
            c.put_list("inputs", inputs);
        }

        let button = |b: &ActionButton| button_to_nbt(b, *version);
        match self.r#type.trim_start_matches("minecraft:") {
            "multi_action" => {
                if self.buttons.is_empty() {
                    return err("multi_action dialog needs at least one button");
                }
                let actions = self
                    .buttons
                    .iter()
                    .map(|b| button(b).map(NbtTag::Compound))
                    .collect::<Result<_, _>>()?;
                c.put_list("actions", actions);
            }
            "confirmation" => {
                let (Some(yes), Some(no)) = (self.buttons.first(), self.buttons.get(1)) else {
                    return err("confirmation dialog needs two buttons (yes, no)");
                };
                c.put_compound("yes", button(yes)?);
                c.put_compound("no", button(no)?);
            }
            "notice" => {
                if let Some(b) = self.buttons.first() {
                    c.put_compound("action", button(b)?);
                }
            }
            "server_links" => {}
            "dialog_list" => return err("dialog_list dialogs cannot be serialized"),
            other => {
                return Err(WritingError::Message(format!(
                    "Unknown dialog type {other}"
                )));
            }
        }
        Ok(c)
    }
}

#[derive(Clone, Debug)]
pub struct Dialog {
    pub r#type: String,
    pub title: TextComponent,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<DialogInput>,
    pub buttons: Vec<ActionButton>,
    pub links: Vec<DialogLink>,
    pub exit_action: Option<DialogAction>,
    pub after_action: Option<String>,
    pub can_close_with_escape: bool,
    pub external_title: Option<TextComponent>,
}

#[derive(Clone, Debug)]
pub enum DialogBody {
    PlainMessage { contents: TextComponent },
    Item { item: i32 }, // TODO: ItemStack serialization to NBT
}

#[derive(Clone, Debug)]
pub enum DialogInput {
    Boolean {
        label: TextComponent,
        default_value: bool,
    },
    Text {
        label: TextComponent,
        placeholder: TextComponent,
        default_value: String,
    },
    NumberRange {
        label: TextComponent,
        min: f32,
        max: f32,
        initial: f32,
        step: f32,
        label_format: Option<String>,
    },
    SingleOption {
        label: TextComponent,
        options: Vec<TextComponent>,
        initial_index: u32,
    },
}

#[derive(Clone, Debug)]
pub struct ActionButton {
    pub text: TextComponent,
    pub tooltip: Option<TextComponent>,
    pub width: Option<u32>,
    pub action: DialogAction,
}

#[derive(Clone, Debug)]
pub enum DialogAction {
    OpenUrl {
        url: String,
    },
    Custom {
        id: String,
        payload: Option<Vec<u8>>,
    },
}

#[derive(Clone, Debug)]
pub struct DialogLink {
    pub label: crate::Label,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const V: JavaMinecraftVersion = JavaMinecraftVersion::V_26_3;

    fn t(s: &'static str) -> TextComponent {
        TextComponent::text(s)
    }
    fn s(v: &str) -> NbtTag {
        NbtTag::String(v.into())
    }
    fn comp(entries: Vec<(&str, NbtTag)>) -> NbtTag {
        let mut c = NbtCompound::new();
        for (k, v) in entries {
            c.put(k, v);
        }
        NbtTag::Compound(c)
    }
    fn byte(b: i8) -> NbtTag {
        NbtTag::Byte(b)
    }
    fn button(label: &'static str, action: DialogAction) -> ActionButton {
        ActionButton {
            text: t(label),
            tooltip: None,
            width: None,
            action,
        }
    }
    fn custom(id: &str) -> DialogAction {
        DialogAction::Custom {
            id: id.into(),
            payload: None,
        }
    }
    fn dialog(ty: &str, buttons: Vec<ActionButton>) -> Dialog {
        Dialog {
            r#type: ty.into(),
            title: t("T"),
            body: vec![],
            inputs: vec![],
            buttons,
            links: vec![],
            exit_action: None,
            after_action: None,
            can_close_with_escape: true,
            external_title: None,
        }
    }

    pub(super) fn multi_action() -> Dialog {
        let mut d = dialog(
            "minecraft:multi_action",
            vec![
                button("Go", custom("gaspezia:go")),
                button(
                    "Web",
                    DialogAction::OpenUrl {
                        url: "https://example.org".into(),
                    },
                ),
            ],
        );
        d.body = vec![DialogBody::PlainMessage { contents: t("hi") }];
        d.inputs = vec![
            DialogInput::Boolean {
                label: t("b"),
                default_value: true,
            },
            DialogInput::Text {
                label: t("t"),
                placeholder: t("ignored"),
                default_value: "x".into(),
            },
            DialogInput::NumberRange {
                label: t("n"),
                min: 0.0,
                max: 10.0,
                initial: 5.0,
                step: 1.0,
                label_format: None,
            },
            DialogInput::SingleOption {
                label: t("s"),
                options: vec![t("A"), t("B")],
                initial_index: 0,
            },
        ];
        d
    }

    #[test]
    fn multi_action_matches_vanilla_reference() {
        let expected = comp(vec![
            ("type", s("minecraft:multi_action")),
            ("title", s("T")),
            ("can_close_with_escape", byte(1)),
            (
                "body",
                NbtTag::List(vec![comp(vec![
                    ("type", s("minecraft:plain_message")),
                    ("contents", s("hi")),
                ])]),
            ),
            (
                "inputs",
                NbtTag::List(vec![
                    comp(vec![
                        ("key", s("i0")),
                        ("type", s("minecraft:boolean")),
                        ("label", s("b")),
                        ("initial", byte(1)),
                    ]),
                    comp(vec![
                        ("key", s("i1")),
                        ("type", s("minecraft:text")),
                        ("label", s("t")),
                        ("initial", s("x")),
                        ("max_length", NbtTag::Int(32)),
                    ]),
                    comp(vec![
                        ("key", s("i2")),
                        ("type", s("minecraft:number_range")),
                        ("label", s("n")),
                        ("start", NbtTag::Float(0.0)),
                        ("end", NbtTag::Float(10.0)),
                        ("initial", NbtTag::Float(5.0)),
                        ("step", NbtTag::Float(1.0)),
                    ]),
                    comp(vec![
                        ("key", s("i3")),
                        ("type", s("minecraft:single_option")),
                        ("label", s("s")),
                        (
                            "options",
                            NbtTag::List(vec![
                                comp(vec![
                                    ("id", s("0")),
                                    ("display", s("A")),
                                    ("initial", byte(1)),
                                ]),
                                comp(vec![("id", s("1")), ("display", s("B"))]),
                            ]),
                        ),
                    ]),
                ]),
            ),
            (
                "actions",
                NbtTag::List(vec![
                    comp(vec![
                        ("label", s("Go")),
                        (
                            "action",
                            comp(vec![
                                ("type", s("minecraft:dynamic/custom")),
                                ("id", s("gaspezia:go")),
                            ]),
                        ),
                    ]),
                    comp(vec![
                        ("label", s("Web")),
                        (
                            "action",
                            comp(vec![
                                ("type", s("minecraft:open_url")),
                                ("url", s("https://example.org")),
                            ]),
                        ),
                    ]),
                ]),
            ),
        ]);
        assert_eq!(
            NbtTag::Compound(multi_action().to_nbt_compound(&V).unwrap()),
            expected
        );
    }

    #[test]
    fn confirmation_notice_server_links() {
        let yn = vec![button("Y", custom("a:y")), button("N", custom("a:n"))];
        let c = dialog("minecraft:confirmation", yn.clone())
            .to_nbt_compound(&V)
            .unwrap();
        assert_eq!(
            c.get("yes")
                .unwrap()
                .extract_compound()
                .unwrap()
                .get("label"),
            Some(&s("Y"))
        );
        assert_eq!(
            c.get("no")
                .unwrap()
                .extract_compound()
                .unwrap()
                .get("label"),
            Some(&s("N"))
        );
        assert!(c.get("actions").is_none());
        assert!(
            dialog("minecraft:confirmation", yn[..1].to_vec())
                .to_nbt_compound(&V)
                .is_err()
        );

        let c = dialog("minecraft:notice", yn[..1].to_vec())
            .to_nbt_compound(&V)
            .unwrap();
        assert!(c.get("action").is_some());
        let c = dialog("minecraft:notice", vec![])
            .to_nbt_compound(&V)
            .unwrap();
        assert!(c.get("action").is_none());

        let mut d = dialog("minecraft:server_links", yn);
        d.after_action = Some("close".into());
        let c = d.to_nbt_compound(&V).unwrap();
        assert_eq!(c.get("after_action"), Some(&s("close")));
        for k in ["actions", "yes", "no", "action", "links"] {
            assert!(c.get(k).is_none(), "{k}");
        }
        // absent after_action is omitted (vanilla default), never "none"
        let c = dialog("minecraft:notice", vec![])
            .to_nbt_compound(&V)
            .unwrap();
        assert!(c.get("after_action").is_none());
    }

    #[test]
    fn unsupported_shapes_error() {
        assert!(
            dialog("minecraft:dialog_list", vec![])
                .to_nbt_compound(&V)
                .is_err()
        );
        assert!(
            dialog("minecraft:multi_action", vec![])
                .to_nbt_compound(&V)
                .is_err()
        );
        let mut d = dialog("minecraft:notice", vec![]);
        d.body = vec![DialogBody::Item { item: 0 }];
        assert!(d.to_nbt_compound(&V).is_err());
        // write_packet_data surfaces the error, and succeeds for a valid dialog
        let mut out = Vec::new();
        assert!(
            DialogNBT::from_dialog(&d)
                .write_packet_data(&mut out, &V)
                .is_err()
        );
        let ok = multi_action();
        assert!(
            DialogNBT::from_dialog(&ok)
                .write_packet_data(&mut out, &V)
                .is_ok()
        );
        assert_eq!(out[0], 0x0a);
    }

    #[test]
    fn custom_additions_and_text_length() {
        let mut add = NbtCompound::new();
        add.put_string("k", "v".into());
        let mut payload = Vec::new();
        payload
            .write_nbt_with_version(Some(&NbtTag::Compound(add.clone())), &V)
            .unwrap();
        let c = action_to_nbt(
            &DialogAction::Custom {
                id: "a:b".into(),
                payload: Some(payload),
            },
            V,
        )
        .unwrap();
        assert_eq!(c.get("additions"), Some(&NbtTag::Compound(add)));
        let c = action_to_nbt(
            &DialogAction::Custom {
                id: "a:b".into(),
                payload: Some(vec![1, 2, 3]),
            },
            V,
        )
        .unwrap();
        assert!(c.get("additions").is_none());

        let long = "y".repeat(50);
        let c = input_to_nbt(
            0,
            &DialogInput::Text {
                label: t("l"),
                placeholder: t("p"),
                default_value: long,
            },
            V,
        )
        .unwrap();
        assert_eq!(c.get_int("max_length"), Some(50));
    }

    #[test]
    fn refusable_values_are_errors() {
        let ok = |w: Option<u32>| {
            let mut b = button("B", custom("a:b"));
            b.width = w;
            dialog("minecraft:notice", vec![b]).to_nbt_compound(&V)
        };
        assert!(ok(Some(1)).is_ok() && ok(Some(1024)).is_ok() && ok(None).is_ok());
        assert!(ok(Some(0)).is_err() && ok(Some(1025)).is_err());
        assert!(ok(Some(u32::MAX)).is_err());

        for bad in ["A:b", "a:b c", "a:b:c"] {
            let d = dialog("minecraft:notice", vec![button("B", custom(bad))]);
            assert!(d.to_nbt_compound(&V).is_err(), "{bad:?}");
        }
        let d = dialog("minecraft:notice", vec![button("B", custom("a:b/c.d"))]);
        assert!(d.to_nbt_compound(&V).is_ok());

        let mut d = dialog("minecraft:notice", vec![]);
        d.inputs = vec![DialogInput::SingleOption {
            label: t("s"),
            options: vec![],
            initial_index: 0,
        }];
        assert!(d.to_nbt_compound(&V).is_err());
    }

    #[test]
    fn decreasing_number_range_keeps_initial() {
        let range = |min, max, initial| {
            input_to_nbt(
                0,
                &DialogInput::NumberRange {
                    label: t("n"),
                    min,
                    max,
                    initial,
                    step: 1.0,
                    label_format: None,
                },
                V,
            )
            .unwrap()
        };
        let c = range(10.0, 0.0, 5.0);
        assert_eq!(c.get_float("initial"), Some(5.0));
        assert_eq!(c.get_float("start"), Some(10.0));
        assert!(range(10.0, 0.0, 11.0).get("initial").is_none());
        assert!(range(0.0, 10.0, -1.0).get("initial").is_none());
    }

    #[test]
    fn after_action_only_close_or_wait_for_response() {
        let after = |a: &str| {
            let mut d = dialog("minecraft:notice", vec![]);
            d.after_action = Some(a.into());
            d.to_nbt_compound(&V).unwrap().get("after_action").cloned()
        };
        assert_eq!(after("wait_for_response"), Some(s("wait_for_response")));
        for a in ["close", "peek", "pop", "none", "bogus"] {
            assert_eq!(after(a), Some(s("close")), "{a}");
        }
    }

    /// Writes the network NBT of the test dialogs to `$DIALOG_NBT_DIR` for
    /// `Verify.java` (real vanilla codec). Run with `--ignored`.
    #[test]
    #[ignore = "writes files for the external Java verifier"]
    fn write_nbt_for_java_verifier() {
        let dir = std::env::var("DIALOG_NBT_DIR").expect("DIALOG_NBT_DIR");
        let yn = vec![button("Y", custom("a:y")), button("N", custom("a:n"))];
        let mut with_add = dialog("minecraft:notice", vec![button("Ok", custom("a:o"))]);
        let mut add = NbtCompound::new();
        add.put_string("k", "v".into());
        let mut payload = Vec::new();
        payload
            .write_nbt_with_version(Some(&NbtTag::Compound(add)), &V)
            .unwrap();
        with_add.buttons[0].action = DialogAction::Custom {
            id: "a:o".into(),
            payload: Some(payload),
        };
        let mut long_text = dialog("minecraft:notice", vec![]);
        long_text.inputs = vec![DialogInput::Text {
            label: t("l"),
            placeholder: t("p"),
            default_value: "z".repeat(60),
        }];
        let mut closing = dialog("minecraft:server_links", vec![]);
        closing.after_action = Some("close".into());
        closing.external_title = Some(t("ext"));
        let all = [
            ("multi_action", multi_action()),
            ("confirmation", dialog("minecraft:confirmation", yn.clone())),
            ("notice", dialog("minecraft:notice", yn[..1].to_vec())),
            ("notice_empty", dialog("minecraft:notice", vec![])),
            ("notice_additions", with_add),
            ("text_long", long_text),
            ("server_links", closing),
        ];
        for (name, d) in all {
            let mut out = Vec::new();
            DialogNBT::from_dialog(&d)
                .write_packet_data(&mut out, &V)
                .unwrap();
            std::fs::write(format!("{dir}/{name}.nbt"), out).unwrap();
        }
    }
}
