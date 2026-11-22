//! #2374: the post offer as the MCP tools report it - the agent's
//! counterpart of the Setup page's banner. `load_machine` and `get_job`
//! say which post the setup's machine asks for and how to take it; the
//! mount itself changes nothing.
//!
//! Provenance: our own reply shape; no vendor material.

use crate::posting::{post_name, post_offer_for, OfferWhy};
use flecs_ecs::prelude::*;
use serde_json::{json, Value};

/// `null` when the setup needs no other post, else the post, why, and
/// the one call that takes it.
pub(super) fn offer_view(world: &World, declared: Option<&str>) -> Value {
    let Some(offer) = post_offer_for(world, declared) else {
        return Value::Null;
    };
    let Some(name) = post_name(&offer.post) else {
        return Value::Null;
    };
    let why = match offer.why {
        OfferWhy::Machine => "the machine names this post".to_owned(),
        OfferWhy::Kind(serves) => camcraft_i18n::tf("the current post serves a {serves}", &[format!("{}", serves)]),
    };
    let hint = match declared {
        Some(s) => camcraft_i18n::tf("set_setup_post {{\"setup\": \"{s}\", \"post\": \"{name}\"}} takes it as one undo step; until then the setup posts with its own post", &[format!("{}", s), format!("{}", name)]),
        None => camcraft_i18n::tf("post with \"post\": \"{name}\" uses it for one program; add_setup, then set_setup_post, keeps it", &[format!("{}", name)]),
    };
    json!({ "post": name, "why": why, "hint": hint })
}
