using System.Collections.Generic;
using Maze.Api;

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
            _ => null
        };

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
