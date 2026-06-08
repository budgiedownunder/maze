using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Api;
using Maze.Maui.App.Services;
using CellType = Maze.Api.Maze.CellType;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Drives the inline cell-override panel — the per-cell characteristics editor for
    /// a single selected feature cell (W/K/D/E/H). Mirrors the web editor's
    /// CellOverridePanel: fields seed from the cell's current override, every change
    /// applies live (or clears the override when every field is back to default), and
    /// the wall type is a two-tier choice (a special type, or "Wall" + a solid texture).
    /// The panel never introduces a new cell type — it only layers characteristics.
    /// </summary>
    public partial class CellOverridePanelViewModel : ObservableObject
    {
        private readonly ICellOverrideEditor editor;
        // Suppresses the live-apply while LoadCell / Reset seed the fields, so seeding
        // never re-writes the override it just read.
        private bool suppressApply;

        /// <summary>
        /// Constructor.
        /// </summary>
        /// <param name="editor">The grid the panel reads and writes overrides on</param>
        public CellOverridePanelViewModel(ICellOverrideEditor editor)
        {
            this.editor = editor;
        }

        /// <summary>Selected cell row (one-based).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Title))]
        private int row;

        /// <summary>Selected cell column (one-based).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Title))]
        private int column;

        /// <summary>The selected cell's feature type.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Title))]
        [NotifyPropertyChangedFor(nameof(IsEnemy))]
        [NotifyPropertyChangedFor(nameof(IsHealth))]
        [NotifyPropertyChangedFor(nameof(IsKey))]
        [NotifyPropertyChangedFor(nameof(IsDoor))]
        [NotifyPropertyChangedFor(nameof(IsWall))]
        [NotifyPropertyChangedFor(nameof(IsWallTextureVisible))]
        private CellType cellType;

        /// <summary>Whether the panel is shown (a single overridable cell is selected).</summary>
        [ObservableProperty]
        private bool isVisible;

        // ── Enemy ──
        /// <summary>Enemy rig override, or null to inherit the default.</summary>
        [ObservableProperty]
        private EnemyType? enemyTypeValue;
        /// <summary>Enemy damage override (numeric text; blank = inherit).</summary>
        [ObservableProperty]
        private string damageText = "";
        /// <summary>Enemy move-interval override in ms (numeric text; blank = inherit).</summary>
        [ObservableProperty]
        private string movePeriodMsText = "";

        // ── Health ──
        /// <summary>Health rig override, or null to inherit.</summary>
        [ObservableProperty]
        private HealthStyle? healthStyleValue;
        /// <summary>Heal-amount override (numeric text; blank = inherit).</summary>
        [ObservableProperty]
        private string healAmountText = "";

        // ── Key ──
        /// <summary>Key-holder rig override, or null to inherit.</summary>
        [ObservableProperty]
        private KeyHolderStyle? keyHolderValue;

        // ── Door ──
        /// <summary>Door rig override, or null to inherit.</summary>
        [ObservableProperty]
        private DoorStyle? doorStyleValue;

        // ── Wall (two-tier) ──
        /// <summary>The wall's special type (water/lava/iron-fence), or null for "Wall"
        /// (a solid texture chosen via <see cref="WallTexture"/>).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsWallTextureVisible))]
        private WallType? specialWallType;
        /// <summary>The solid wall texture (shown only under "Wall"), or null for the
        /// varied default.</summary>
        [ObservableProperty]
        private WallType? wallTexture;

        /// <summary>Whether the selected cell is an enemy cell.</summary>
        public bool IsEnemy => CellType == CellType.Enemy;
        /// <summary>Whether the selected cell is a health cell.</summary>
        public bool IsHealth => CellType == CellType.Health;
        /// <summary>Whether the selected cell is a key cell.</summary>
        public bool IsKey => CellType == CellType.Key;
        /// <summary>Whether the selected cell is a door cell.</summary>
        public bool IsDoor => CellType == CellType.Door;
        /// <summary>Whether the selected cell is a wall cell.</summary>
        public bool IsWall => CellType == CellType.Wall;
        /// <summary>Whether the solid-texture picker is shown (only under "Wall").</summary>
        public bool IsWallTextureVisible => IsWall && SpecialWallType is null;

        /// <summary>The panel heading: the cell type and its one-based coordinates.</summary>
        public string Title => $"{TypeLabel(CellType)} [{Row},{Column}]";

        partial void OnEnemyTypeValueChanged(EnemyType? value) => ApplyCurrent();
        partial void OnDamageTextChanged(string value) => ApplyCurrent();
        partial void OnMovePeriodMsTextChanged(string value) => ApplyCurrent();
        partial void OnHealthStyleValueChanged(HealthStyle? value) => ApplyCurrent();
        partial void OnHealAmountTextChanged(string value) => ApplyCurrent();
        partial void OnKeyHolderValueChanged(KeyHolderStyle? value) => ApplyCurrent();
        partial void OnDoorStyleValueChanged(DoorStyle? value) => ApplyCurrent();
        partial void OnSpecialWallTypeChanged(WallType? value) => ApplyCurrent();
        partial void OnWallTextureChanged(WallType? value) => ApplyCurrent();

        /// <summary>
        /// Seeds the panel for a cell: shows it for an overridable feature type and
        /// populates the fields from the cell's current override (or defaults). Hidden
        /// for start/finish/empty cells.
        /// </summary>
        /// <param name="cellRow">Row index (one-based)</param>
        /// <param name="cellColumn">Column index (one-based)</param>
        /// <param name="type">The cell's type</param>
        public void LoadCell(int cellRow, int cellColumn, CellType type)
        {
            suppressApply = true;
            Row = cellRow;
            Column = cellColumn;
            CellType = type;
            IsVisible = IsOverridable(type);

            // Reset every field, then seed from the existing override.
            EnemyTypeValue = null;
            DamageText = "";
            MovePeriodMsText = "";
            HealthStyleValue = null;
            HealAmountText = "";
            KeyHolderValue = null;
            DoorStyleValue = null;
            SpecialWallType = null;
            WallTexture = null;

            CellEntityInfo? current = IsVisible ? editor.GetCellOverride(cellRow, cellColumn) : null;
            switch (current)
            {
                case EnemyCellEntity e:
                    EnemyTypeValue = e.EnemyType;
                    DamageText = e.Damage?.ToString() ?? "";
                    MovePeriodMsText = e.MovePeriodMs?.ToString() ?? "";
                    break;
                case HealthCellEntity h:
                    HealthStyleValue = h.HealthStyle;
                    HealAmountText = h.HealAmount?.ToString() ?? "";
                    break;
                case KeyCellEntity k:
                    KeyHolderValue = k.KeyHolder;
                    break;
                case DoorCellEntity d:
                    DoorStyleValue = d.DoorStyle;
                    break;
                case WallCellEntity { WallType: WallType wt }:
                    if (IsSpecialWall(wt))
                    {
                        SpecialWallType = wt;
                    }
                    else
                    {
                        WallTexture = wt;
                    }
                    break;
            }
            suppressApply = false;
        }

        /// <summary>Clears the cell's override and resets every field to its default.</summary>
        [RelayCommand]
        private void Reset()
        {
            suppressApply = true;
            EnemyTypeValue = null;
            DamageText = "";
            MovePeriodMsText = "";
            HealthStyleValue = null;
            HealAmountText = "";
            KeyHolderValue = null;
            DoorStyleValue = null;
            SpecialWallType = null;
            WallTexture = null;
            suppressApply = false;

            editor.ClearCellOverride(Row, Column);
            editor.RefreshCellContent(Row, Column);
        }

        // Builds the entity from the current fields and applies it — or clears the
        // override when every field is back to default (matching the web editor's emit
        // rule: an entity with no characteristics beyond its type is a clear).
        private void ApplyCurrent()
        {
            if (suppressApply)
            {
                return;
            }
            CellEntityInfo? entity = BuildEntity();
            if (entity is not null)
            {
                editor.SetCellOverride(Row, Column, entity);
            }
            else
            {
                editor.ClearCellOverride(Row, Column);
            }
            editor.RefreshCellContent(Row, Column);
        }

        private CellEntityInfo? BuildEntity()
        {
            switch (CellType)
            {
                case CellType.Enemy:
                    EnemyCellEntity enemy = new()
                    {
                        EnemyType = EnemyTypeValue,
                        Damage = ParseNonNegInt(DamageText),
                        MovePeriodMs = ParseNonNegFloat(MovePeriodMsText)
                    };
                    return enemy.EnemyType is null && enemy.Damage is null && enemy.MovePeriodMs is null ? null : enemy;
                case CellType.Health:
                    HealthCellEntity health = new()
                    {
                        HealthStyle = HealthStyleValue,
                        HealAmount = ParseNonNegInt(HealAmountText)
                    };
                    return health.HealthStyle is null && health.HealAmount is null ? null : health;
                case CellType.Key:
                    return KeyHolderValue is null ? null : new KeyCellEntity { KeyHolder = KeyHolderValue };
                case CellType.Door:
                    return DoorStyleValue is null ? null : new DoorCellEntity { DoorStyle = DoorStyleValue };
                case CellType.Wall:
                    WallType? wallType = EffectiveWallType(SpecialWallType, WallTexture);
                    return wallType is null ? null : new WallCellEntity { WallType = wallType };
                default:
                    return null;
            }
        }

        // "Wall" + a solid texture resolves to that texture; a special type resolves to
        // itself; "Wall" + the default texture resolves to no override.
        private static WallType? EffectiveWallType(WallType? special, WallType? texture) => special ?? texture;

        private static bool IsSpecialWall(WallType type) =>
            type is WallType.Water or WallType.Lava or WallType.IronFence;

        private static bool IsOverridable(CellType type) =>
            type is CellType.Wall or CellType.Key or CellType.Door or CellType.Enemy or CellType.Health;

        private static string TypeLabel(CellType type) => type switch
        {
            CellType.Enemy => "Enemy",
            CellType.Health => "Health",
            CellType.Key => "Key",
            CellType.Door => "Door",
            CellType.Wall => "Wall",
            _ => ""
        };

        private static uint? ParseNonNegInt(string? text) =>
            uint.TryParse((text ?? "").Trim(), out uint value) ? value : null;

        private static float? ParseNonNegFloat(string? text)
        {
            string trimmed = (text ?? "").Trim();
            if (trimmed.Length == 0)
            {
                return null;
            }
            return float.TryParse(trimmed, out float value) && float.IsFinite(value) && value >= 0 ? value : null;
        }
    }
}
