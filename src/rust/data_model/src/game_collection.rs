use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Visibility;

/// One game definition's membership in a [`GameCollection`], carrying only its
/// position within that collection.
///
/// Ordering is per-*membership*, so the same game can sit at a different
/// position in several collections. Presentation (name, description, image) is
/// intrinsic to the referenced definition and shared across every collection,
/// so it is not repeated here.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItem {
    #[schema(value_type = String)]
    /// The game definition this membership points at.
    pub definition_id: Uuid,
    /// Position within the collection (ascending). The source of truth for
    /// display order; the containing list is kept sorted by it.
    pub sort_order: u32,
}

/// An ordered, described, optionally-illustrated grouping of game definitions.
///
/// A collection is presentation only — it groups [`GameDefinition`](crate::GameDefinition)s
/// for browsing and does not affect generation or scoring (leaderboards stay
/// per-definition). It carries its own [`Visibility`] the same way a definition
/// does; the visibility gates the grouping, while each item still enforces its
/// own access when viewed or played.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameCollection {
    #[schema(value_type = String)]
    /// Unique identifier.
    pub id: Uuid,
    #[schema(value_type = String)]
    /// The user that owns (created) this collection. Curated collections are
    /// owned by an administrator but exposed to every user by the storage layer.
    pub owner_id: Uuid,
    /// Display name.
    pub name: String,
    #[schema(value_type = String)]
    /// Access tier — who may see this collection.
    pub visibility: Visibility,
    /// Optional collection-level description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Cache-key for the optional collection-level image; `None` when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_updated_at: Option<DateTime<Utc>>,
    /// The member games, in display order (kept sorted by `sort_order`).
    #[serde(default)]
    pub items: Vec<CollectionItem>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn ts() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid timestamp")
    }

    fn item(order: u32) -> CollectionItem {
        CollectionItem {
            definition_id: Uuid::nil(),
            sort_order: order,
        }
    }

    fn sample() -> GameCollection {
        GameCollection {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: "Difficulty".to_string(),
            visibility: Visibility::Curated,
            description: Some("Warm up then climb".to_string()),
            image_updated_at: Some(ts()),
            items: vec![item(0), item(1), item(2)],
            created_at: ts(),
            updated_at: ts(),
        }
    }

    #[test]
    fn round_trips_a_collection() {
        let collection = sample();
        let json = serde_json::to_string(&collection).expect("serialize");
        let loaded: GameCollection = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded, collection);
    }

    #[test]
    fn preserves_item_order() {
        let collection = sample();
        let json = serde_json::to_string(&collection).expect("serialize");
        let loaded: GameCollection = serde_json::from_str(&json).expect("deserialize");
        let orders: Vec<u32> = loaded.items.iter().map(|i| i.sort_order).collect();
        assert_eq!(orders, vec![0, 1, 2]);
    }

    #[test]
    fn empty_collection_omits_optionals() {
        let collection = GameCollection {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: "Empty".to_string(),
            visibility: Visibility::Private,
            description: None,
            image_updated_at: None,
            items: vec![],
            created_at: ts(),
            updated_at: ts(),
        };
        let value = serde_json::to_value(&collection).expect("serialize");
        let object = value.as_object().expect("object");
        assert!(!object.contains_key("description"), "absent description must be omitted");
        assert!(!object.contains_key("imageUpdatedAt"), "absent image must be omitted");
        assert_eq!(object["items"], serde_json::json!([]));
    }

    #[test]
    fn deserialises_missing_items_as_empty() {
        // A collection JSON written before an item was ever added must load.
        let json = serde_json::json!({
            "id": Uuid::nil(),
            "ownerId": Uuid::nil(),
            "name": "n",
            "visibility": "private",
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        });
        let loaded: GameCollection = serde_json::from_value(json).expect("deserialize");
        assert!(loaded.items.is_empty());
        assert!(loaded.description.is_none());
    }

    #[test]
    fn serialises_camel_case_wire_names() {
        let value = serde_json::to_value(sample()).expect("serialize");
        let object = value.as_object().expect("object");
        for key in ["ownerId", "createdAt", "updatedAt", "imageUpdatedAt", "items"] {
            assert!(object.contains_key(key), "missing camelCase key `{key}`: {value}");
        }
        let first = object["items"][0].as_object().expect("item object");
        for key in ["definitionId", "sortOrder"] {
            assert!(first.contains_key(key), "missing camelCase item key `{key}`");
        }
    }
}
