use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;
use uuid::Uuid;

/// Who may see and play a [`GameDefinition`] (or game collection).
///
/// This is an access *tier*. `Private` is an owner-only draft; `Shared` is
/// accessible to an explicit grant list (held separately by the storage layer);
/// `Public` is any signed-in user; `Curated` is admin-featured. The wire form is
/// the lowercase string `"private"`, `"shared"`, `"public"` or `"curated"`; an
/// unrecognised value falls back to `Private` so a record written by a newer
/// build still loads (and defaults to the most restrictive tier).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Owner-only draft. Not leaderboard-tracked; freely editable.
    #[default]
    Private,
    /// Accessible to an explicit list of granted users.
    Shared,
    /// Discoverable and playable by any signed-in user.
    Public,
    /// Admin-featured; surfaced in curated collections.
    Curated,
}

impl Visibility {
    /// Returns the lowercase wire string for this visibility tier.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::Visibility;
    /// assert_eq!(Visibility::Private.as_wire_str(), "private");
    /// assert_eq!(Visibility::Curated.as_wire_str(), "curated");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Shared => "shared",
            Self::Public => "public",
            Self::Curated => "curated",
        }
    }
}

impl Serialize for Visibility {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for Visibility {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "shared" => Self::Shared,
            "public" => Self::Public,
            "curated" => Self::Curated,
            _ => Self::Private,
        })
    }
}

/// How a game definition's maze layout (and thus its leaderboard) rotates.
///
/// `Static` keeps one fixed layout and board for the life of the definition;
/// `Daily` derives a fresh layout and a per-date board from the definition's
/// seed and the current UTC date, so a daily-challenge definition yields a new,
/// comparable board each day. The wire form is the lowercase string `"static"`
/// or `"daily"`; an unrecognised value falls back to `Static`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Rotation {
    /// One fixed layout and board.
    #[default]
    Static,
    /// Layout and board rotate by UTC date.
    Daily,
}

impl Rotation {
    /// Returns the lowercase wire string for this rotation policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::Rotation;
    /// assert_eq!(Rotation::Static.as_wire_str(), "static");
    /// assert_eq!(Rotation::Daily.as_wire_str(), "daily");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Daily => "daily",
        }
    }
}

impl Serialize for Rotation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "daily" => Self::Daily,
            _ => Self::Static,
        })
    }
}

/// A stored, parametric 3D game.
///
/// Unlike a [`Maze`](crate::Maze), a game definition stores no maze grid: its
/// `config` is an opaque, client-owned JSON blob of generation and render
/// parameters (size, counts, scene, the multi-level `levels` group, …) from
/// which the Bevy client regenerates the whole game deterministically using
/// `seed`. The server stores and forwards `config` without interpreting it. The
/// `seed` is first-class (not inside `config`) so the server can own layout
/// rotation for daily challenges; it is auto-minted and hidden from the editor.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinition {
    #[schema(value_type = String)]
    /// Unique identifier.
    pub id: Uuid,
    #[schema(value_type = String)]
    /// The user that owns (created) this definition. Curated definitions are
    /// owned by an administrator but exposed to every user by the storage layer.
    pub owner_id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional description, shown wherever the game appears (intrinsic to the
    /// game, not per-collection). `None` when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[schema(value_type = String)]
    /// Access tier — who may see and play this definition.
    pub visibility: Visibility,
    /// Generation seed. Fixed per definition so the layout is stable and the
    /// board fair; auto-minted and hidden from the editor.
    pub seed: u64,
    #[schema(value_type = String)]
    /// Layout/board rotation policy.
    pub rotation: Rotation,
    #[schema(value_type = Object)]
    /// Opaque, client-owned generation + render parameters. The server stores
    /// and forwards this verbatim; only its byte size is validated.
    pub config: serde_json::Value,
    /// Cache-key for the game's optional thumbnail image, shown everywhere the
    /// game appears; `None` when unset. The image bytes live in the storage
    /// layer, not here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_updated_at: Option<DateTime<Utc>>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last-update timestamp.
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample() -> GameDefinition {
        let ts = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid timestamp");
        GameDefinition {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: "Nightfall".to_string(),
            description: Some("A three-level night climb".to_string()),
            visibility: Visibility::Public,
            seed: 9_007_199_254_740_991,
            rotation: Rotation::Daily,
            config: serde_json::json!({ "rows": 5, "cols": 5, "levels": { "count": 3 } }),
            image_updated_at: Some(ts),
            created_at: ts,
            updated_at: ts,
        }
    }

    #[test]
    fn round_trips_a_definition() {
        let def = sample();
        let json = serde_json::to_string(&def).expect("serialize");
        let loaded: GameDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded, def);
    }

    #[test]
    fn round_trips_a_large_seed() {
        // Seeds are u64; a big value must survive a serialize/deserialize cycle.
        let mut def = sample();
        def.seed = u64::MAX;
        let json = serde_json::to_string(&def).expect("serialize");
        let loaded: GameDefinition = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(loaded.seed, u64::MAX);
    }

    #[test]
    fn serialises_camel_case_wire_names() {
        let value = serde_json::to_value(sample()).expect("serialize");
        let object = value.as_object().expect("object");
        for key in
            ["ownerId", "createdAt", "updatedAt", "imageUpdatedAt", "visibility", "rotation", "seed", "config"]
        {
            assert!(object.contains_key(key), "missing camelCase key `{key}`: {value}");
        }
        assert_eq!(object["visibility"], serde_json::json!("public"));
        assert_eq!(object["rotation"], serde_json::json!("daily"));
    }

    #[test]
    fn omits_optional_presentation_when_absent() {
        let mut def = sample();
        def.description = None;
        def.image_updated_at = None;
        let value = serde_json::to_value(&def).expect("serialize");
        let object = value.as_object().expect("object");
        assert!(
            !object.contains_key("description"),
            "an absent description must be omitted: {value}"
        );
        assert!(
            !object.contains_key("imageUpdatedAt"),
            "an image-less definition must serialise without the key: {value}"
        );
    }

    #[test]
    fn unknown_visibility_falls_back_to_private() {
        let def: GameDefinition = serde_json::from_value(serde_json::json!({
            "id": Uuid::nil(),
            "ownerId": Uuid::nil(),
            "name": "n",
            "visibility": "galaxy",
            "seed": 1,
            "rotation": "static",
            "config": {},
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        }))
        .expect("deserialize");
        assert_eq!(def.visibility, Visibility::Private);
    }

    #[test]
    fn unknown_rotation_falls_back_to_static() {
        let def: GameDefinition = serde_json::from_value(serde_json::json!({
            "id": Uuid::nil(),
            "ownerId": Uuid::nil(),
            "name": "n",
            "visibility": "public",
            "seed": 1,
            "rotation": "hourly",
            "config": {},
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z"
        }))
        .expect("deserialize");
        assert_eq!(def.rotation, Rotation::Static);
    }
}
