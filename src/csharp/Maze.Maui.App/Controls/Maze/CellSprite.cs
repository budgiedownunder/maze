using System.Collections.Generic;
using Maze.Api;
using Maze.Maui.App.Models;
using static Maze.Api.Maze;

namespace Maze.Maui.App
{
    /// <summary>
    /// Resolves the variant sprite an editor cell should show for its per-cell
    /// override. Mirrors the web editor's sprite resolver: only ghost / potion / water
    /// / lava / iron-fence have a distinct 2D sprite. Every other override — a default
    /// rig (goblin / heart), a numeric-only override (just a damage or heal amount), a
    /// key-holder or door style, or a solid wall texture (brick / dressed stone / wood
    /// / cobblestone, which are 3D-only) — renders the cell's base sprite, with the
    /// override badge marking that it carries one.
    /// </summary>
    public static class CellSprite
    {
        /// <summary>
        /// The variant image name for an override (e.g. <c>"ghost.png"</c>), or
        /// <c>null</c> when it has no distinct 2D sprite.
        /// </summary>
        /// <param name="cellOverride">The cell's override, or null</param>
        /// <returns>The variant image name, or null</returns>
        public static string? VariantImageName(CellEntityInfo? cellOverride) => cellOverride switch
        {
            EnemyCellEntity { EnemyType: EnemyType.Ghost } => "ghost.png",
            HealthCellEntity { HealthStyle: HealthStyle.Potion } => "potion.png",
            WallCellEntity { WallType: WallType.Water } => "water.png",
            WallCellEntity { WallType: WallType.Lava } => "lava.png",
            WallCellEntity { WallType: WallType.IronFence } => "iron_fence.png",
            // Silver is the default treasure sprite (the hardcoded base), so only the
            // richer styles carry a distinct variant.
            TreasureCellEntity { Style: TreasureStyle.Gold } => "gold_in_trunk.png",
            TreasureCellEntity { Style: TreasureStyle.Diamonds } => "diamonds_in_trunk.png",
            TreasureCellEntity { Style: TreasureStyle.Jewels } => "jewels_in_trunk.png",
            _ => null
        };

        /// <summary>
        /// The base sprite a non-overridden cell inherits from the maze's game settings
        /// (e.g. a lava maze renders its walls as <c>"lava.png"</c>), or <c>null</c> when
        /// the maze default has no distinct 2D sprite (a solid wall texture, the goblin /
        /// heart rigs) so the hardcoded base applies. Only wall / enemy / health carry a 2D
        /// variant; every other cell type is unaffected.
        ///
        /// A per-cell override always wins: when the override explicitly sets this family's
        /// visual field (e.g. a goblin or brick override), the maze default is ignored
        /// (<c>null</c>) so the override's own resolution stands — mirroring the web
        /// editor's <c>override.field ?? settings.field</c> precedence.
        /// </summary>
        /// <param name="cellType">The cell's feature type.</param>
        /// <param name="cellOverride">The cell's per-cell override, or null.</param>
        /// <param name="settings">The maze's game settings, or null.</param>
        /// <returns>The maze-default base image name, or null.</returns>
        public static string? BaseImageName(CellType cellType, CellEntityInfo? cellOverride, MazeGameSettings? settings)
        {
            if (settings is null)
            {
                return null;
            }
            return cellType switch
            {
                CellType.Wall when cellOverride is not WallCellEntity { WallType: not null } => settings.WallType switch
                {
                    "water" => "water.png",
                    "lava" => "lava.png",
                    "iron_fence" => "iron_fence.png",
                    _ => null
                },
                CellType.Enemy when cellOverride is not EnemyCellEntity { EnemyType: not null } =>
                    settings.EnemyType == "ghost" ? "ghost.png" : null,
                CellType.Health when cellOverride is not HealthCellEntity { HealthStyle: not null } =>
                    settings.HealthStyle == "potion" ? "potion.png" : null,
                _ => null
            };
        }

        /// <summary>
        /// Whether the maze-default 2D base sprite of any wall / enemy / health cell differs
        /// between two settings (treating null as "no settings" → the hardcoded bases). Used
        /// to skip the editor-grid refresh when a settings edit only touched 3D-only fields
        /// (sky, timer, rig styles, …) that don't change the 2D display.
        /// </summary>
        /// <param name="before">The settings before the edit, or null.</param>
        /// <param name="after">The settings after the edit, or null.</param>
        /// <returns>True when a 2D base sprite changed.</returns>
        public static bool MazeDefaultBaseChanged(MazeGameSettings? before, MazeGameSettings? after)
        {
            foreach (CellType cellType in new[] { CellType.Wall, CellType.Enemy, CellType.Health })
            {
                if (BaseImageName(cellType, null, before) != BaseImageName(cellType, null, after))
                {
                    return true;
                }
            }
            return false;
        }

        /// <summary>
        /// The sprite for a live in-game enemy: its own rig wins (ghost), else the maze's
        /// game-settings enemy default (a goblin maze or no settings → the generic goblin
        /// <c>"enemy.png"</c>). Mirrors the web game's live-enemy overlay, which resolves
        /// <c>rig ?? settings.enemyType</c>. Unlike a static cell this always returns a
        /// concrete sprite (the overlay is never empty).
        /// </summary>
        /// <param name="rig">The live enemy's visual rig, or null for the default.</param>
        /// <param name="settings">The maze's game settings, or null.</param>
        /// <returns>The enemy sprite image name.</returns>
        public static string LiveEnemyImageName(EnemyType? rig, MazeGameSettings? settings) =>
            VariantImageName(new EnemyCellEntity { EnemyType = rig })
            ?? BaseImageName(CellType.Enemy, rig is null ? null : new EnemyCellEntity { EnemyType = rig }, settings)
            ?? "enemy.png";

        /// <summary>
        /// The sprite to preview for a per-cell selection in the override panel: the
        /// selection's own variant (ghost / potion / lava …), else the maze's game-settings
        /// default for that family (so a "Default" selection previews what the cell actually
        /// renders), else the generic fallback. Mirrors the web panel's preview, which
        /// resolves <c>override ?? mazeDefault</c>.
        /// </summary>
        /// <param name="cellType">The cell's feature type.</param>
        /// <param name="selection">The selection as a cell entity, or null for "Default" (inherit).</param>
        /// <param name="settings">The maze's game settings, or null.</param>
        /// <param name="fallback">The generic base sprite for the cell type.</param>
        /// <returns>The preview image name.</returns>
        public static string PreviewImageName(CellType cellType, CellEntityInfo? selection, MazeGameSettings? settings, string fallback) =>
            VariantImageName(selection) ?? BaseImageName(cellType, selection, settings) ?? fallback;

        /// <summary>
        /// The rig to display for a cell shared by multiple enemies: a rig with a distinct
        /// sprite (e.g. ghost) takes priority over the default goblin, so a mixed stack
        /// surfaces the special enemy; otherwise the first enemy's rig. Null for an empty
        /// stack.
        /// </summary>
        /// <param name="rigs">The rigs of the enemies on the cell, in order.</param>
        /// <returns>The rig whose sprite the stack should show, or null.</returns>
        public static EnemyType? DominantEnemyRig(IReadOnlyList<EnemyType?> rigs)
        {
            foreach (EnemyType? rig in rigs)
            {
                if (VariantImageName(new EnemyCellEntity { EnemyType = rig }) is not null)
                {
                    return rig;
                }
            }
            return rigs.Count > 0 ? rigs[0] : null;
        }
    }
}
