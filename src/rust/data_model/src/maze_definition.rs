use serde::de::{IgnoredAny, SeqAccess, Unexpected, Visitor};
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use utoipa::ToSchema;

use crate::Error;
use crate::MazeCellState;
use crate::MazePoint;

#[derive(Clone, Debug, ToSchema)]
/// Represents a maze definition
pub struct MazeDefinition {
    // 2-d grid (rows x columns) of characters describing the maze layout, where
    // - `'S'`:  Represents the starting cell (limited to one).
    // - `'F'`:  Represents the finishing cell (limited to one).
    // - `'W'`:  Represents a wall.
    // - `' '`:  Represents an empty cell.
    // - `'K'`:  Represents a cell holding a key (multiple allowed).
    // - `'D'`:  Represents a door (multiple allowed).
    // - `'E'`:  Represents an enemy spawn cell (multiple allowed).
    // - `'H'`:  Represents a health-pickup cell (multiple allowed).
    //
    // On the wire each cell is normally a bare single-character string. A cell
    // may instead carry an *override* (non-default characteristics for the
    // entity standing on it), in which case it is encoded as an array holding
    // one entity object `[ { "type": <char>, …fields } ]`. Overrides are parsed
    // out into `cell_entities`; this grid always holds the plain cell
    // character so the generator, solver and renderers keep working on clean
    // characters plus an override lookup.
    pub grid: Vec<Vec<char>>,
    /// Sparse per-cell entities keyed by `(row, col)`. Only cells whose entity
    /// carries a non-default characteristic appear here, so an all-default maze
    /// has an empty map and serialises byte-for-byte as a plain character grid.
    ///
    /// The value is a list so it mirrors the always-array wire form one-to-one;
    /// today every list holds exactly one entity (the length is capped at 1),
    /// so callers read it with `.first()`. Modelling it as a list now means a
    /// future cell holding several co-located entities is a cap relaxation
    /// rather than a change to this type.
    ///
    /// Excluded from the OpenAPI schema: the documented contract is the
    /// character grid; the array-of-one entity form is an optional,
    /// sparsely-used extension layered on individual cells.
    #[schema(ignore, value_type = Object)]
    pub cell_entities: HashMap<(usize, usize), Vec<CellEntity>>,
}

/// Visual rig used to render an enemy (`'E'`) cell.
///
/// This is a *characteristic* layered on an enemy cell, not a new cell type:
/// the chase AI is identical for every variant and only the renderers read it.
/// The wire form is the lowercase string `"goblin"` or `"ghost"`; an
/// unrecognised value falls back to `Goblin` so a maze authored by a newer
/// build still loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyType {
    /// The default enemy rig.
    #[default]
    Goblin,
    /// An alternative enemy rig.
    Ghost,
}

impl EnemyType {
    /// Returns the lowercase wire string for this enemy rig.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::EnemyType;
    /// assert_eq!(EnemyType::Goblin.as_wire_str(), "goblin");
    /// assert_eq!(EnemyType::Ghost.as_wire_str(), "ghost");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Goblin => "goblin",
            Self::Ghost => "ghost",
        }
    }
}

impl Serialize for EnemyType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for EnemyType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "ghost" => Self::Ghost,
            _ => Self::Goblin,
        })
    }
}

/// Visual rig used to render a health-pickup (`'H'`) cell.
///
/// Like [`EnemyType`] this is a per-cell *characteristic*, not a new cell type;
/// only the renderers read it. The wire form is the lowercase string `"heart"`
/// or `"potion"`; an unrecognised value falls back to `Heart`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HealthStyle {
    /// The default health-pickup rig.
    #[default]
    Heart,
    /// An alternative health-pickup rig.
    Potion,
}

impl HealthStyle {
    /// Returns the lowercase wire string for this health-pickup rig.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::HealthStyle;
    /// assert_eq!(HealthStyle::Heart.as_wire_str(), "heart");
    /// assert_eq!(HealthStyle::Potion.as_wire_str(), "potion");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Heart => "heart",
            Self::Potion => "potion",
        }
    }
}

impl Serialize for HealthStyle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for HealthStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "potion" => Self::Potion,
            _ => Self::Heart,
        })
    }
}

/// Visual rig used to render a key-holder (`'K'`) cell.
///
/// Like [`EnemyType`] this is a per-cell *characteristic*, not a new cell type;
/// only the renderers read it. The wire form is the `snake_case` string
/// `"pedestal"`, `"chest"` or `"floating_key"`; an unrecognised value falls
/// back to `Pedestal`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KeyHolderStyle {
    /// The default key-holder rig.
    #[default]
    Pedestal,
    /// A chest the key sits inside.
    Chest,
    /// A free-floating key with no holder.
    FloatingKey,
}

impl KeyHolderStyle {
    /// Returns the `snake_case` wire string for this key-holder rig.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::KeyHolderStyle;
    /// assert_eq!(KeyHolderStyle::Pedestal.as_wire_str(), "pedestal");
    /// assert_eq!(KeyHolderStyle::FloatingKey.as_wire_str(), "floating_key");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Pedestal => "pedestal",
            Self::Chest => "chest",
            Self::FloatingKey => "floating_key",
        }
    }
}

impl Serialize for KeyHolderStyle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for KeyHolderStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "chest" => Self::Chest,
            "floating_key" => Self::FloatingKey,
            _ => Self::Pedestal,
        })
    }
}

/// Open-animation style used to render a door (`'D'`) cell.
///
/// Like [`EnemyType`] this is a per-cell *characteristic*, not a new cell type;
/// only the renderers read it. The wire form is the lowercase string
/// `"swing"`, `"slide"`, `"portcullis"` or `"dissolve"`; an unrecognised value
/// falls back to `Swing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DoorStyle {
    /// The default door rig (hinged swing).
    #[default]
    Swing,
    /// A door that slides aside.
    Slide,
    /// A portcullis that lifts.
    Portcullis,
    /// A door that dissolves away.
    Dissolve,
}

impl DoorStyle {
    /// Returns the lowercase wire string for this door rig.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::DoorStyle;
    /// assert_eq!(DoorStyle::Swing.as_wire_str(), "swing");
    /// assert_eq!(DoorStyle::Portcullis.as_wire_str(), "portcullis");
    /// ```
    pub fn as_wire_str(&self) -> &'static str {
        match self {
            Self::Swing => "swing",
            Self::Slide => "slide",
            Self::Portcullis => "portcullis",
            Self::Dissolve => "dissolve",
        }
    }
}

impl Serialize for DoorStyle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for DoorStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "slide" => Self::Slide,
            "portcullis" => Self::Portcullis,
            "dissolve" => Self::Dissolve,
            _ => Self::Swing,
        })
    }
}

/// Non-default characteristics for an enemy (`'E'`) cell. Every field is
/// optional: a `None` field means "inherit the per-game / built-in default".
/// `enemy_type` is read by the renderers; `damage` and `move_period_ms` are
/// applied by the engine.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyOverride {
    /// Visual rig for this enemy. Renderer-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enemy_type: Option<EnemyType>,
    /// Damage dealt on contact, overriding the per-game default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage: Option<u32>,
    /// How often this enemy advances one cell, in milliseconds, overriding the
    /// per-game default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_period_ms: Option<f32>,
}

impl EnemyOverride {
    fn is_empty(&self) -> bool {
        self.enemy_type.is_none() && self.damage.is_none() && self.move_period_ms.is_none()
    }
}

/// Non-default characteristics for a health-pickup (`'H'`) cell. Every field is
/// optional: a `None` field means "inherit the per-game / built-in default".
/// `health_style` is read by the renderers; `heal_amount` is applied by the
/// engine.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthOverride {
    /// Visual rig for this pickup. Renderer-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_style: Option<HealthStyle>,
    /// Hit points restored when consumed, overriding the built-in default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heal_amount: Option<u32>,
}

impl HealthOverride {
    fn is_empty(&self) -> bool {
        self.health_style.is_none() && self.heal_amount.is_none()
    }
}

/// Non-default characteristics for a key-holder (`'K'`) cell. `key_holder` is
/// read by the renderers; a `None` field means "inherit the per-game / built-in
/// default". (The key&harr;door pairing is a separate concern and is not modelled
/// here.)
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyOverride {
    /// Visual rig for this key holder. Renderer-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_holder: Option<KeyHolderStyle>,
}

impl KeyOverride {
    fn is_empty(&self) -> bool {
        self.key_holder.is_none()
    }
}

/// Non-default characteristics for a door (`'D'`) cell. `door_style` is read by
/// the renderers; a `None` field means "inherit the per-game / built-in
/// default". (The key&harr;door pairing is a separate concern and is not
/// modelled here.)
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorOverride {
    /// Visual open-animation rig for this door. Renderer-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub door_style: Option<DoorStyle>,
}

impl DoorOverride {
    fn is_empty(&self) -> bool {
        self.door_style.is_none()
    }
}

/// One entity occupying a cell, together with its (optional) override
/// characteristics. A cell holds a list of these (see
/// [`MazeDefinition::cell_entities`]); today that list is capped at one
/// element, but the wire form is an array so it can grow to hold several
/// co-located entities later with no format change.
///
/// This is also the on-the-wire entity type: it serialises as a flat object
/// tagged by `"type"` (`{ "type": "E", "enemyType": "ghost", "damage": 2 }`)
/// with only the set override fields present. Adding a new entity kind later is
/// a single new variant — the (de)serialiser, the type→field mapping, and the
/// per-variant validation all follow from the variant's own payload struct.
/// Unrecognised fields inside an entity are ignored (forward tolerance); an
/// unrecognised `"type"` is rejected.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CellEntity {
    /// Override for an enemy (`'E'`) cell.
    #[serde(rename = "E")]
    Enemy(EnemyOverride),
    /// Override for a health-pickup (`'H'`) cell.
    #[serde(rename = "H")]
    Health(HealthOverride),
    /// Override for a key-holder (`'K'`) cell.
    #[serde(rename = "K")]
    Key(KeyOverride),
    /// Override for a door (`'D'`) cell.
    #[serde(rename = "D")]
    Door(DoorOverride),
}

impl CellEntity {
    /// The grid character this override variant belongs on. An override is only
    /// meaningful on (and only serialised onto) a cell holding this character;
    /// a stale override left on a non-matching cell — e.g. after an in-place
    /// edit turned an `'E'` into a `'W'` — is dropped on serialisation rather
    /// than emitted as a malformed entity.
    fn cell_char(&self) -> char {
        match self {
            CellEntity::Enemy(_) => 'E',
            CellEntity::Health(_) => 'H',
            CellEntity::Key(_) => 'K',
            CellEntity::Door(_) => 'D',
        }
    }

    /// Whether the override sets no field at all. A field-less override carries
    /// no information and is normalised away on read (it would serialise back to
    /// a bare character anyway).
    fn is_empty(&self) -> bool {
        match self {
            CellEntity::Enemy(e) => e.is_empty(),
            CellEntity::Health(h) => h.is_empty(),
            CellEntity::Key(k) => k.is_empty(),
            CellEntity::Door(d) => d.is_empty(),
        }
    }

    /// Per-variant sanity check applied after deserialisation. Type-system
    /// guarantees (`u32` counts can't be negative) cover most fields; this
    /// catches the few that need a runtime range check.
    fn validate(&self) -> Result<(), &'static str> {
        if let CellEntity::Enemy(e) = self {
            if let Some(period) = e.move_period_ms {
                if !period.is_finite() || period < 0.0 {
                    return Err("enemy 'movePeriodMs' must be a non-negative finite number");
                }
            }
        }
        Ok(())
    }
}

/// A single grid cell as deserialised from the wire: its character plus an
/// optional parsed override.
struct WireCell {
    ch: char,
    over: Option<CellEntity>,
}

impl<'de> Deserialize<'de> for WireCell {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CellVisitor;

        impl<'de> Visitor<'de> for CellVisitor {
            type Value = WireCell;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a single-character string or an array containing one entity object")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<WireCell, E> {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(WireCell { ch: c, over: None }),
                    _ => Err(E::invalid_value(Unexpected::Str(value), &"a character")),
                }
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<WireCell, A::Error> {
                let first: Option<CellEntity> = seq.next_element()?;
                let over = match first {
                    Some(over) => over,
                    None => {
                        return Err(de::Error::custom(
                            "a cell entity array must contain exactly one entity (found 0)",
                        ))
                    }
                };
                // Length cap = 1. Drain any extra entities so the count in the
                // error reflects how many were actually supplied.
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    let mut count = 2;
                    while seq.next_element::<IgnoredAny>()?.is_some() {
                        count += 1;
                    }
                    return Err(de::Error::custom(format!(
                        "multiple entities per cell not yet supported (found {count})"
                    )));
                }
                over.validate().map_err(de::Error::custom)?;
                let ch = over.cell_char();
                // A field-less override carries no information; normalise it away
                // so it round-trips as a bare character (read tolerant, write
                // canonical).
                let over = (!over.is_empty()).then_some(over);
                Ok(WireCell { ch, over })
            }
        }

        deserializer.deserialize_any(CellVisitor)
    }
}

/// A single grid cell as it should be serialised: either the bare character or
/// an array of entity objects (one per override that belongs on the cell). Each
/// override serialises itself (via the `#[serde(tag = "type")]` derive on
/// [`CellEntity`]) as the flat tagged object form.
enum SerCell<'a> {
    Bare(char),
    Entities(Vec<&'a CellEntity>),
}

impl Serialize for SerCell<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SerCell::Bare(ch) => serializer.serialize_char(*ch),
            SerCell::Entities(overrides) => {
                let mut seq = serializer.serialize_seq(Some(overrides.len()))?;
                for over in overrides {
                    seq.serialize_element(over)?;
                }
                seq.end()
            }
        }
    }
}

impl Serialize for MazeDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Build the canonical wire grid: a cell emits its bare character unless
        // it carries one or more overrides whose type matches the cell, in
        // which case it emits the array form. Overrides whose type no longer
        // matches the cell character (e.g. left stale by an in-place edit) are
        // dropped so the output is always valid.
        let rows: Vec<Vec<SerCell>> = self
            .grid
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
                row.iter()
                    .enumerate()
                    .map(|(col_idx, &ch)| {
                        let matching: Vec<&CellEntity> = self
                            .cell_entities
                            .get(&(row_idx, col_idx))
                            .map(|overrides| {
                                overrides
                                    .iter()
                                    .filter(|over| over.cell_char() == ch)
                                    .collect()
                            })
                            .unwrap_or_default();
                        if matching.is_empty() {
                            SerCell::Bare(ch)
                        } else {
                            SerCell::Entities(matching)
                        }
                    })
                    .collect()
            })
            .collect();

        let mut state = serializer.serialize_struct("MazeDefinition", 1)?;
        state.serialize_field("grid", &rows)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for MazeDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, Vec<Vec<WireCell>>> = Deserialize::deserialize(deserializer)?;

        for key in map.keys() {
            if key != "grid" {
                return Err(serde::de::Error::unknown_field(key, &["grid"]));
            }
        }

        let wire_grid = match map.into_iter().find(|(key, _)| key == "grid") {
            Some((_, rows)) => rows,
            None => {
                return Err(serde::de::Error::missing_field("grid"));
            }
        };

        // Split the wire grid into a plain character grid plus a sparse override
        // map so downstream code keeps working on clean characters.
        let mut grid: Vec<Vec<char>> = Vec::with_capacity(wire_grid.len());
        let mut cell_entities: HashMap<(usize, usize), Vec<CellEntity>> = HashMap::new();
        for (row_idx, row) in wire_grid.into_iter().enumerate() {
            let mut char_row: Vec<char> = Vec::with_capacity(row.len());
            for (col_idx, cell) in row.into_iter().enumerate() {
                char_row.push(cell.ch);
                if let Some(over) = cell.over {
                    // The wire cap of one entity per cell means each overridden
                    // cell yields a single-element list; the list type leaves
                    // room for co-located entities without a format change.
                    cell_entities.insert((row_idx, col_idx), vec![over]);
                }
            }
            grid.push(char_row);
        }

        for row in &grid {
            for ch in row {
                if !Self::is_valid_char(*ch) {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Char(*ch),
                        &"valid characters are 'S', 'F', 'W', 'K', 'D', 'E', 'H' or ' '",
                    ));
                }
            }
        }

        if let Some(error) = Self::validate_grid(&grid) {
            return Err(de::Error::custom(error.to_string()));
        }

        Ok(MazeDefinition {
            grid,
            cell_entities,
        })
    }
}

impl MazeDefinition {
    // Public interface functions

    /// Creates a maze definition instance with the given number of rows x columns empty cells
    ///
    /// # Arguments
    /// * `row_count` - Number of rows
    /// * `col_count` - Number of columns
    ///
    /// # Returns
    ///
    /// A new maze definition instance
    ///
    /// # Examples
    ///
    /// Create a definition with 3 rows and 4 columns and then verify its dimensions
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.row_count(), 3);
    /// assert_eq!(definition.col_count(), 4);
    /// ```
    pub fn new(row_count: usize, col_count: usize) -> Self {
        MazeDefinition {
            grid: Self::alloc_empty_rows(row_count, col_count),
            cell_entities: HashMap::new(),
        }
    }
    /// Resets a maze definition instance to empty
    ///
    /// # Returns
    ///
    /// The maze definition instance
    ///
    /// # Examples
    ///
    /// Create a definition with 3 rows and 4 columns, verify its dimensions, reset it and
    /// then confirm it is empty
    /// ```
    /// use data_model::MazeDefinition;
    /// let mut definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.row_count(), 3);
    /// assert_eq!(definition.col_count(), 4);
    /// assert_eq!(definition.reset().is_empty(), true);
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.grid = vec![];
        self.cell_entities.clear();
        self
    }
    /// Resizes a maze definition instance
    ///
    /// # Arguments
    /// * `new_row_count` - New number of rows
    /// * `new_col_count` - New number of columns
    ///
    /// # Returns
    ///
    /// The maze definition instance
    ///
    /// # Examples
    ///
    /// Create an empty maze definition, resize it to 3 rows and 4 columns and then verify its dimensions
    /// ```
    /// use data_model::MazeDefinition;
    /// let mut definition = MazeDefinition::new(0, 0);
    /// assert_eq!(definition.row_count(), 0);
    /// assert_eq!(definition.col_count(), 0);
    /// definition.resize(3, 4);
    /// println!("Resize successful");
    /// assert_eq!(definition.row_count(), 3);
    /// assert_eq!(definition.col_count(), 4);
    ///
    /// ```
    pub fn resize(&mut self, new_row_count: usize, new_col_count: usize) -> &mut Self {
        for row in self.grid.iter_mut() {
            row.resize(new_col_count, ' ');
        }
        self.grid.resize(new_row_count, vec![' '; new_col_count]);
        self
    }
    /// Returns the number of rows associated with the definition instance
    ///
    /// # Returns
    ///
    /// Number of rows
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.row_count(), 3);
    /// ```
    pub fn row_count(&self) -> usize {
        self.grid.len()
    }
    /// Returns the number of columns associated with the definition instance
    ///
    /// # Returns
    ///
    /// Number of columns
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.col_count(), 4);
    /// ```
    pub fn col_count(&self) -> usize {
        Self::first_row_col_count(&self.grid)
    }
    /// Checks whether the definition instance is empty
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.row_count() == 0
    }
    /// Checks whether the given character is valid for use within the definition
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Print whether 'X' (`false`) and 'S' (`true`) are valid characters
    /// ```
    /// use data_model::MazeDefinition;
    /// let x_is_valid = MazeDefinition::is_valid_char('X');
    /// println!("Character 'X' is valid => {}", x_is_valid);
    /// let s_is_valid = MazeDefinition::is_valid_char('S');
    /// println!("Character 'S' is valid => {}", s_is_valid);
    /// ```
    pub fn is_valid_char(ch: char) -> bool {
        matches!(ch, 'S' | 'F' | 'W' | 'K' | 'D' | 'E' | 'H' | ' ')
    }
    /// Verifies whether the definition instance is empty, returning an error if it is
    ///
    /// # Returns
    ///
    /// This function will return an error if the definition is empty
    ///
    /// # Examples
    ///
    /// Create an empty maze definition and then verify it
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(0, 0);
    /// match definition.verify_not_empty() {
    ///     Err(e) => println!("Verification failed: {}", e.to_string()),
    ///     Ok(()) => println!("MazeDefinition is not empty"),
    /// }
    /// ```
    pub fn verify_not_empty(&self) -> Result<(), Error> {
        if self.is_empty() {
            return Err(Error::MazeValidation("definition is empty".to_string()));
        }
        Ok(())
    }
    /// Creates a new maze definition for the given vector of cell definition character rows, where:
    /// - `'S'`:  Represents the starting cell (limited to one).
    /// - `'F'`:  Represents the finishing cell (limited to one).
    /// - `'W'`:  Represents a wall.
    /// - `' '`:  Represents an empty cell.
    /// - `'K'`:  Represents a cell holding a key (multiple allowed).
    /// - `'D'`:  Represents a door (multiple allowed).
    /// - `'E'`:  Represents an enemy spawn cell (multiple allowed).
    /// - `'H'`:  Represents a health-pickup cell (multiple allowed).
    ///
    /// # Arguments
    ///
    /// * `grid` - Vector of row-column cell states
    ///
    /// # Returns
    ///
    /// A new definition instance
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let definition = MazeDefinition::from_vec(grid);
    /// assert_eq!(definition.row_count(), 2);
    /// assert_eq!(definition.col_count(), 3);
    /// ```
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        if let Some(error) = Self::validate_grid(&grid) {
            panic!("{}", error.to_string());
        }
        MazeDefinition {
            grid,
            cell_entities: HashMap::new(),
        }
    }
    /// Converts the definition instance to a vector of row cell states
    ///
    /// # Returns
    ///
    /// A vector of row-column cell states
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns, convert it to a row-column state vector and then confirm that
    /// the number of rows in the state vector is the same as the number of rows in the definition (3).
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// let state = definition.to_state();
    /// assert_eq!(state.len(), definition.row_count());
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn to_state(&self) -> Vec<Vec<MazeCellState>> {
        self.grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'W' => MazeCellState::Wall,
                        // `K` (key), `D` (door), `E` (enemy spawn) and `H` (health
                        // pickup) are passable terrain at the cell-state level;
                        // their gameplay semantics live in the `maze` crate. The
                        // solver therefore treats doors as openable and is enemy-
                        // blind.
                        'S' | 'F' | ' ' | 'K' | 'D' | 'E' | 'H' => MazeCellState::Empty,
                        _ => panic!(
                            "internal error - grid contains unsupported cell character: {value}"
                        ),
                    })
                    .collect::<Vec<MazeCellState>>()
            })
            .collect()
    }
    /// Checks that a point is valid for the definition instance
    ///
    /// # Arguments
    ///
    /// * `pt` - MazePoint to validate
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns and confirm that `[2,1]` is valid, but that `[3,1]` is not
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// use data_model::MazePoint;
    /// let definition = MazeDefinition::new(3, 4);
    /// assert_eq!(definition.is_valid( &MazePoint {row: 2, col: 1}), true);
    /// assert_eq!(definition.is_valid( &MazePoint {row: 3, col: 1}), false);
    /// ```
    pub fn is_valid(&self, pt: &MazePoint) -> bool {
        if pt.row >= self.row_count() || pt.col >= self.col_count() {
            return false;
        }
        true
    }
    /// Converts the definition instance to a vector of display characters
    ///
    /// # Returns
    ///
    /// Vector containing the rows of display characters
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns and print it
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let definition = MazeDefinition::new(3, 4);
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn to_display_chars(&self) -> Vec<Vec<char>> {
        self.grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'S' => 'S',
                        'F' => 'F',
                        'W' => '\u{2588}',
                        'K' => 'K',
                        'D' => 'D',
                        'E' => 'E',
                        'H' => 'H',
                        ' ' => '\u{2591}',
                        _ => '-',
                    })
                    .collect::<Vec<char>>()
            })
            .collect()
    }
    /// Deletes one or more consecutive columns from the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 2 rows and 4 columns with a start, finish and a wall at the end of each row,
    /// delete the second and third columns and print the result
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', ' ', 'W'],
    ///    vec![' ', 'F', ' ', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// definition.delete_cols(1,2).expect("delete_cols() failed");
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn delete_cols(&mut self, start_col: usize, count: usize) -> Result<(), Error> {
        self.verify_not_empty()?;
        if start_col >= self.col_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'start_col' index ({start_col})"
            )));
        }
        if start_col + count > self.col_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'count' ({count}) - too large"
            )));
        }
        for row in &mut self.grid {
            row.drain(start_col..(start_col + count));
        }
        Ok(())
    }
    /// Inserts one or more empty columns into the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 2 rows and 4 columns, with a start, finish and a wall at
    /// the end of each row, insert 2 columns at the start of each row and print the result
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', ' ', 'W'],
    ///    vec![' ', 'F', ' ', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// definition.insert_cols(0,2).expect("insert_cols() failed");
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn insert_cols(&mut self, start_col: usize, count: usize) -> Result<(), Error> {
        self.verify_not_empty()?;
        if start_col > self.col_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'start_col' index ({start_col})"
            )));
        }
        for row in &mut self.grid {
            row.splice(start_col..start_col, vec![' '; count]);
        }
        Ok(())
    }
    /// Deletes one or more consecutive rows from the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target rows are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, delete the first and and second rows and print the result
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W'],
    ///    vec![' ', ' ', 'W', 'W'],
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// definition.delete_rows(1,2).expect("delete_rows() failed");
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn delete_rows(&mut self, start_row: usize, count: usize) -> Result<(), Error> {
        self.verify_not_empty()?;
        if start_row >= self.row_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'start_row' index ({start_row})"
            )));
        }
        if start_row + count > self.row_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'count' ({count}) - too large"
            )));
        }
        self.grid.drain(start_row..(start_row + count));
        Ok(())
    }
    /// Inserts one or more empty rows into the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target rows are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, insert 2 rows after the fourth row and print the result
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W'],
    ///    vec![' ', ' ', 'W', 'W'],
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// definition.insert_rows(3,2).expect("insert_rows() failed");
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn insert_rows(&mut self, start_row: usize, count: usize) -> Result<(), Error> {
        if start_row > self.row_count() {
            return Err(Error::MazeValidation(format!(
                "invalid 'start_row' index ({start_row})"
            )));
        }
        if count > 0 {
            let empty_rows = Self::alloc_empty_rows(count, self.col_count());
            self.grid.splice(start_row..start_row, empty_rows);
        }
        Ok(())
    }
    /// Locates the starting position within the maze definition (if any)
    ///
    /// # Returns
    ///
    /// The starting position, else none
    ///
    /// # Examples
    ///
    /// Locate the starting position in a 2 row x 3 column definition
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let definition = MazeDefinition::from_vec(grid);
    /// match definition.get_start() {
    ///     Some(start) => {
    ///         println!("Start found at point {}", start);
    ///     },
    ///     None => {
    ///         println!("Start not found");
    ///     }
    /// };
    /// ```
    pub fn get_start(&self) -> Option<MazePoint> {
        self.find_first_char('S')
    }
    /// Sets the starting position within the maze definition (if any)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the new starting position is out of range
    ///
    /// # Examples
    ///
    /// Reset the starting position in a 2 row x 3 column definition
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// use data_model::MazePoint;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// let new_start = MazePoint {row: 1, col: 2};
    /// definition.set_start(Some(new_start)).expect("set_start() failed");
    /// ```
    pub fn set_start(&mut self, new_start: Option<MazePoint>) -> Result<(), Error> {
        self.reset_point("start", self.get_start(), new_start, 'S')
    }
    /// Locates the finishing position within the maze definition (if any)
    ///
    /// # Returns
    ///
    /// The finishing position, else none
    ///
    /// # Examples
    ///
    /// Locate the finishing position in a 2 row x 3 column definition
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let definition = MazeDefinition::from_vec(grid);
    /// match definition.get_finish() {
    ///     Some(finish) => {
    ///         println!("Finish found at point {}", finish);
    ///     },
    ///     None => {
    ///         println!("Finish not found");
    ///     }
    /// };
    /// ```
    pub fn get_finish(&self) -> Option<MazePoint> {
        self.find_first_char('F')
    }
    /// Sets the finishing position within the maze definition (if any)
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the new finishing position is out of range
    ///
    /// # Examples
    ///
    /// Reset the finishing position in a 2 row x 3 column definition
    ///
    /// ```
    /// use data_model::MazeDefinition;
    /// use data_model::MazePoint;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut definition = MazeDefinition::from_vec(grid);
    /// let new_finish = MazePoint {row: 0, col: 2};
    /// definition.set_start(Some(new_finish)).expect("new_finish() failed");
    /// ```
    pub fn set_finish(&mut self, new_finish: Option<MazePoint>) -> Result<(), Error> {
        self.reset_point("finish", self.get_finish(), new_finish, 'F')
    }
    /// Modify the value of each cell in a given region of the definition instance
    /// # Arguments
    ///
    /// * `from` - Starting point of cell region to modify
    /// * `to` - Ending point of cell region to modify
    /// * `value` - Value to set. Must be one of `'W'` (wall), `'K'` (key),
    ///   `'D'` (door), `'E'` (enemy spawn), `'H'` (health pickup), or `' '` (empty).
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target points are out of range
    /// - if the character value is invalid
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, then set the central region (1,1) to (3, 2) to be a wall and then print it
    ///
    ///
    /// ```
    /// use data_model::MazeCellState;
    /// use data_model::MazeDefinition;
    /// use data_model::MazePoint;
    /// let mut definition = MazeDefinition::new(5, 4);
    /// let from = MazePoint { row: 1, col: 1, };
    /// let to = MazePoint { row: 3, col: 2, };
    /// definition.set_value( from, to, 'W').expect("set_value() failed");
    /// println!("{:?}", definition.to_display_chars());
    /// ```
    pub fn set_value(&mut self, from: MazePoint, to: MazePoint, value: char) -> Result<(), Error> {
        if !self.is_valid(&from) {
            return Err(Error::MazeValidation(format!("invalid 'from' point {from}")));
        }
        if !self.is_valid(&to) {
            return Err(Error::MazeValidation(format!("invalid 'to' point {to}")));
        }
        match value {
            'W' | 'K' | 'D' | 'E' | 'H' | ' ' => {
                let top_row = from.row.min(to.row);
                let bottom_row = from.row.max(to.row);
                let left_col = from.col.min(to.col);
                let right_col = from.col.max(to.col);
                for row_idx in top_row..(bottom_row + 1) {
                    for col_idx in left_col..(right_col + 1) {
                        self.grid[row_idx][col_idx] = value;
                    }
                }
            }
            _ => return Err(Error::MazeValidation(format!("invalid 'value' ('{value}')"))),
        }
        Ok(())
    }

    // Private helper functions

    fn first_row_col_count(grid: &[Vec<char>]) -> usize {
        grid.first().map_or(0, |inner_vec| inner_vec.len())
    }

    fn validate_grid(grid: &[Vec<char>]) -> Option<Error> {
        let first_row_col_count = Self::first_row_col_count(grid);
        let same_col_counts = grid
            .iter()
            .all(|inner_vec| inner_vec.len() == first_row_col_count);
        if !same_col_counts {
            let msg = format!("grid vector contains rows with different numbers of columns (expected {first_row_col_count} for all rows)").clone();
            return Some(Error::MazeValidation(msg));
        }
        let mut num_starts = 0;
        let mut num_finishes = 0;
        for (row_idx, row) in grid.iter().enumerate() {
            for (col_idx, &item) in row.iter().enumerate() {
                if !Self::is_valid_char(item) {
                    let msg = format!(
                        "grid vector contains an invalid character '{}' at location {}",
                        item,
                        MazePoint {
                            row: row_idx,
                            col: col_idx
                        }
                    );
                    return Some(Error::MazeValidation(msg));
                } else if item == 'S' {
                    num_starts += 1;
                    if num_starts > 1 {
                        return Some(Error::MazeValidation("too many start characters `S`".to_string()));
                    }
                } else if item == 'F' {
                    num_finishes += 1;
                    if num_finishes > 1 {
                        return Some(Error::MazeValidation("too many finish characters `F`".to_string()));
                    }
                }
            }
        }
        None
    }

    fn alloc_empty_rows(row_count: usize, col_count: usize) -> Vec<Vec<char>> {
        vec![vec![' '; col_count]; row_count]
    }

    fn find_first_char(&self, target: char) -> Option<MazePoint> {
        for (i, row) in self.grid.iter().enumerate() {
            for (j, &ch) in row.iter().enumerate() {
                if ch == target {
                    return Some(MazePoint { row: i, col: j });
                }
            }
        }
        None
    }

    fn reset_point(
        &mut self,
        name: &str,
        current: Option<MazePoint>,
        new: Option<MazePoint>,
        ch: char,
    ) -> Result<(), Error> {
        if let Some(new_pt) = new {
            if !self.is_valid(&new_pt) {
                return Err(Error::MazeValidation(format!(
                    "invalid '{name}' point {new_pt}"
                )));
            }
            if let Some(current_pt) = current {
                self.grid[current_pt.row][current_pt.col] = ' ';
            }
            self.grid[new_pt.row][new_pt.col] = ch;
        } else if let Some(current_pt) = self.get_start() {
            self.grid[current_pt.row][current_pt.col] = ' ';
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;    

    #[test]
    fn can_create_empty_from_dimensions() {
        let definition = MazeDefinition::new(0, 0);
        assert_eq!(definition.row_count(), 0);
        assert_eq!(definition.col_count(), 0);
    }

    #[test]
    fn can_create_new_from_dimensions() {
        let definition = MazeDefinition::new(2, 3);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
    }

    #[test]
    fn can_reset_to_empty() {
        let mut definition = MazeDefinition::new(2, 3);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
        assert!(!definition.is_empty());
        assert!(definition.reset().is_empty())
    }

    #[test]
    fn can_create_empty_from_vector() {
        let grid: Vec<Vec<char>> = vec![];
        let definition = MazeDefinition::from_vec(grid);
        assert_eq!(definition.row_count(), 0);
        assert_eq!(definition.col_count(), 0);
    }

    #[test]
    fn can_create_new_from_vector() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let definition = MazeDefinition::from_vec(grid);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
    }

    #[test]
    fn is_valid_char_accepts_keys_doors_enemies_health() {
        assert!(MazeDefinition::is_valid_char('K'));
        assert!(MazeDefinition::is_valid_char('D'));
        assert!(MazeDefinition::is_valid_char('E'));
        assert!(MazeDefinition::is_valid_char('H'));
    }

    #[test]
    fn can_create_new_from_vector_with_multiple_keys_and_doors() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'K', 'D', ' '],
            vec!['K', 'D', ' ', 'F']
        ];
        let definition = MazeDefinition::from_vec(grid.clone());
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 4);
        assert_eq!(definition.grid, grid);
    }

    #[test]
    fn can_create_new_from_vector_with_multiple_enemies_and_health() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'E', 'H', ' '],
            vec!['E', 'H', ' ', 'F']
        ];
        let definition = MazeDefinition::from_vec(grid.clone());
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 4);
        assert_eq!(definition.grid, grid);
    }

    #[test]
    fn can_deserialize_with_keys_and_doors() {
        let s = r#"{"grid":[["S","K","D"," "],["K","D"," ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        let grid: Vec<Vec<char>> =
            vec![vec!['S', 'K', 'D', ' '], vec!['K', 'D', ' ', 'F']];
        assert_eq!(d.grid, grid);
    }

    #[test]
    fn can_deserialize_with_enemies_and_health() {
        let s = r#"{"grid":[["S","E","H"," "],["E","H"," ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        let grid: Vec<Vec<char>> =
            vec![vec!['S', 'E', 'H', ' '], vec!['E', 'H', ' ', 'F']];
        assert_eq!(d.grid, grid);
    }

    #[test]
    #[should_panic(expected = "grid vector contains an invalid character 'X' at location [1, 2]")]
    fn cannot_create_new_from_vector_with_invalid_char() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', 'X']
        ];
        let _definition = MazeDefinition::from_vec(grid);
    }

    #[test]
    #[should_panic(expected = "too many start characters `S`")]
    fn cannot_create_new_from_vector_with_too_many_start_chars() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', ' '],
            vec!['S', ' ', ' ']
        ];
        let _definition = MazeDefinition::from_vec(grid);
    }

    #[test]
    #[should_panic(expected = "too many finish characters `F`")]
    fn cannot_create_new_from_vector_with_too_many_finish_chars() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'F'],
            vec!['F', ' ', ' ']
        ];
        let _definition = MazeDefinition::from_vec(grid);
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_create_new_from_vector_with_diff_row_counts() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ']
        ];
        let _definition = MazeDefinition::from_vec(grid);
    }

    #[test]
    fn can_confirm_empty() {
        let definition = MazeDefinition::new(0, 0);
        assert!(definition.is_empty());
    }

    #[test]
    fn can_confirm_not_empty() {
        let definition = MazeDefinition::new(1, 1);
        assert!(!definition.is_empty());
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn confirm_verify_not_empty_detects_empty() {
        let definition = MazeDefinition::new(0, 0);
        if let Err(error) = definition.verify_not_empty() {
            panic!("{}", error.to_string());
        }
        panic!("verify_not_empty() did not return an error");
    }

    #[test]
    fn confirm_verify_not_empty_ignores_non_empty() {
        let definition = MazeDefinition::new(1, 1);
        if let Err(error) = definition.verify_not_empty() {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    fn can_resize_empty_to_empty() {
        let mut definition = MazeDefinition::new(0, 0);
        definition.resize(0, 0);
        assert_eq!(definition.row_count(), 0);
        assert_eq!(definition.col_count(), 0);
    }

    #[test]
    fn can_resize_to_empty() {
        let mut definition = MazeDefinition::new(10, 5);
        definition.resize(0, 0);
        assert_eq!(definition.row_count(), 0);
        assert_eq!(definition.col_count(), 0);
    }

    #[test]
    fn can_expand_with_resize() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['W', ' ', ' '],
            vec![' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
        definition.resize(3, 5);
        assert_eq!(definition.row_count(), 3);
        assert_eq!(definition.col_count(), 5);
        #[rustfmt::skip]
        let grid_check: Vec<Vec<char>> = vec![
            vec!['W', ' ', ' ', ' ', ' '],
            vec![' ', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', ' ', ' ']
        ];
        assert_eq!(definition.grid, grid_check);
    }

    #[test]
    fn can_shrink_with_resize_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['W', ' ', ' '],
            vec![' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
        definition.resize(2, 1);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 1);
        #[rustfmt::skip]
        let grid_check: Vec<Vec<char>> = vec![
            vec!['W'],
            vec![' ']
        ];
        assert_eq!(definition.grid, grid_check);
    }

    #[test]
    fn can_shrink_with_resize_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['W', ' ', ' '],
            vec![' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        assert_eq!(definition.row_count(), 2);
        assert_eq!(definition.col_count(), 3);
        definition.resize(1, 2);
        assert_eq!(definition.row_count(), 1);
        assert_eq!(definition.col_count(), 2);
        #[rustfmt::skip]
        let grid_check: Vec<Vec<char>> = vec![
            vec!['W', ' ']
        ];
        assert_eq!(definition.grid, grid_check);
    }

    #[test]
    fn can_serialize_empty_1() {
        let definition = MazeDefinition::new(0, 0);
        let s = serde_json::to_string(&definition).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[]}"#);
    }

    #[test]
    fn can_serialize_empty_2() {
        let grid: Vec<Vec<char>> = vec![];
        let definition = MazeDefinition::from_vec(grid);
        let s = serde_json::to_string(&definition).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[]}"#);
    }

    #[test]
    fn can_serialize_non_empty_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let definition = MazeDefinition::from_vec(grid);
        let s = serde_json::to_string(&definition).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[[" "," "," "],[" "," "," "]]}"#);
    }

    #[test]
    fn can_serialize_non_empty_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' '],
            vec![' ', ' ', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        let s = serde_json::to_string(&definition).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[[" ","W"," "],[" "," ","W"]]}"#);
    }

    #[test]
    fn can_serialize_non_empty_3() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'W', ' '],
            vec![' ', 'F', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        let s = serde_json::to_string(&definition).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[["S","W"," "],[" ","F","W"]]}"#);
    }

    #[test]
    fn can_deserialize_empty() {
        let s = r#"{"grid":[]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.row_count(), 0);
        assert_eq!(d.col_count(), 0);
    }

    #[test]
    fn can_deserialize_non_empty() {
        let s = r#"{"grid":[["S","W"," "],["F"," ","W"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.row_count(), 2);
        assert_eq!(d.col_count(), 3);
        let grid: Vec<Vec<char>> = vec![vec!['S', 'W', ' '], vec!['F', ' ', 'W']];
        assert_eq!(d.grid, grid);
    }

    #[test]
    #[should_panic(expected = "EOF while parsing an object")]
    fn cannot_deserialize_bad_json_format_incomplete_object() {
        let s = "{";
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "expected value")]
    fn cannot_deserialize_bad_json_format_no_open_object() {
        let s = "}";
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "expected value")]
    fn cannot_deserialize_bad_json_format_missing_field_value() {
        let s = r#"{"grid":}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "EOF while parsing a string")]
    fn cannot_deserialize_bad_json_format_field_name_not_closed() {
        let s = r#"{"grid:}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "key must be a string")]
    fn cannot_deserialize_bad_json_format_field_name_not_quoted() {
        let s = "{grid:}";
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = r#"invalid type: string \"a\", expected a sequence"#)]
    fn cannot_deserialize_json_with_non_vec_grid_value() {
        let s = r#"{"grid":"a"}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "missing field `grid`")]
    fn cannot_deserialize_json_missing_grid_field() {
        let s = "{}";
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "unknown field `grid2`")]
    fn cannot_deserialize_json_with_invalid_field_name() {
        let s = r#"{"grid2":[[" ","W"," "],[" "," ","W"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(
        expected = "invalid value: character `X`, expected valid characters are 'S', 'F', 'W', 'K', 'D', 'E', 'H' or ' '"
    )]
    fn cannot_deserialize_bad_json_invalid_char_1() {
        let s = r#"{"grid":[["S","X"," "],["F"," ","W"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = r#"invalid value: string \"XX\", expected a character"#)]
    fn cannot_deserialize_bad_json_invalid_char_2() {
        let s = r#"{"grid":[["S","XX"," "],["F"," ","W"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "too many start characters `S`")]
    fn cannot_deserialize_bad_json_more_than_one_start_char() {
        let s = r#"{"grid":[["S"," "," "],["F","S","W"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "too many finish characters `F`")]
    fn cannot_deserialize_bad_json_more_than_one_finish_char() {
        let s = r#"{"grid":[["S"," "," "],["F","F","W"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_deserialize_bad_json_with_different_col_counts() {
        let s = r#"{"grid":[[" "," "," "],[" "," "]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_delete_cols_if_empty() {
        let mut definition = MazeDefinition::new(0, 0);
        definition.delete_cols(0, 1).expect("delete_cols() failed");
    }

    #[test]
    fn can_delete_valid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_cols(1, 2).expect("delete_cols() failed");
        assert_eq!(definition.col_count(), 2);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_col' index (4)")]
    fn cannot_delete_invalid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_cols(4, 2).expect("delete_cols() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'count' (3) - too large")]
    fn cannot_delete_too_many_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_cols(2, 3).expect("delete_cols() failed");
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_insert_cols_if_empty() {
        let mut definition = MazeDefinition::new(0, 0);
        definition.insert_cols(0, 1).expect("insert_cols() failed");
        assert_empty_cols(&definition, 0, 1);
    }

    #[test]
    fn can_insert_valid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_cols(1, 2).expect("insert_cols() failed");
        assert_eq!(definition.col_count(), 6);
        assert_empty_cols(&definition, 1, 2);
    }

    #[test]
    fn can_insert_no_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_cols(1, 0).expect("insert_cols() failed");
        assert_eq!(definition.col_count(), 4);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_col' index (5)")]
    fn cannot_insert_invalid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_cols(5, 2).expect("insert_cols() failed");
    }

    #[test]
    fn can_append_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_cols(4, 2).expect("insert_cols() failed");
        assert_eq!(definition.col_count(), 6);
        assert_empty_cols(&definition, 4, 5);
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_delete_rows_if_empty() {
        let mut definition = MazeDefinition::new(0, 0);
        definition.delete_rows(0, 1).expect("delete_rows() failed");
    }

    #[test]
    fn can_delete_valid_rows_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_rows(0, 2).expect("delete_rows() failed");
        assert_eq!(definition.row_count(), 1);
        assert_eq!(definition.col_count(), 4);
    }

    #[test]
    fn can_delete_valid_rows_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_rows(0, 3).expect("delete_rows() failed");
        assert_eq!(definition.row_count(), 0);
        assert_eq!(definition.col_count(), 0);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_row' index (2)")]
    fn cannot_delete_invalid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_rows(2, 1).expect("delete_rows() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'count' (2) - too large")]
    fn cannot_delete_too_many_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.delete_rows(1, 2).expect("delete_rows() failed");
    }

    #[test]
    fn can_insert_valid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_rows(1, 3).expect("insert_rows() failed");
        assert_eq!(definition.row_count(), 5);
        assert_empty_rows(&definition, 1, 3);
    }

    #[test]
    fn can_insert_no_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_rows(1, 0).expect("insert_rows() failed");
        assert_eq!(definition.row_count(), 2);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_row' index (3)")]
    fn cannot_insert_invalid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_rows(3, 2).expect("insert_rows() failed");
    }

    #[test]
    fn can_append_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition.insert_rows(2, 2).expect("insert_rows() failed");
        assert_eq!(definition.row_count(), 4);
        assert_empty_rows(&definition, 2, 3);
    }

    #[test]
    fn should_find_start() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', 'S', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        match definition.get_start() {
            Some(start) => {
                assert_eq!(start, MazePoint { row: 1, col: 2 });
            }
            None => {
                panic!("Failed to find start")
            }
        };
    }

    #[test]
    fn should_not_find_start() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        if let Some(start) = definition.get_start() {
            panic!("Unexpectedly found start at {start}");
        };
    }

    #[test]
    fn should_reset_start() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', 'S', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        match definition.set_start(Some(MazePoint { row: 1, col: 0 })) {
            Err(error) => {
                panic!("Failed to reset start: {error}");
            }
            _ => {
                let new_start = definition.get_start().unwrap();
                assert_eq!(new_start, MazePoint { row: 1, col: 0 });
            }
        }
    }

    #[test]
    #[should_panic(expected = "invalid 'start' point [1, 8]")]
    fn should_not_reset_start() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', 'S', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition
            .set_start(Some(MazePoint { row: 1, col: 8 }))
            .expect("set_start() failed");
    }

    #[test]
    fn should_find_finish() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'F', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        match definition.get_finish() {
            Some(finish) => {
                assert_eq!(finish, MazePoint { row: 0, col: 2 });
            }
            None => {
                panic!("Failed to find finish")
            }
        };
    }

    #[test]
    fn should_not_find_finish() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let definition = MazeDefinition::from_vec(grid);
        if let Some(finish) = definition.get_finish() {
            panic!("Unexpectedly found finish at {finish}");
        };
    }

    #[test]
    fn should_reset_finish() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', 'F', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        match definition.set_finish(Some(MazePoint { row: 0, col: 1 })) {
            Err(error) => {
                panic!("Failed to reset finish: {error}");
            }
            _ => {
                let new_finish = definition.get_finish().unwrap();
                assert_eq!(new_finish, MazePoint { row: 0, col: 1 });
            }
        }
    }

    #[test]
    #[should_panic(expected = "invalid 'finish' point [1, 8]")]
    fn should_not_reset_finish() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', 'F', 'W']
        ];
        let mut definition = MazeDefinition::from_vec(grid);
        definition
            .set_finish(Some(MazePoint { row: 1, col: 8 }))
            .expect("set_finish() failed");
    }

    #[test]
    fn can_set_value_valid_range() {
        let mut definition = MazeDefinition::new(5, 4);
        let from = MazePoint { row: 1, col: 1 };
        let to = MazePoint { row: 3, col: 2 };
        definition
            .set_value(from.clone(), to.clone(), 'W')
            .expect("set_value() failed");
        assert_cell_value(&definition, from.clone(), to.clone(), 'W');
    }

    #[test]
    fn can_set_value_enemy_and_health() {
        let mut definition = MazeDefinition::new(5, 4);
        let enemy_from = MazePoint { row: 1, col: 1 };
        let enemy_to = MazePoint { row: 1, col: 1 };
        definition
            .set_value(enemy_from.clone(), enemy_to.clone(), 'E')
            .expect("set_value('E') failed");
        assert_cell_value(&definition, enemy_from.clone(), enemy_to.clone(), 'E');

        let health_from = MazePoint { row: 2, col: 2 };
        let health_to = MazePoint { row: 2, col: 2 };
        definition
            .set_value(health_from.clone(), health_to.clone(), 'H')
            .expect("set_value('H') failed");
        assert_cell_value(&definition, health_from.clone(), health_to.clone(), 'H');
    }

    #[test]
    #[should_panic(expected = "invalid 'from' point [6, 1]")]
    fn cannot_set_value_invalid_from() {
        let mut definition = MazeDefinition::new(5, 4);
        let from = MazePoint { row: 6, col: 1 };
        let to = MazePoint { row: 2, col: 2 };
        definition
            .set_value(from, to, 'W')
            .expect("set_value() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'to' point [6, 2]")]
    fn cannot_set_value_invalid_to() {
        let mut definition = MazeDefinition::new(5, 4);
        let from = MazePoint { row: 1, col: 1 };
        let to = MazePoint { row: 6, col: 2 };
        definition
            .set_value(from, to, 'W')
            .expect("set_value() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'value' ('X')")]
    fn cannot_set_value_invalid_value() {
        let mut definition = MazeDefinition::new(5, 4);
        let from = MazePoint { row: 1, col: 1 };
        let to = MazePoint { row: 3, col: 2 };
        definition
            .set_value(from, to, 'X')
            .expect("set_value() failed");
    }

    // Per-cell override (de)serialisation tests

    /// Returns the single override on a cell, asserting the per-cell list holds
    /// exactly one element (the current cap).
    fn single_override(d: &MazeDefinition, row: usize, col: usize) -> &CellEntity {
        let overrides = d
            .cell_entities
            .get(&(row, col))
            .unwrap_or_else(|| panic!("expected an override at ({row}, {col})"));
        assert_eq!(
            overrides.len(),
            1,
            "every per-cell override list holds exactly one element for now"
        );
        &overrides[0]
    }

    #[test]
    fn default_maze_round_trips_byte_identical() {
        // An all-default maze (no overrides) must serialise byte-for-byte as a
        // plain character grid — no array-of-one cells anywhere.
        let s = r#"{"grid":[["S","E","H"," "],["E","H"," ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert!(d.cell_entities.is_empty());
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, s);
    }

    #[test]
    fn enemy_override_round_trips() {
        let s = r#"{"grid":[["S",[{"type":"E","enemyType":"ghost","damage":2,"movePeriodMs":900.0}]],[" ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'E');
        match single_override(&d, 0, 1) {
            CellEntity::Enemy(e) => {
                assert_eq!(e.enemy_type, Some(EnemyType::Ghost));
                assert_eq!(e.damage, Some(2));
                assert_eq!(e.move_period_ms, Some(900.0));
            }
            other => panic!("expected an enemy override, got {other:?}"),
        }
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, s);
    }

    #[test]
    fn health_override_round_trips() {
        let s =
            r#"{"grid":[["S",[{"type":"H","healthStyle":"potion","healAmount":3}]],[" ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'H');
        match single_override(&d, 0, 1) {
            CellEntity::Health(h) => {
                assert_eq!(h.health_style, Some(HealthStyle::Potion));
                assert_eq!(h.heal_amount, Some(3));
            }
            other => panic!("expected a health override, got {other:?}"),
        }
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, s);
    }

    #[test]
    fn key_override_round_trips() {
        let s = r#"{"grid":[["S",[{"type":"K","keyHolder":"chest"}]],["D","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'K');
        match single_override(&d, 0, 1) {
            CellEntity::Key(k) => assert_eq!(k.key_holder, Some(KeyHolderStyle::Chest)),
            other => panic!("expected a key override, got {other:?}"),
        }
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, s);
    }

    #[test]
    fn door_override_round_trips() {
        let s = r#"{"grid":[["S",[{"type":"D","doorStyle":"portcullis"}]],["K","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'D');
        match single_override(&d, 0, 1) {
            CellEntity::Door(door) => assert_eq!(door.door_style, Some(DoorStyle::Portcullis)),
            other => panic!("expected a door override, got {other:?}"),
        }
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, s);
    }

    #[test]
    fn override_less_array_entity_normalises_to_bare_char() {
        // An array entity that sets no field is accepted on read but written
        // back as a bare character (read is tolerant, write is canonical).
        let s = r#"{"grid":[["S",[{"type":"E"}]],[" ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'E');
        assert!(d.cell_entities.is_empty());
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, r#"{"grid":[["S","E"],[" ","F"]]}"#);
    }

    #[test]
    fn field_less_key_and_door_array_forms_normalise_to_bare_chars() {
        // A key or door array entity that sets no field carries no override and
        // normalises back to a bare character on write.
        let s = r#"{"grid":[["S",[{"type":"K"}]],[[{"type":"D"}],"F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        assert_eq!(d.grid[0][1], 'K');
        assert_eq!(d.grid[1][0], 'D');
        assert!(d.cell_entities.is_empty());
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, r#"{"grid":[["S","K"],["D","F"]]}"#);
    }

    #[test]
    fn unknown_override_fields_are_ignored() {
        let s = r#"{"grid":[["S",[{"type":"E","damage":2,"speedBoost":true,"foo":"bar"}]],[" ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        match single_override(&d, 0, 1) {
            CellEntity::Enemy(e) => {
                assert_eq!(e.damage, Some(2));
                assert!(e.enemy_type.is_none());
                assert!(e.move_period_ms.is_none());
            }
            other => panic!("expected an enemy override, got {other:?}"),
        }
    }

    #[test]
    fn unknown_enemy_type_falls_back_to_goblin() {
        let s = r#"{"grid":[["S",[{"type":"E","enemyType":"dragon"}]],[" ","F"]]}"#;
        let d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
        match single_override(&d, 0, 1) {
            CellEntity::Enemy(e) => assert_eq!(e.enemy_type, Some(EnemyType::Goblin)),
            other => panic!("expected an enemy override, got {other:?}"),
        }
    }

    #[test]
    fn override_enum_wire_strings_match() {
        assert_eq!(EnemyType::Goblin.as_wire_str(), "goblin");
        assert_eq!(EnemyType::Ghost.as_wire_str(), "ghost");
        assert_eq!(HealthStyle::Heart.as_wire_str(), "heart");
        assert_eq!(HealthStyle::Potion.as_wire_str(), "potion");
        assert_eq!(KeyHolderStyle::Pedestal.as_wire_str(), "pedestal");
        assert_eq!(KeyHolderStyle::Chest.as_wire_str(), "chest");
        assert_eq!(KeyHolderStyle::FloatingKey.as_wire_str(), "floating_key");
        assert_eq!(DoorStyle::Swing.as_wire_str(), "swing");
        assert_eq!(DoorStyle::Slide.as_wire_str(), "slide");
        assert_eq!(DoorStyle::Portcullis.as_wire_str(), "portcullis");
        assert_eq!(DoorStyle::Dissolve.as_wire_str(), "dissolve");
    }

    #[test]
    fn stale_override_on_non_matching_char_is_dropped_on_serialise() {
        // A wall cell carrying a leftover enemy override (e.g. after an
        // in-place edit) must serialise as a bare character, never a malformed
        // wall-with-enemy-fields entity.
        let mut d = MazeDefinition::from_vec(vec![vec!['S', 'W'], vec![' ', 'F']]);
        d.cell_entities.insert(
            (0, 1),
            vec![CellEntity::Enemy(EnemyOverride {
                enemy_type: None,
                damage: Some(9),
                move_period_ms: None,
            })],
        );
        let back = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(back, r#"{"grid":[["S","W"],[" ","F"]]}"#);
    }

    #[test]
    #[should_panic(expected = "multiple entities per cell not yet supported (found 2)")]
    fn cannot_deserialize_cell_with_two_entities() {
        let s = r#"{"grid":[["S",[{"type":"E"},{"type":"E"}]],[" ","F"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "a cell entity array must contain exactly one entity (found 0)")]
    fn cannot_deserialize_empty_cell_entity_array() {
        let s = r#"{"grid":[["S",[]],[" ","F"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "unknown variant `W`")]
    fn cannot_override_non_feature_cell() {
        // A `type` outside the four override variants (E/H/K/D) is rejected by
        // the tagged-enum deserialiser — overrides only exist on those cells.
        let s = r#"{"grid":[["S",[{"type":"W","damage":2}]],[" ","F"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "enemy 'movePeriodMs' must be a non-negative finite number")]
    fn cannot_deserialize_negative_move_period() {
        let s = r#"{"grid":[["S",[{"type":"E","movePeriodMs":-5.0}]],[" ","F"]]}"#;
        let _d: MazeDefinition = serde_json::from_str(s).expect("Failed to deserialize");
    }

    // Private test helper functions
    fn assert_empty_cols(d: &MazeDefinition, start_col: usize, end_col: usize) {
        let row_count = d.row_count();
        for row_idx in 0..row_count {
            for col_idx in start_col..(end_col + 1) {
                assert_eq!(d.grid[row_idx][col_idx], ' ');
            }
        }
    }

    fn assert_empty_rows(d: &MazeDefinition, start_row: usize, end_row: usize) {
        let col_count = d.col_count();
        for row_idx in start_row..(end_row + 1) {
            for col_idx in 0..col_count {
                assert_eq!(d.grid[row_idx][col_idx], ' ');
            }
        }
    }

    fn assert_cell_value(d: &MazeDefinition, from: MazePoint, to: MazePoint, expected: char) {
        let top_row = from.row.min(to.row);
        let bottom_row = from.row.max(to.row);
        let left_col = from.col.min(to.col);
        let right_col = from.col.max(to.col);
        for row_idx in top_row..(bottom_row + 1) {
            for col_idx in left_col..(right_col + 1) {
                if d.grid[row_idx][col_idx] != expected {
                    panic!(
                        "grid contains unexpected value: '{}' - expected: '{}' (row: {}, col: {})",
                        d.grid[row_idx][col_idx], expected, row_idx, col_idx
                    );
                }
            }
        }
    }
}