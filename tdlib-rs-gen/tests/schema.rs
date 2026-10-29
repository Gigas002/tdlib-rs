use tdlib_rs_gen::generate_rust_code;
use tdlib_rs_parser::errors::ParseError;
use tdlib_rs_parser::parse_tl_file;

/// The `double`, `string` and `vector` declarations are builtins that the parser skips.
const BUILTIN_DECLARATIONS: usize = 3;

fn api_tl() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tdlib-rs/tl/api.tl");
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn parse_api_schema() {
    let (definitions, errors): (Vec<_>, Vec<_>) = parse_tl_file(api_tl()).partition(Result::is_ok);

    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    assert_eq!(errors.len(), BUILTIN_DECLARATIONS, "{errors:?}");
    assert!(errors.iter().all(|e| *e == ParseError::NotImplemented));
    assert!(definitions.len() > 3000);
}

#[test]
fn generate_api_schema() {
    let definitions: Vec<_> = parse_tl_file(api_tl()).filter_map(Result::ok).collect();

    for gen_bots_only_api in [false, true] {
        let mut out = Vec::new();
        generate_rust_code(&mut out, &definitions, gen_bots_only_api).unwrap();
        let code = String::from_utf8(out).unwrap();

        assert!(code.contains("pub enum Update {"));
        assert!(code.contains("pub async fn get_me(client_id: i32)"));
        // TDLib 1.8.68 introduced a cycle through the bare `inlineButton` type
        assert!(code.contains("Button(Box<crate::types::RichTextButton>),"));
    }
}
