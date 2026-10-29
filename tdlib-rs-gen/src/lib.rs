// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
// Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module gathers all the code generation submodules and coordinates
//! them, feeding them the right data.
mod enums;
mod functions;
mod metadata;
mod rustifier;
mod types;

use std::io::{self, Write};
use tdlib_rs_parser::tl::{Definition, Type};

/// Don't generate types for definitions of this type,
/// since they are "core" types and treated differently.
const SPECIAL_CASED_TYPES: [&str; 6] = ["Bool", "Bytes", "Int32", "Int53", "Int64", "Ok"];

fn ignore_type(ty: &Type) -> bool {
    SPECIAL_CASED_TYPES.iter().any(|&x| x == ty.name)
}

pub fn generate_rust_code(
    file: &mut impl Write,
    definitions: &[Definition],
    gen_bots_only_api: bool,
) -> io::Result<()> {
    write!(
        file,
        "\
         // Copyright 2020 - developers of the `grammers` project.\n\
         // Copyright 2021 - developers of the `tdlib-rs` project.\n\
         // Copyright 2024 - developers of the `tgt` and `tdlib-rs` projects.\n\
         //\n\
         // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or\n\
         // https://www.apache.org/licenses/LICENSE-2.0> or the MIT license\n\
         // <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your\n\
         // option. This file may not be copied, modified, or distributed\n\
         // except according to those terms.\n\
         "
    )?;

    let metadata = metadata::Metadata::new(definitions);
    types::write_types_mod(file, definitions, &metadata, gen_bots_only_api)?;
    enums::write_enums_mod(file, definitions, &metadata, gen_bots_only_api)?;
    functions::write_functions_mod(file, definitions, &metadata, gen_bots_only_api)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdlib_rs_parser::parse_tl_file;

    const SCHEMA: &str = "
        double ? = Double;
        string ? = String;

        int32 = Int32;
        int53 = Int53;
        int64 = Int64;
        bytes = Bytes;

        boolFalse = Bool;
        boolTrue = Bool;

        vector {t:Type} # [ t ] = Vector t;

        //@description An object of this type can be returned on every function call, in case of an error
        //@code Error code
        //@message Error message
        error code:int32 message:string = Error;

        //@description An object of this type is returned on a successful function call for certain functions
        ok = Ok;

        //@description A point @x X coordinate @y Y coordinate
        point x:double y:double = Point;

        //@description A plain text @text Text
        richTextPlain text:string = RichText;

        //@description A bold text @text Text
        richTextBold text:RichText = RichText;

        //@description The client is closed
        authorizationStateClosed = AuthorizationState;

        //@description A user @id User identifier @access_hash Access hash @photo Profile photo; may be null
        //@bot_token Token of the bot; for bots only
        user id:int53 access_hash:int64 photo:point bot_token:string = User;

        //@description A secret bot type; for bots only @id Identifier
        botSecret id:int32 = BotSecret;

        //@description Users @user_ids Identifiers @messages Messages; messages may be null
        users user_ids:vector<int64> messages:vector<RichText> = Users;

        ---functions---

        //@description Returns a user @user_id User identifier @type Kind of the user
        getUser user_id:int53 type:string = User;

        //@description Closes the client
        close = Ok;

        //@description Answers a query; for bots only @query_id Query identifier
        answerQuery query_id:int64 = Ok;
    ";

    fn generate(gen_bots_only_api: bool) -> String {
        let definitions: Vec<_> = parse_tl_file(SCHEMA.into())
            .filter_map(Result::ok)
            .collect();
        let mut out = Vec::new();
        generate_rust_code(&mut out, &definitions, gen_bots_only_api).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn check_modules() {
        let code = generate(false);
        assert!(code.starts_with("// Copyright"));
        assert!(code.contains("pub mod types {"));
        assert!(code.contains("pub mod enums {"));
        assert!(code.contains("pub mod functions {"));
    }

    #[test]
    fn check_special_cased_types_are_skipped() {
        let code = generate(false);
        for name in SPECIAL_CASED_TYPES {
            assert!(!code.contains(&format!("pub enum {name} {{")), "{name}");
        }
        assert!(!code.contains("pub struct Ok "));
    }

    #[test]
    fn check_struct() {
        let code = generate(false);
        assert!(code.contains(
            "    /// A point\n    #[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]\n    pub struct Point {"
        ));
        assert!(code.contains("        /// X coordinate\n        pub x: f64,"));
        assert!(code.contains(
            "        #[serde_as(as = \"DisplayFromStr\")]\n        pub access_hash: i64,"
        ));
        assert!(code.contains("        pub photo: Option<crate::types::Point>,"));
        assert!(code.contains(
            "        #[serde_as(as = \"Vec<DisplayFromStr>\")]\n        pub user_ids: Vec<i64>,"
        ));
        assert!(code.contains("        pub messages: Vec<Option<crate::enums::RichText>>,"));
    }

    #[test]
    fn check_struct_without_default() {
        let code = generate(false);
        assert!(code.contains(
            "    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]\n    pub struct RichTextBold {"
        ));
    }

    #[test]
    fn check_enum() {
        let code = generate(false);
        assert!(code.contains("    #[serde(tag = \"@type\")]\n    pub enum RichText {"));
        assert!(code.contains(
            "        #[serde(rename(serialize = \"richTextPlain\", deserialize = \"richTextPlain\"))]\n        Plain(crate::types::RichTextPlain),"
        ));
        assert!(code.contains("        Bold(Box<crate::types::RichTextBold>),"));
        assert!(code.contains("        Closed,"));
    }

    #[test]
    fn check_function() {
        let code = generate(false);
        assert!(code.contains(
            "    pub async fn get_user(user_id: i64, r#type: String, client_id: i32) -> Result<crate::enums::User, crate::types::Error> {"
        ));
        assert!(code.contains("            \"@type\": \"getUser\",\n            \"user_id\": user_id,\n            \"type\": r#type,"));
        assert!(code.contains("        Ok(serde_json::from_value(response).unwrap())"));
        assert!(code.contains(
            "    pub async fn close(client_id: i32) -> Result<(), crate::types::Error> {"
        ));
        assert!(code.contains("        Ok(())\n"));
    }

    #[test]
    fn check_bots_only_api_excluded() {
        let code = generate(false);
        assert!(!code.contains("bot_token"));
        assert!(!code.contains("pub struct BotSecret"));
        assert!(!code.contains("BotSecret(crate::types::BotSecret)"));
        assert!(!code.contains("answer_query"));
    }

    #[test]
    fn check_bots_only_api_included() {
        let code = generate(true);
        assert!(code.contains("        pub bot_token: String,"));
        assert!(code.contains("    pub struct BotSecret {"));
        assert!(code.contains("BotSecret(crate::types::BotSecret)"));
        assert!(code.contains("    pub async fn answer_query(query_id: i64, client_id: i32)"));
    }
}
