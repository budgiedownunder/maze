using Maze.Api;
using Maze.Maui.App;
using Maze.Maui.App.Models;
using Xunit;
using static Maze.Api.Maze;

namespace Maze.Maui.App.Tests.Controls
{
    /// <summary>
    /// Tests for <see cref="CellSprite.VariantImageName"/>, the editor's per-cell
    /// override → variant sprite resolver. Mirrors the web editor's sprite set: only
    /// ghost / potion / water / lava / iron-fence have a distinct 2D sprite.
    /// </summary>
    public class CellSpriteTests
    {
        [Fact]
        public void No_override_has_no_variant() => Assert.Null(CellSprite.VariantImageName(null));

        [Theory]
        [InlineData(EnemyType.Ghost, "ghost.png")]
        [InlineData(EnemyType.Goblin, null)] // the default rig renders the base sprite
        public void Enemy_variant(EnemyType type, string? expected) =>
            Assert.Equal(expected, CellSprite.VariantImageName(new EnemyCellEntity { EnemyType = type }));

        [Theory]
        [InlineData(HealthStyle.Potion, "potion.png")]
        [InlineData(HealthStyle.Heart, null)]
        public void Health_variant(HealthStyle style, string? expected) =>
            Assert.Equal(expected, CellSprite.VariantImageName(new HealthCellEntity { HealthStyle = style }));

        [Theory]
        [InlineData(WallType.Water, "water.png")]
        [InlineData(WallType.Lava, "lava.png")]
        [InlineData(WallType.IronFence, "iron_fence.png")]
        [InlineData(WallType.Brick, null)] // solid textures are 3D-only — base sprite in 2D
        [InlineData(WallType.DressedStone, null)]
        [InlineData(WallType.Wood, null)]
        [InlineData(WallType.Cobblestone, null)]
        public void Wall_variant(WallType type, string? expected) =>
            Assert.Equal(expected, CellSprite.VariantImageName(new WallCellEntity { WallType = type }));

        [Theory]
        [InlineData(TreasureStyle.Gold, "gold.png")]
        [InlineData(TreasureStyle.Diamonds, "diamonds.png")]
        [InlineData(TreasureStyle.Jewels, "jewels.png")]
        [InlineData(TreasureStyle.Silver, null)] // Silver is the default sprite (hardcoded base)
        public void Treasure_variant(TreasureStyle style, string? expected) =>
            Assert.Equal(expected, CellSprite.VariantImageName(new TreasureCellEntity { Style = style }));

        [Fact]
        public void Value_only_treasure_override_has_no_variant() =>
            // A value-only override (no style) is the Silver baseline — base sprite, badge marks it.
            Assert.Null(CellSprite.VariantImageName(new TreasureCellEntity { Value = 99 }));

        [Fact]
        public void Numeric_only_enemy_override_has_no_variant() =>
            // An override with only a damage value (no rig change) keeps the base sprite;
            // the badge alone marks it.
            Assert.Null(CellSprite.VariantImageName(new EnemyCellEntity { Damage = 3 }));

        [Fact]
        public void Key_and_door_overrides_have_no_2d_variant()
        {
            Assert.Null(CellSprite.VariantImageName(new KeyCellEntity { KeyHolder = KeyHolderStyle.Chest }));
            Assert.Null(CellSprite.VariantImageName(new DoorCellEntity { DoorStyle = DoorStyle.Portcullis }));
        }

        [Theory]
        [InlineData("water", "water.png")]
        [InlineData("lava", "lava.png")]
        [InlineData("iron_fence", "iron_fence.png")]
        [InlineData("brick", null)] // solid textures have no 2D sprite — hardcoded base
        [InlineData("wood", null)]
        public void Base_wall_from_settings(string wallType, string? expected) =>
            Assert.Equal(expected, CellSprite.BaseImageName(CellType.Wall, null, new MazeGameSettings { WallType = wallType }));

        [Theory]
        [InlineData("ghost", "ghost.png")]
        [InlineData("goblin", null)] // the default rig has no 2D sprite — hardcoded base
        public void Base_enemy_from_settings(string enemyType, string? expected) =>
            Assert.Equal(expected, CellSprite.BaseImageName(CellType.Enemy, null, new MazeGameSettings { EnemyType = enemyType }));

        [Theory]
        [InlineData("potion", "potion.png")]
        [InlineData("heart", null)] // the default style has no 2D sprite — hardcoded base
        public void Base_health_from_settings(string healthStyle, string? expected) =>
            Assert.Equal(expected, CellSprite.BaseImageName(CellType.Health, null, new MazeGameSettings { HealthStyle = healthStyle }));

        [Fact]
        public void Base_is_null_without_settings() =>
            Assert.Null(CellSprite.BaseImageName(CellType.Wall, null, null));

        [Fact]
        public void Base_ignores_non_feature_cell_types() =>
            Assert.Null(CellSprite.BaseImageName(CellType.Door, null, new MazeGameSettings { WallType = "lava" }));

        [Fact]
        public void Explicit_override_field_suppresses_the_maze_default()
        {
            // A per-cell override that explicitly sets the family's field wins: the maze
            // default is ignored (null) so the override's own resolution (variant or
            // hardcoded base) stands — mirroring the web editor's override-wins precedence.
            var lava = new MazeGameSettings { WallType = "lava" };
            Assert.Null(CellSprite.BaseImageName(CellType.Wall, new WallCellEntity { WallType = WallType.Brick }, lava));
            var ghost = new MazeGameSettings { EnemyType = "ghost" };
            Assert.Null(CellSprite.BaseImageName(CellType.Enemy, new EnemyCellEntity { EnemyType = EnemyType.Goblin }, ghost));
        }

        [Fact]
        public void A_field_less_override_still_inherits_the_maze_default() =>
            // An override with only a numeric field doesn't set the rig, so the cell still
            // inherits the maze default (matches the web editor).
            Assert.Equal("ghost.png",
                CellSprite.BaseImageName(CellType.Enemy, new EnemyCellEntity { Damage = 3 }, new MazeGameSettings { EnemyType = "ghost" }));

        [Fact]
        public void Default_base_change_triggers_on_a_2d_relevant_change()
        {
            // A wall special change, an enemy rig change, and a health style change each flip
            // a 2D base sprite, so a refresh is needed.
            Assert.True(CellSprite.MazeDefaultBaseChanged(new MazeGameSettings { WallType = "brick" }, new MazeGameSettings { WallType = "lava" }));
            Assert.True(CellSprite.MazeDefaultBaseChanged(new MazeGameSettings { EnemyType = "goblin" }, new MazeGameSettings { EnemyType = "ghost" }));
            Assert.True(CellSprite.MazeDefaultBaseChanged(new MazeGameSettings { HealthStyle = "heart" }, new MazeGameSettings { HealthStyle = "potion" }));
            Assert.True(CellSprite.MazeDefaultBaseChanged(new MazeGameSettings { WallType = "lava" }, new MazeGameSettings { WallType = "water" }));
        }

        [Fact]
        public void Default_base_change_skips_a_solid_to_solid_or_3d_only_change()
        {
            // brick -> wood are both solid (no 2D sprite), so no 2D base changed.
            Assert.False(CellSprite.MazeDefaultBaseChanged(new MazeGameSettings { WallType = "brick" }, new MazeGameSettings { WallType = "wood" }));
            // A 3D-only edit (sky / timer) leaves every 2D base unchanged.
            Assert.False(CellSprite.MazeDefaultBaseChanged(
                new MazeGameSettings { SkyType = "night", TimerSeconds = 60 },
                new MazeGameSettings { SkyType = "day", TimerSeconds = 120 }));
            // null (no settings) vs all-default tokens both render the hardcoded bases.
            Assert.False(CellSprite.MazeDefaultBaseChanged(null, new MazeGameSettings()));
        }

        [Fact]
        public void Live_enemy_uses_its_own_rig_then_the_maze_default()
        {
            var ghostMaze = new MazeGameSettings { EnemyType = "ghost" };
            var goblinMaze = new MazeGameSettings { EnemyType = "goblin" };
            // The enemy's own rig wins.
            Assert.Equal("ghost.png", CellSprite.LiveEnemyImageName(EnemyType.Ghost, goblinMaze));
            // An explicit goblin rig stays goblin even on a ghost maze (pin wins).
            Assert.Equal("enemy.png", CellSprite.LiveEnemyImageName(EnemyType.Goblin, ghostMaze));
            // A default (null) rig inherits the maze default.
            Assert.Equal("ghost.png", CellSprite.LiveEnemyImageName(null, ghostMaze));
            Assert.Equal("enemy.png", CellSprite.LiveEnemyImageName(null, goblinMaze));
            // No settings → the generic goblin.
            Assert.Equal("enemy.png", CellSprite.LiveEnemyImageName(null, null));
        }

        [Fact]
        public void Dominant_rig_prefers_a_distinctive_sprite_over_the_default()
        {
            // A ghost in the stack wins regardless of order; an all-default stack keeps the
            // first rig; an empty stack has no rig.
            Assert.Equal(EnemyType.Ghost, CellSprite.DominantEnemyRig(new EnemyType?[] { EnemyType.Goblin, EnemyType.Ghost }));
            Assert.Equal(EnemyType.Ghost, CellSprite.DominantEnemyRig(new EnemyType?[] { EnemyType.Ghost, EnemyType.Goblin }));
            Assert.Equal(EnemyType.Ghost, CellSprite.DominantEnemyRig(new EnemyType?[] { null, EnemyType.Ghost }));
            Assert.Equal(EnemyType.Goblin, CellSprite.DominantEnemyRig(new EnemyType?[] { EnemyType.Goblin, EnemyType.Goblin }));
            Assert.Null(CellSprite.DominantEnemyRig([]));
        }
    }
}
