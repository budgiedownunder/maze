use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Visibility;

/// How a [`GameCollection`] is played once opened.
///
/// `Arcade` presents the member games as a free-choice picker (pick any game and
/// play it); `Campaign` presents them as an ordered progression (levels unlock as
/// earlier ones are completed). This affects presentation only — leaderboards
/// stay per-definition and generation is unchanged. The wire form is the
/// lowercase string `"arcade"` or `"campaign"`; an unrecognised value falls back
/// to `Arcade`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayMode {
    /// Free-choice: any member game may be picked and played.
    #[default]
    Arcade,
    /// Ordered progression: member games are played in sequence.
    Campaign,
}

impl PlayMode {
    /// Returns the lowercase wire string for this play mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::PlayMode;
    /// assert_eq!(PlayMode::Arcade.as_wire_str(), "arcade");
    /// assert_eq!(PlayMode::Campaign.as_wire_str(), "campaign");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Arcade => "arcade",
            Self::Campaign => "campaign",
        }
    }

    /// Parses a lenient wire string into a play mode — case-insensitive,
    /// unrecognised values falling back to `Arcade`. The single source of truth
    /// for both `Deserialize` and callers reading the mode from a plain string
    /// column.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::PlayMode;
    /// assert_eq!(PlayMode::from_wire_str("Campaign"), PlayMode::Campaign);
    /// assert_eq!(PlayMode::from_wire_str("nonsense"), PlayMode::Arcade);
    /// ```
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "campaign" => Self::Campaign,
            _ => Self::Arcade,
        }
    }
}

impl Serialize for PlayMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for PlayMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_wire_str(&String::deserialize(deserializer)?))
    }
}

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
    #[schema(value_type = String)]
    /// How the collection is played (free-choice `Arcade` or ordered `Campaign`).
    /// `#[serde(default)]` so a collection written before this field existed still
    /// loads (as `Arcade`).
    #[serde(default)]
    pub play_mode: PlayMode,
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
            play_mode: PlayMode::Campaign,
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
            play_mode: PlayMode::Arcade,
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
        // A collection JSON written before `play_mode` existed loads as Arcade.
        assert_eq!(loaded.play_mode, PlayMode::Arcade);
    }

    #[test]
    fn play_mode_round_trips_through_json() {
        for mode in [PlayMode::Arcade, PlayMode::Campaign] {
            let json = serde_json::to_string(&mode).expect("serialize");
            let loaded: PlayMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(loaded, mode);
        }
    }

    #[test]
    fn play_mode_serialises_lowercase_wire_form() {
        assert_eq!(serde_json::to_value(PlayMode::Arcade).unwrap(), serde_json::json!("arcade"));
        assert_eq!(serde_json::to_value(PlayMode::Campaign).unwrap(), serde_json::json!("campaign"));
    }

    #[test]
    fn play_mode_unknown_wire_value_falls_back_to_arcade() {
        let loaded: PlayMode = serde_json::from_value(serde_json::json!("weekly")).expect("deserialize");
        assert_eq!(loaded, PlayMode::Arcade);
    }

    #[test]
    fn deserialises_camel_case_play_mode_key() {
        // The wire key is camelCase `playMode`; a `campaign` collection loads it.
        let json = serde_json::json!({
            "id": Uuid::nil(),
            "ownerId": Uuid::nil(),
            "name": "n",
            "visibility": "private",
            "playMode": "campaign",
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        });
        let loaded: GameCollection = serde_json::from_value(json).expect("deserialize");
        assert_eq!(loaded.play_mode, PlayMode::Campaign);
    }

    #[test]
    fn serialises_camel_case_wire_names() {
        let value = serde_json::to_value(sample()).expect("serialize");
        let object = value.as_object().expect("object");
        for key in ["ownerId", "createdAt", "updatedAt", "imageUpdatedAt", "items", "playMode"] {
            assert!(object.contains_key(key), "missing camelCase key `{key}`: {value}");
        }
        let first = object["items"][0].as_object().expect("item object");
        for key in ["definitionId", "sortOrder"] {
            assert!(first.contains_key(key), "missing camelCase item key `{key}`");
        }
    }
}
