use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{GameCollection, GameDefinition};

/// Which kind of entity a featured-list row points at.
///
/// The featured list mixes game definitions and game collections in one
/// admin-ordered sequence, so each row carries the kind alongside its id. The
/// wire form is the lowercase string `"definition"` or `"collection"`.
///
/// Unlike [`Visibility`](crate::Visibility) there is no safe default: a row with
/// an unrecognised kind is a data-integrity problem, not a value to coerce, so
/// [`FeaturedGameItemKind::from_wire_str`] returns `None` and the storage layer surfaces
/// a loud error rather than silently picking a kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum FeaturedGameItemKind {
    /// A [`GameDefinition`].
    Definition,
    /// A [`GameCollection`].
    Collection,
}

impl FeaturedGameItemKind {
    /// Returns the lowercase wire string for this kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::FeaturedGameItemKind;
    /// assert_eq!(FeaturedGameItemKind::Definition.as_wire_str(), "definition");
    /// assert_eq!(FeaturedGameItemKind::Collection.as_wire_str(), "collection");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Collection => "collection",
        }
    }

    /// Parses a lowercase wire string into a kind, returning `None` for anything
    /// unrecognised (a stored value that doesn't match is data corruption, not a
    /// value to default). Case-insensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::FeaturedGameItemKind;
    /// assert_eq!(FeaturedGameItemKind::from_wire_str("Collection"), Some(FeaturedGameItemKind::Collection));
    /// assert_eq!(FeaturedGameItemKind::from_wire_str("galaxy"), None);
    /// ```
    pub fn from_wire_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "definition" => Some(Self::Definition),
            "collection" => Some(Self::Collection),
            _ => None,
        }
    }
}

/// One hydrated entry of the featured catalogue — either a full
/// [`GameDefinition`] or a full [`GameCollection`].
///
/// The storage layer returns these in `sort_order` from
/// `list_featured_game_items`, so a caller can render the mixed, admin-ordered
/// list without a second lookup per row.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum FeaturedGameItem {
    /// A featured game definition.
    Definition(GameDefinition),
    /// A featured game collection.
    Collection(GameCollection),
}

impl FeaturedGameItem {
    /// The [`FeaturedGameItemKind`] of this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{FeaturedGameItem, FeaturedGameItemKind, GameCollection, GameCollectionMeta, PlayMode, Visibility};
    /// use uuid::Uuid;
    /// let collection = GameCollection {
    ///     meta: GameCollectionMeta {
    ///         id: Uuid::nil(), owner_id: Uuid::nil(), name: "Difficulty".into(),
    ///         visibility: Visibility::Curated, play_mode: PlayMode::Arcade,
    ///         description: None, image_updated_at: None,
    ///         created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    ///     },
    ///     items: vec![],
    /// };
    /// let item = FeaturedGameItem::Collection(collection);
    /// assert_eq!(item.kind(), FeaturedGameItemKind::Collection);
    /// ```
    pub fn kind(&self) -> FeaturedGameItemKind {
        match self {
            Self::Definition(_) => FeaturedGameItemKind::Definition,
            Self::Collection(_) => FeaturedGameItemKind::Collection,
        }
    }

    /// The id of the wrapped definition or collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{FeaturedGameItem, GameCollection, GameCollectionMeta, PlayMode, Visibility};
    /// use uuid::Uuid;
    /// let id = Uuid::new_v4();
    /// let collection = GameCollection {
    ///     meta: GameCollectionMeta {
    ///         id, owner_id: Uuid::nil(), name: "Difficulty".into(),
    ///         visibility: Visibility::Curated, play_mode: PlayMode::Arcade,
    ///         description: None, image_updated_at: None,
    ///         created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    ///     },
    ///     items: vec![],
    /// };
    /// assert_eq!(FeaturedGameItem::Collection(collection).id(), id);
    /// ```
    pub fn id(&self) -> uuid::Uuid {
        match self {
            Self::Definition(d) => d.id,
            Self::Collection(c) => c.meta.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn kind_round_trips_through_wire_string() {
        for kind in [FeaturedGameItemKind::Definition, FeaturedGameItemKind::Collection] {
            assert_eq!(FeaturedGameItemKind::from_wire_str(kind.as_wire_str()), Some(kind));
        }
    }

    #[test]
    fn kind_parse_is_case_insensitive_and_rejects_unknown() {
        assert_eq!(FeaturedGameItemKind::from_wire_str("DEFINITION"), Some(FeaturedGameItemKind::Definition));
        assert_eq!(FeaturedGameItemKind::from_wire_str("nonsense"), None);
    }

    #[test]
    fn kind_serialises_lowercase() {
        assert_eq!(
            serde_json::to_value(FeaturedGameItemKind::Definition).expect("serialize"),
            serde_json::json!("definition")
        );
    }
}
