using Maze.Api;
using Maze.Maui.App;
using Xunit;

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
    }
}
