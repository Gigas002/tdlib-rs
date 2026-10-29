use serde_json::json;
use tdlib_rs::{enums, types};

#[test]
fn deserialize_update() {
    let update: enums::Update = serde_json::from_value(json!({
        "@type": "updateAuthorizationState",
        "authorization_state": {"@type": "authorizationStateClosed"},
    }))
    .unwrap();

    assert_eq!(
        update,
        enums::Update::AuthorizationState(types::UpdateAuthorizationState {
            authorization_state: enums::AuthorizationState::Closed,
        })
    );
}

#[test]
fn int64_is_a_string() {
    let value = json!({
        "@type": "updateOption",
        "name": "my_id",
        "value": {"@type": "optionValueInteger", "value": "9007199254740993"},
    });
    let update: enums::Update = serde_json::from_value(value.clone()).unwrap();

    let enums::Update::Option(option) = &update else {
        panic!("unexpected update: {update:?}");
    };
    assert_eq!(
        option.value,
        enums::OptionValue::Integer(types::OptionValueInteger {
            value: 9007199254740993,
        })
    );
    assert_eq!(serde_json::to_value(&update).unwrap(), value);
}

#[test]
fn optional_field() {
    let null: enums::StickerFullType = serde_json::from_value(json!({
        "@type": "stickerFullTypeMask",
        "mask_position": null,
    }))
    .unwrap();
    assert_eq!(
        null,
        enums::StickerFullType::Mask(types::StickerFullTypeMask {
            mask_position: None,
        })
    );

    let value = json!({
        "@type": "stickerFullTypeMask",
        "mask_position": {
            "point": {"@type": "maskPointEyes"},
            "x_shift": 0.5,
            "y_shift": -1.0,
            "scale": 2.0,
        },
    });
    let some: enums::StickerFullType = serde_json::from_value(value.clone()).unwrap();
    let enums::StickerFullType::Mask(mask) = &some else {
        panic!("unexpected sticker type: {some:?}");
    };
    assert_eq!(
        mask.mask_position,
        Some(types::MaskPosition {
            point: enums::MaskPoint::Eyes,
            x_shift: 0.5,
            y_shift: -1.0,
            scale: 2.0,
        })
    );
    assert_eq!(serde_json::to_value(&some).unwrap(), value);
}

#[test]
fn recursive_rich_text() {
    let value = json!({
        "@type": "richTextButton",
        "button": {
            "text": {
                "@type": "richTextBold",
                "text": {"@type": "richTextPlain", "text": "Press"},
            },
            "style": {"@type": "buttonStylePrimary"},
            "type": {"@type": "inlineKeyboardButtonTypeCallback", "data": "ZGF0YQ=="},
        },
    });
    let text: enums::RichText = serde_json::from_value(value.clone()).unwrap();

    let enums::RichText::Button(button) = &text else {
        panic!("unexpected rich text: {text:?}");
    };
    assert_eq!(button.button.style, enums::ButtonStyle::Primary);
    assert_eq!(
        button.button.text,
        enums::RichText::Bold(Box::new(types::RichTextBold {
            text: enums::RichText::Plain(types::RichTextPlain {
                text: "Press".into(),
            }),
        }))
    );
    assert_eq!(serde_json::to_value(&text).unwrap(), value);
}

#[test]
fn deserialize_error() {
    let error: types::Error =
        serde_json::from_value(json!({"@type": "error", "code": 401, "message": "Unauthorized"}))
            .unwrap();

    assert_eq!(error.code, 401);
    assert_eq!(error.message, "Unauthorized");
}

#[test]
fn reject_unknown_type() {
    let result = serde_json::from_value::<enums::Update>(json!({"@type": "updateDoesNotExist"}));
    assert!(result.is_err());
}
