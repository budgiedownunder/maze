using System.Text.Json.Serialization;

namespace Maze.Api
{
    /// <summary>Visual rig for an enemy (<c>'E'</c>) cell. Renderer-only.</summary>
    [JsonConverter(typeof(JsonStringEnumConverter<EnemyType>))]
    public enum EnemyType
    {
        /// <summary>The default enemy rig.</summary>
        [JsonStringEnumMemberName("goblin")] Goblin,
        /// <summary>An alternative enemy rig.</summary>
        [JsonStringEnumMemberName("ghost")] Ghost,
    }

    /// <summary>Visual rig for a health-pickup (<c>'H'</c>) cell. Renderer-only.</summary>
    [JsonConverter(typeof(JsonStringEnumConverter<HealthStyle>))]
    public enum HealthStyle
    {
        /// <summary>The default health-pickup rig.</summary>
        [JsonStringEnumMemberName("heart")] Heart,
        /// <summary>An alternative health-pickup rig.</summary>
        [JsonStringEnumMemberName("potion")] Potion,
    }

    /// <summary>Visual rig for a key-holder (<c>'K'</c>) cell. Renderer-only.</summary>
    [JsonConverter(typeof(JsonStringEnumConverter<KeyHolderStyle>))]
    public enum KeyHolderStyle
    {
        /// <summary>The default key-holder rig.</summary>
        [JsonStringEnumMemberName("pedestal")] Pedestal,
        /// <summary>A chest the key sits inside.</summary>
        [JsonStringEnumMemberName("chest")] Chest,
        /// <summary>A free-floating key with no holder.</summary>
        [JsonStringEnumMemberName("floating_key")] FloatingKey,
    }

    /// <summary>
    /// Visual style for a treasure (<c>'T'</c>) cell. Also drives the cell's default score
    /// value (Silver lowest, Diamonds highest). A bare <c>'T'</c> defaults to <see cref="Silver"/>.
    /// </summary>
    [JsonConverter(typeof(JsonStringEnumConverter<TreasureStyle>))]
    public enum TreasureStyle
    {
        /// <summary>Silver treasure — the default style and lowest value.</summary>
        [JsonStringEnumMemberName("silver")] Silver,
        /// <summary>Gold treasure.</summary>
        [JsonStringEnumMemberName("gold")] Gold,
        /// <summary>Diamonds treasure — the highest value.</summary>
        [JsonStringEnumMemberName("diamonds")] Diamonds,
        /// <summary>Jewels treasure.</summary>
        [JsonStringEnumMemberName("jewels")] Jewels,
    }

    /// <summary>Open-animation rig for a door (<c>'D'</c>) cell. Renderer-only.</summary>
    [JsonConverter(typeof(JsonStringEnumConverter<DoorStyle>))]
    public enum DoorStyle
    {
        /// <summary>The default door rig (hinged swing).</summary>
        [JsonStringEnumMemberName("swing")] Swing,
        /// <summary>A door that slides aside.</summary>
        [JsonStringEnumMemberName("slide")] Slide,
        /// <summary>A portcullis that lifts.</summary>
        [JsonStringEnumMemberName("portcullis")] Portcullis,
        /// <summary>A door that dissolves away.</summary>
        [JsonStringEnumMemberName("dissolve")] Dissolve,
    }

    /// <summary>
    /// Visual type for a wall (<c>'W'</c>) cell. Renderer-only; the cell stays
    /// impassable. Shares its vocabulary with the per-maze <c>wall_type</c> launch
    /// setting: the solid-wall textures force a specific texture in place of the
    /// default per-cell variation, and <c>water</c> / <c>lava</c> / <c>iron_fence</c>
    /// are non-occluding skins (a floor-level pool, or see-through bars).
    /// </summary>
    [JsonConverter(typeof(JsonStringEnumConverter<WallType>))]
    public enum WallType
    {
        /// <summary>A brick wall texture.</summary>
        [JsonStringEnumMemberName("brick")] Brick,
        /// <summary>A dressed-stone wall texture.</summary>
        [JsonStringEnumMemberName("dressed_stone")] DressedStone,
        /// <summary>A wood-plank wall texture.</summary>
        [JsonStringEnumMemberName("wood")] Wood,
        /// <summary>A cobblestone wall texture.</summary>
        [JsonStringEnumMemberName("cobblestone")] Cobblestone,
        /// <summary>A floor-level pool of water (non-occluding).</summary>
        [JsonStringEnumMemberName("water")] Water,
        /// <summary>A floor-level pool of lava (non-occluding).</summary>
        [JsonStringEnumMemberName("lava")] Lava,
        /// <summary>A wall of see-through vertical iron bars (non-occluding).</summary>
        [JsonStringEnumMemberName("iron_fence")] IronFence,
    }

    /// <summary>
    /// One entity occupying a cell, together with its optional override
    /// characteristics — the typed mirror of the wire entity object
    /// (e.g. <c>{ "type": "E", "enemyType": "ghost", "damage": 2 }</c>).
    ///
    /// The concrete subclass (<see cref="EnemyCellEntity"/> / <see cref="HealthCellEntity"/> /
    /// <see cref="KeyCellEntity"/> / <see cref="DoorCellEntity"/>) determines the cell type and
    /// carries only the fields meaningful to it, so an invalid field/type combination is
    /// unrepresentable. It serialises as a <c>"type"</c>-tagged object (<c>"E"</c>/<c>"H"</c>/<c>"K"</c>/<c>"D"</c>),
    /// mirroring the Rust <c>CellEntity</c> tagged enum. Read and written via
    /// <see cref="Maze.GetCellEntity"/> / <see cref="Maze.SetCellEntity"/>.
    /// </summary>
    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(EnemyCellEntity), "E")]
    [JsonDerivedType(typeof(HealthCellEntity), "H")]
    [JsonDerivedType(typeof(KeyCellEntity), "K")]
    [JsonDerivedType(typeof(DoorCellEntity), "D")]
    [JsonDerivedType(typeof(WallCellEntity), "W")]
    [JsonDerivedType(typeof(TreasureCellEntity), "T")]
    public abstract class CellEntityInfo
    {
    }

    /// <summary>Per-cell override for an enemy (<c>'E'</c>) cell.</summary>
    public sealed class EnemyCellEntity : CellEntityInfo
    {
        /// <summary>Enemy visual rig.</summary>
        [JsonPropertyName("enemyType")] public EnemyType? EnemyType { get; set; }
        /// <summary>Enemy damage per collision.</summary>
        [JsonPropertyName("damage")] public uint? Damage { get; set; }
        /// <summary>Enemy move period in milliseconds.</summary>
        [JsonPropertyName("movePeriodMs")] public float? MovePeriodMs { get; set; }
    }

    /// <summary>Per-cell override for a health-pickup (<c>'H'</c>) cell.</summary>
    public sealed class HealthCellEntity : CellEntityInfo
    {
        /// <summary>Health-pickup visual rig.</summary>
        [JsonPropertyName("healthStyle")] public HealthStyle? HealthStyle { get; set; }
        /// <summary>Hit points restored when consumed.</summary>
        [JsonPropertyName("healAmount")] public uint? HealAmount { get; set; }
    }

    /// <summary>Per-cell override for a key-holder (<c>'K'</c>) cell.</summary>
    public sealed class KeyCellEntity : CellEntityInfo
    {
        /// <summary>Key-holder visual rig.</summary>
        [JsonPropertyName("keyHolder")] public KeyHolderStyle? KeyHolder { get; set; }
    }

    /// <summary>Per-cell override for a door (<c>'D'</c>) cell.</summary>
    public sealed class DoorCellEntity : CellEntityInfo
    {
        /// <summary>Door open-animation rig.</summary>
        [JsonPropertyName("doorStyle")] public DoorStyle? DoorStyle { get; set; }
    }

    /// <summary>Per-cell override for a wall (<c>'W'</c>) cell.</summary>
    public sealed class WallCellEntity : CellEntityInfo
    {
        /// <summary>Wall visual type.</summary>
        [JsonPropertyName("wallType")] public WallType? WallType { get; set; }
    }

    /// <summary>Per-cell override for a treasure (<c>'T'</c>) cell.</summary>
    public sealed class TreasureCellEntity : CellEntityInfo
    {
        /// <summary>Treasure visual style. A bare <c>'T'</c> (no style) renders as <see cref="TreasureStyle.Silver"/>.</summary>
        [JsonPropertyName("style")] public TreasureStyle? Style { get; set; }
        /// <summary>Score awarded when collected. When <c>null</c>, the style's default value applies.</summary>
        [JsonPropertyName("value")] public uint? Value { get; set; }
    }
}
