using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Api;
using Maze.Maui.App.Services;
using CellType = Maze.Api.Maze.CellType;

// CellSprite lives in the parent Maze.Maui.App namespace.
using Maze.Maui.App;

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

        // Bottom-right of the selection (one-based); the target cell (Row/Column) is its
        // top-left. Equal to the target for a single cell.
        private int rectBottom;
        private int rectRight;

        /// <summary>Number of cells in the selection (1 for a single cell).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsMultiCell))]
        [NotifyPropertyChangedFor(nameof(ApplyToAllText))]
        [NotifyPropertyChangedFor(nameof(Title))]
        private int selectionCount = 1;

        /// <summary>Whether more than one cell is selected (enables "Apply to all").</summary>
        public bool IsMultiCell => SelectionCount > 1;
        /// <summary>Label for the "Apply to all" button.</summary>
        public string ApplyToAllText => $"Apply to all {SelectionCount} cells";

        // ── Enemy ──
        /// <summary>Enemy rig override, or null to inherit the default.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(EnemyTypeIndex))]
        [NotifyPropertyChangedFor(nameof(EnemyPreviewImage))]
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
        [NotifyPropertyChangedFor(nameof(HealthStyleIndex))]
        [NotifyPropertyChangedFor(nameof(HealthPreviewImage))]
        private HealthStyle? healthStyleValue;
        /// <summary>Heal-amount override (numeric text; blank = inherit).</summary>
        [ObservableProperty]
        private string healAmountText = "";

        // ── Key ──
        /// <summary>Key-holder rig override, or null to inherit.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(KeyHolderIndex))]
        private KeyHolderStyle? keyHolderValue;

        // ── Door ──
        /// <summary>Door rig override, or null to inherit.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(DoorStyleIndex))]
        private DoorStyle? doorStyleValue;

        // ── Wall (two-tier) ──
        /// <summary>The tier-1 wall kind selected by the user.</summary>
        public enum WallKind
        {
            /// <summary>Inherit the maze's wallType — no per-cell override.</summary>
            Default,
            /// <summary>Force a solid wall for this cell (texture chosen in tier 2).</summary>
            Wall,
            /// <summary>Water.</summary>
            Water,
            /// <summary>Lava.</summary>
            Lava,
            /// <summary>Iron fence.</summary>
            IronFence
        }

        /// <summary>Tier-1 wall kind: Default (inherit the maze's wallType — no override),
        /// Wall (force a solid texture, chosen via <see cref="WallTexture"/>), or a special
        /// type (water/lava/iron-fence).</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsWallTextureVisible))]
        [NotifyPropertyChangedFor(nameof(WallTypeIndex))]
        [NotifyPropertyChangedFor(nameof(WallTextureOptions))]
        [NotifyPropertyChangedFor(nameof(WallTextureIndex))]
        [NotifyPropertyChangedFor(nameof(WallPreviewImage))]
        private WallKind wallTypeKind;
        /// <summary>The solid wall texture for tier 2, or null for the "Default" (inherit)
        /// option under the Default kind.</summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(WallTextureIndex))]
        [NotifyPropertyChangedFor(nameof(WallPreviewImage))]
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
        /// <summary>Whether the solid-texture picker is shown: under "Wall", or under
        /// "Default" when the maze default wall is itself solid (so just this cell's texture
        /// can be overridden). Hidden when "Default" inherits a special (water/lava/iron-fence) look.</summary>
        public bool IsWallTextureVisible =>
            IsWall && (WallTypeKind == WallKind.Wall || (WallTypeKind == WallKind.Default && MazeDefaultWallIsSolid));

        // Whether the maze's default wallType has no distinct 2D sprite (a solid texture, or
        // unset → the brick default). False for water/lava/iron-fence.
        private bool MazeDefaultWallIsSolid =>
            editor.GameSettings?.WallType is not ("water" or "lava" or "iron_fence");

        /// <summary>Sprite previewing the selected enemy rig, or the maze default (ghost)
        /// when "Default" is selected.</summary>
        public string EnemyPreviewImage =>
            CellSprite.PreviewImageName(CellType.Enemy,
                EnemyTypeValue is null ? null : new EnemyCellEntity { EnemyType = EnemyTypeValue },
                editor.GameSettings, "enemy.png");
        /// <summary>Sprite previewing the selected health rig, or the maze default (potion)
        /// when "Default" is selected.</summary>
        public string HealthPreviewImage =>
            CellSprite.PreviewImageName(CellType.Health,
                HealthStyleValue is null ? null : new HealthCellEntity { HealthStyle = HealthStyleValue },
                editor.GameSettings, "health.png");
        /// <summary>Sprite previewing the selected wall type, or the maze default (e.g. lava)
        /// when "Default" inherits it.</summary>
        public string WallPreviewImage =>
            CellSprite.PreviewImageName(CellType.Wall,
                EffectiveWallType() is { } wallType ? new WallCellEntity { WallType = wallType } : null,
                editor.GameSettings, "wall.png");

        /// <summary>The panel heading: the cell type and its one-based coordinates.</summary>
        public string Title =>
            $"{TypeLabel(CellType)} [{Row},{Column}]{(IsMultiCell ? $" +{SelectionCount - 1} more" : "")}";

        // ── Picker bindings ──
        // MAUI Pickers bind to an options list + a SelectedIndex; each index maps to the
        // nullable-enum field above, with index 0 being the inherit option ("Default", or
        // "Wall" for the wall type tier).

        /// <summary>Enemy rig picker options.</summary>
        public IReadOnlyList<string> EnemyTypeOptions { get; } = new[] { "Default", "Goblin", "Ghost" };
        /// <summary>Selected index of the enemy rig picker.</summary>
        public int EnemyTypeIndex
        {
            get => EnemyTypeValue is null ? 0 : (int)EnemyTypeValue.Value + 1;
            set => EnemyTypeValue = value <= 0 ? null : (EnemyType)(value - 1);
        }

        /// <summary>Health rig picker options.</summary>
        public IReadOnlyList<string> HealthStyleOptions { get; } = new[] { "Default", "Heart", "Potion" };
        /// <summary>Selected index of the health rig picker.</summary>
        public int HealthStyleIndex
        {
            get => HealthStyleValue is null ? 0 : (int)HealthStyleValue.Value + 1;
            set => HealthStyleValue = value <= 0 ? null : (HealthStyle)(value - 1);
        }

        /// <summary>Key-holder picker options.</summary>
        public IReadOnlyList<string> KeyHolderOptions { get; } = new[] { "Default", "Pedestal", "Chest", "Floating Key" };
        /// <summary>Selected index of the key-holder picker.</summary>
        public int KeyHolderIndex
        {
            get => KeyHolderValue is null ? 0 : (int)KeyHolderValue.Value + 1;
            set => KeyHolderValue = value <= 0 ? null : (KeyHolderStyle)(value - 1);
        }

        /// <summary>Door rig picker options.</summary>
        public IReadOnlyList<string> DoorStyleOptions { get; } = new[] { "Default", "Swing", "Slide", "Portcullis", "Dissolve" };
        /// <summary>Selected index of the door rig picker.</summary>
        public int DoorStyleIndex
        {
            get => DoorStyleValue is null ? 0 : (int)DoorStyleValue.Value + 1;
            set => DoorStyleValue = value <= 0 ? null : (DoorStyle)(value - 1);
        }

        /// <summary>Wall type (tier 1) picker options.</summary>
        public IReadOnlyList<string> WallTypeOptions { get; } = new[] { "Default", "Wall", "Water", "Lava", "Iron Fence" };
        /// <summary>Selected index of the wall type (tier 1) picker (maps to <see cref="WallKind"/>).</summary>
        public int WallTypeIndex
        {
            get => (int)WallTypeKind;
            set => WallTypeKind = value >= 0 && value <= (int)WallKind.IronFence ? (WallKind)value : WallKind.Default;
        }

        // Tier-2 solid textures map to WallType 0..3 (Brick, DressedStone, Wood, Cobblestone).
        private static readonly string[] SolidTextureOptions = { "Brick", "Dressed Stone", "Wood", "Cobblestone" };
        private static readonly string[] SolidTextureOptionsWithDefault = { "Default", "Brick", "Dressed Stone", "Wood", "Cobblestone" };
        /// <summary>Wall texture (tier 2) picker options — "Default" (inherit) is offered only
        /// under the Default kind; "Wall" forces a concrete solid, so it drops "Default".</summary>
        public IReadOnlyList<string> WallTextureOptions =>
            WallTypeKind == WallKind.Wall ? SolidTextureOptions : SolidTextureOptionsWithDefault;
        /// <summary>Selected index of the wall texture (tier 2) picker.</summary>
        public int WallTextureIndex
        {
            get => WallTypeKind == WallKind.Wall
                ? (WallTexture is null ? 0 : (int)WallTexture.Value)
                : (WallTexture is null ? 0 : (int)WallTexture.Value + 1);
            set
            {
                if (WallTypeKind == WallKind.Wall)
                {
                    WallTexture = value >= 0 && value < SolidTextureOptions.Length ? (WallType)value : WallType.Brick;
                }
                else
                {
                    WallTexture = value <= 0 ? null : (WallType)(value - 1);
                }
            }
        }

        partial void OnEnemyTypeValueChanged(EnemyType? value) => ApplyCurrent();
        partial void OnDamageTextChanged(string value) => ApplyCurrent();
        partial void OnMovePeriodMsTextChanged(string value) => ApplyCurrent();
        partial void OnHealthStyleValueChanged(HealthStyle? value) => ApplyCurrent();
        partial void OnHealAmountTextChanged(string value) => ApplyCurrent();
        partial void OnKeyHolderValueChanged(KeyHolderStyle? value) => ApplyCurrent();
        partial void OnDoorStyleValueChanged(DoorStyle? value) => ApplyCurrent();
        partial void OnWallTypeKindChanged(WallKind value)
        {
            // Mirror the web panel's tier-1 change: "Wall" forces a concrete solid (keep a
            // prior solid texture, else brick); "Default"/special carry no solid texture.
            // Adjust the texture silently, then apply once.
            bool wasSuppressed = suppressApply;
            suppressApply = true;
            WallTexture = value == WallKind.Wall
                ? (WallTexture is { } t && IsSolidWall(t) ? t : WallType.Brick)
                : null;
            suppressApply = wasSuppressed;
            ApplyCurrent();
        }
        partial void OnWallTextureChanged(WallType? value) => ApplyCurrent();

        /// <summary>
        /// Seeds the panel for a single cell — shorthand for a one-cell selection.
        /// </summary>
        /// <param name="cellRow">Row index (one-based)</param>
        /// <param name="cellColumn">Column index (one-based)</param>
        /// <param name="type">The cell's type</param>
        public void LoadCell(int cellRow, int cellColumn, CellType type) =>
            LoadCell(cellRow, cellColumn, cellRow, cellColumn, type);

        /// <summary>
        /// Seeds the panel for a rectangular selection whose cells are all the same type:
        /// shows it for an overridable feature type, targets and seeds from the top-left
        /// cell, and tracks the cell count so "Apply to all" can stamp the block. Hidden
        /// for start/finish/empty selections.
        /// </summary>
        /// <param name="top">Top row of the selection (one-based)</param>
        /// <param name="left">Left column of the selection (one-based)</param>
        /// <param name="bottom">Bottom row of the selection (one-based)</param>
        /// <param name="right">Right column of the selection (one-based)</param>
        /// <param name="type">The selection's (uniform) cell type</param>
        public void LoadCell(int top, int left, int bottom, int right, CellType type)
        {
            suppressApply = true;
            Row = top;
            Column = left;
            rectBottom = bottom;
            rectRight = right;
            SelectionCount = (bottom - top + 1) * (right - left + 1);
            CellType = type;
            IsVisible = IsOverridable(type);

            // Reset every field, then seed from the top-left cell's existing override.
            EnemyTypeValue = null;
            DamageText = "";
            MovePeriodMsText = "";
            HealthStyleValue = null;
            HealAmountText = "";
            KeyHolderValue = null;
            DoorStyleValue = null;
            WallTypeKind = WallKind.Default;
            WallTexture = null;

            CellEntityInfo? current = IsVisible ? editor.GetCellOverride(top, left) : null;
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
                        WallTypeKind = ToWallKind(wt);
                    }
                    else
                    {
                        WallTypeKind = WallKind.Wall;
                        WallTexture = wt;
                    }
                    break;
            }
            suppressApply = false;
        }

        /// <summary>
        /// Resets every field to its default and clears the override on every cell in the
        /// selection (just the one cell for a single-cell selection).
        /// </summary>
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
            WallTypeKind = WallKind.Default;
            WallTexture = null;
            suppressApply = false;

            for (int r = Row; r <= rectBottom; r++)
            {
                for (int c = Column; c <= rectRight; c++)
                {
                    editor.ClearCellOverride(r, c);
                    editor.RefreshCellContent(r, c);
                }
            }
        }

        /// <summary>
        /// Stamps the top-left cell's current override across every cell in the selection
        /// (or clears them all when it has reverted to default), honouring the same
        /// clear-on-default rule. The entity is shared by reference — overrides are
        /// replaced, never mutated, so the cells stay independent.
        /// </summary>
        [RelayCommand]
        private void ApplyToAll()
        {
            CellEntityInfo? current = editor.GetCellOverride(Row, Column);
            for (int r = Row; r <= rectBottom; r++)
            {
                for (int c = Column; c <= rectRight; c++)
                {
                    if (r == Row && c == Column)
                    {
                        continue; // the top-left cell already carries it
                    }
                    if (current is not null)
                    {
                        editor.SetCellOverride(r, c, current);
                    }
                    else
                    {
                        editor.ClearCellOverride(r, c);
                    }
                    editor.RefreshCellContent(r, c);
                }
            }
        }

        /// <summary>Steps the enemy damage override up by one.</summary>
        [RelayCommand]
        private void IncrementDamage() => DamageText = StepInt(DamageText, 1);
        /// <summary>Steps the enemy damage override down by one.</summary>
        [RelayCommand]
        private void DecrementDamage() => DamageText = StepInt(DamageText, -1);
        /// <summary>Steps the enemy move-interval override up by one.</summary>
        [RelayCommand]
        private void IncrementMovePeriod() => MovePeriodMsText = StepFloat(MovePeriodMsText, 1);
        /// <summary>Steps the enemy move-interval override down by one.</summary>
        [RelayCommand]
        private void DecrementMovePeriod() => MovePeriodMsText = StepFloat(MovePeriodMsText, -1);
        /// <summary>Steps the heal-amount override up by one.</summary>
        [RelayCommand]
        private void IncrementHealAmount() => HealAmountText = StepInt(HealAmountText, 1);
        /// <summary>Steps the heal-amount override down by one.</summary>
        [RelayCommand]
        private void DecrementHealAmount() => HealAmountText = StepInt(HealAmountText, -1);

        // A blank field steps up to 1 and stays blank on a step-down; a value steps by
        // `delta`, clamped at 0. Assigning the text live-applies via its change handler.
        private static string StepInt(string text, int delta)
        {
            uint? current = ParseNonNegInt(text);
            if (current is null)
            {
                return delta > 0 ? "1" : "";
            }
            long next = (long)current.Value + delta;
            return next < 0 ? "0" : next.ToString();
        }

        private static string StepFloat(string text, float delta)
        {
            float? current = ParseNonNegFloat(text);
            if (current is null)
            {
                return delta > 0 ? "1" : "";
            }
            float next = current.Value + delta;
            return next < 0 ? "0" : next.ToString();
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
                    WallType? wallType = EffectiveWallType();
                    return wallType is null ? null : new WallCellEntity { WallType = wallType };
                default:
                    return null;
            }
        }

        // The single flat wallType the tri-state UI resolves to, or null when the cell
        // inherits the maze default ("Default" with no texture override). "Wall" always
        // forces a concrete solid; a special kind maps to itself.
        private WallType? EffectiveWallType() => WallTypeKind switch
        {
            WallKind.Default => WallTexture,
            WallKind.Wall => WallTexture ?? WallType.Brick,
            WallKind.Water => WallType.Water,
            WallKind.Lava => WallType.Lava,
            WallKind.IronFence => WallType.IronFence,
            _ => null
        };

        private static bool IsSpecialWall(WallType type) =>
            type is WallType.Water or WallType.Lava or WallType.IronFence;

        private static bool IsSolidWall(WallType type) =>
            type is WallType.Brick or WallType.DressedStone or WallType.Wood or WallType.Cobblestone;

        private static WallKind ToWallKind(WallType special) => special switch
        {
            WallType.Water => WallKind.Water,
            WallType.Lava => WallKind.Lava,
            WallType.IronFence => WallKind.IronFence,
            _ => WallKind.Wall
        };

        /// <summary>
        /// Notifies the panel that the maze's game settings changed, so the maze-default
        /// previews and the maze-aware wall-texture visibility re-evaluate. Called by the
        /// editor after the settings editor applies a change.
        /// </summary>
        public void NotifyGameSettingsChanged()
        {
            OnPropertyChanged(nameof(EnemyPreviewImage));
            OnPropertyChanged(nameof(HealthPreviewImage));
            OnPropertyChanged(nameof(WallPreviewImage));
            OnPropertyChanged(nameof(IsWallTextureVisible));
            OnPropertyChanged(nameof(WallTextureOptions));
            OnPropertyChanged(nameof(WallTextureIndex));
        }

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
