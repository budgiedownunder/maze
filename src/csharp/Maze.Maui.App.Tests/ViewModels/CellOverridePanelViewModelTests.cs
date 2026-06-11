using Maze.Api;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;
using CellType = Maze.Api.Maze.CellType;
using WallKind = Maze.Maui.App.ViewModels.CellOverridePanelViewModel.WallKind;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for <see cref="CellOverridePanelViewModel"/> — the editor's per-cell
    /// override panel. Mirrors the web CellOverridePanel tests: seeding per type, live
    /// apply, clear-on-default, the two-tier wall, and reset.
    /// </summary>
    public class CellOverridePanelViewModelTests
    {
        private static readonly string[] ExpectedWallTypeOptions = { "Default", "Wall", "Water", "Lava", "Iron Fence" };

        private static (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) Build(CellEntityInfo? seed = null, MazeGameSettings? settings = null)
        {
            Mock<ICellOverrideEditor> editor = new();
            editor.Setup(e => e.GetCellOverride(It.IsAny<int>(), It.IsAny<int>())).Returns(seed);
            editor.Setup(e => e.GameSettings).Returns(settings);
            return (new CellOverridePanelViewModel(editor.Object), editor);
        }

        [Fact]
        public void LoadCell_shows_the_type_and_one_based_coordinates_in_the_title()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(2, 3, CellType.Enemy);
            Assert.Equal("Enemy [2,3]", vm.Title);
            Assert.True(vm.IsVisible);
            Assert.True(vm.IsEnemy);
        }

        [Fact]
        public void Panel_is_hidden_for_non_overridable_cells()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Start);
            Assert.False(vm.IsVisible);
        }

        [Fact]
        public void Seeds_the_fields_from_an_existing_enemy_override()
        {
            (CellOverridePanelViewModel vm, _) = Build(new EnemyCellEntity { EnemyType = EnemyType.Ghost, Damage = 2 });
            vm.LoadCell(1, 1, CellType.Enemy);
            Assert.Equal(EnemyType.Ghost, vm.EnemyTypeValue);
            Assert.Equal("2", vm.DamageText);
        }

        [Fact]
        public void Applies_an_override_live_when_a_rig_field_changes()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Enemy);
            vm.EnemyTypeValue = EnemyType.Ghost;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<EnemyCellEntity>(x => x.EnemyType == EnemyType.Ghost && x.Damage == null && x.MovePeriodMs == null)), Times.Once);
            editor.Verify(e => e.RefreshCellContent(1, 1), Times.AtLeastOnce);
        }

        [Fact]
        public void Applies_a_numeric_override_live_as_it_is_typed()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(4, 5, CellType.Health);
            vm.HealAmountText = "3";
            editor.Verify(e => e.SetCellOverride(4, 5,
                It.Is<HealthCellEntity>(x => x.HealAmount == 3 && x.HealthStyle == null)), Times.Once);
        }

        [Fact]
        public void Clears_the_override_when_the_last_set_field_reverts_to_default()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new KeyCellEntity { KeyHolder = KeyHolderStyle.Chest });
            vm.LoadCell(1, 1, CellType.Key);
            vm.KeyHolderValue = null;
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.Once);
        }

        [Fact]
        public void Reset_clears_the_override_and_the_fields()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new DoorCellEntity { DoorStyle = DoorStyle.Portcullis });
            vm.LoadCell(1, 1, CellType.Door);
            Assert.Equal(DoorStyle.Portcullis, vm.DoorStyleValue);
            vm.ResetCommand.Execute(null);
            Assert.Null(vm.DoorStyleValue);
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.Once);
        }

        [Fact]
        public void Seeds_a_special_wall_type_and_hides_the_texture_picker()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(WallKind.Lava, vm.WallTypeKind);
            Assert.Null(vm.WallTexture);
            Assert.False(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Seeds_a_solid_override_as_wall_plus_that_texture()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.DressedStone });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(WallKind.Wall, vm.WallTypeKind);
            Assert.Equal(WallType.DressedStone, vm.WallTexture);
            Assert.True(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Defaults_a_fresh_wall_cell_to_the_inherit_kind()
        {
            // No game settings ⇒ the effective maze default wall is solid (brick), so the
            // texture picker is offered under "Default" for a per-cell texture override.
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(WallKind.Default, vm.WallTypeKind);
            Assert.True(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Wall_type_options_list_default_wall_and_the_specials()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(ExpectedWallTypeOptions, vm.WallTypeOptions);
        }

        [Fact]
        public void Selecting_default_inherits_and_clears_the_override()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTypeKind = WallKind.Default;
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.AtLeastOnce);
        }

        [Fact]
        public void Selecting_wall_forces_a_solid_and_shows_the_texture_picker()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTypeKind = WallKind.Wall;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<WallCellEntity>(x => x.WallType == WallType.Brick)), Times.AtLeastOnce);
            Assert.True(vm.IsWallTextureVisible);
            // "Wall" forces a concrete texture — no inherit option in tier 2.
            Assert.DoesNotContain("Default", vm.WallTextureOptions);
        }

        [Fact]
        public void Applies_a_special_wall_type()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTypeKind = WallKind.Water;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<WallCellEntity>(x => x.WallType == WallType.Water)), Times.AtLeastOnce);
            Assert.False(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Under_default_with_a_solid_maze_default_a_texture_overrides_just_this_cell()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTexture = WallType.Wood;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<WallCellEntity>(x => x.WallType == WallType.Wood)), Times.AtLeastOnce);
            vm.WallTexture = null; // back to "Default texture" = inherit
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.AtLeastOnce);
        }

        [Fact]
        public void Clears_the_override_by_switching_from_wall_to_default()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new WallCellEntity { WallType = WallType.Wood });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(WallKind.Wall, vm.WallTypeKind);
            vm.WallTypeKind = WallKind.Default;
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.AtLeastOnce);
        }

        [Theory]
        [InlineData("brick", true)]   // solid maze default ⇒ a texture override is offered under Default
        [InlineData("lava", false)]   // special maze default ⇒ the cell inherits that look, no texture picker
        public void Texture_visibility_under_default_follows_the_maze_default(string mazeWall, bool visible)
        {
            (CellOverridePanelViewModel vm, _) = Build(settings: new MazeGameSettings { WallType = mazeWall });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(WallKind.Default, vm.WallTypeKind);
            Assert.Equal(visible, vm.IsWallTextureVisible);
        }

        [Fact]
        public void Texture_picker_shows_under_wall_even_when_the_maze_default_is_special()
        {
            (CellOverridePanelViewModel vm, _) = Build(settings: new MazeGameSettings { WallType = "lava" });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.False(vm.IsWallTextureVisible);
            vm.WallTypeKind = WallKind.Wall;
            Assert.True(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Seeding_does_not_apply_or_clear()
        {
            // LoadCell programmatically seeds the fields; that must not write back.
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new EnemyCellEntity { EnemyType = EnemyType.Ghost });
            vm.LoadCell(1, 1, CellType.Enemy);
            editor.Verify(e => e.SetCellOverride(It.IsAny<int>(), It.IsAny<int>(), It.IsAny<CellEntityInfo>()), Times.Never);
            editor.Verify(e => e.ClearCellOverride(It.IsAny<int>(), It.IsAny<int>()), Times.Never);
        }

        [Fact]
        public void Enemy_type_index_round_trips_through_the_value()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Enemy);
            Assert.Equal(0, vm.EnemyTypeIndex); // Default
            vm.EnemyTypeIndex = 2; // [Default, Goblin, Ghost] → Ghost
            Assert.Equal(EnemyType.Ghost, vm.EnemyTypeValue);
        }

        [Fact]
        public void Wall_type_index_maps_the_tier1_kinds()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(3, vm.WallTypeIndex); // [Default, Wall, Water, Lava, Iron Fence] → Lava
            vm.WallTypeIndex = 2; // Water
            Assert.Equal(WallKind.Water, vm.WallTypeKind);
            vm.WallTypeIndex = 0; // Default
            Assert.Equal(WallKind.Default, vm.WallTypeKind);
        }

        [Fact]
        public void Wall_texture_index_maps_solid_textures_under_wall()
        {
            // Seeding a solid override puts the panel in "Wall" kind, whose tier-2 list has
            // no "Default" entry, so the solid textures map from index 0.
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.DressedStone });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(1, vm.WallTextureIndex); // [Brick, Dressed Stone, Wood, Cobblestone] → Dressed Stone
            vm.WallTextureIndex = 0; // Brick
            Assert.Equal(WallType.Brick, vm.WallTexture);
        }

        [Fact]
        public void Increment_damage_from_blank_sets_one_and_applies()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Enemy);
            vm.IncrementDamageCommand.Execute(null);
            Assert.Equal("1", vm.DamageText);
            editor.Verify(e => e.SetCellOverride(1, 1, It.Is<EnemyCellEntity>(x => x.Damage == 1)), Times.Once);
        }

        [Fact]
        public void Decrement_damage_clamps_at_zero()
        {
            (CellOverridePanelViewModel vm, _) = Build(new EnemyCellEntity { Damage = 1 });
            vm.LoadCell(1, 1, CellType.Enemy);
            vm.DecrementDamageCommand.Execute(null);
            Assert.Equal("0", vm.DamageText);
            vm.DecrementDamageCommand.Execute(null);
            Assert.Equal("0", vm.DamageText); // clamped at zero
        }

        [Fact]
        public void Decrement_a_blank_field_keeps_it_blank()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Health);
            vm.DecrementHealAmountCommand.Execute(null);
            Assert.Equal("", vm.HealAmountText);
        }

        [Fact]
        public void Enemy_preview_reflects_the_selected_rig()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Enemy);
            Assert.Equal("enemy.png", vm.EnemyPreviewImage); // default goblin
            vm.EnemyTypeValue = EnemyType.Ghost;
            Assert.Equal("ghost.png", vm.EnemyPreviewImage);
        }

        [Fact]
        public void Wall_preview_reflects_the_special_type()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal("wall.png", vm.WallPreviewImage); // default / solid
            vm.WallTypeKind = WallKind.Water;
            Assert.Equal("water.png", vm.WallPreviewImage);
        }

        [Fact]
        public void Previews_reflect_the_maze_default_when_no_override_is_set()
        {
            // Wall: a lava maze default previews lava under the inherit ("Default") kind.
            (CellOverridePanelViewModel wall, _) = Build(settings: new MazeGameSettings { WallType = "lava" });
            wall.LoadCell(1, 1, CellType.Wall);
            Assert.Equal("lava.png", wall.WallPreviewImage);

            // Enemy: a ghost maze default previews ghost under "Default".
            (CellOverridePanelViewModel enemy, _) = Build(settings: new MazeGameSettings { EnemyType = "ghost" });
            enemy.LoadCell(1, 1, CellType.Enemy);
            Assert.Equal("ghost.png", enemy.EnemyPreviewImage);

            // Health: a potion maze default previews potion under "Default".
            (CellOverridePanelViewModel health, _) = Build(settings: new MazeGameSettings { HealthStyle = "potion" });
            health.LoadCell(1, 1, CellType.Health);
            Assert.Equal("potion.png", health.HealthPreviewImage);
        }

        [Fact]
        public void A_single_cell_is_not_multi_cell()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(3, 4, CellType.Enemy);
            Assert.Equal(1, vm.SelectionCount);
            Assert.False(vm.IsMultiCell);
        }

        [Fact]
        public void A_rectangular_selection_reports_its_count_and_title()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, 1, 3, CellType.Wall); // a 1x3 block
            Assert.Equal(3, vm.SelectionCount);
            Assert.True(vm.IsMultiCell);
            Assert.Equal("Apply to all 3 cells", vm.ApplyToAllText);
            Assert.Contains("+2 more", vm.Title);
        }

        [Fact]
        public void Apply_to_all_stamps_the_top_left_override_across_the_block()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, 2, 2, CellType.Wall); // a 2x2 block (top-left already carries the override)
            vm.ApplyToAllCommand.Execute(null);
            editor.Verify(e => e.SetCellOverride(It.IsAny<int>(), It.IsAny<int>(),
                It.Is<WallCellEntity>(x => x.WallType == WallType.Lava)), Times.Exactly(3));
        }

        [Fact]
        public void Apply_to_all_clears_the_block_when_the_top_left_has_no_override()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(); // no override
            vm.LoadCell(1, 1, 2, 2, CellType.Wall);
            vm.ApplyToAllCommand.Execute(null);
            editor.Verify(e => e.ClearCellOverride(It.IsAny<int>(), It.IsAny<int>()), Times.Exactly(3));
        }

        [Fact]
        public void Reset_clears_the_override_on_every_cell_in_the_selection()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, 2, 2, CellType.Wall); // a 2x2 block
            vm.ResetCommand.Execute(null);
            editor.Verify(e => e.ClearCellOverride(It.IsAny<int>(), It.IsAny<int>()), Times.Exactly(4));
        }

        [Fact]
        public void OverrideChanged_fires_on_a_live_field_change()
        {
            (CellOverridePanelViewModel vm, _) = Build();
            vm.LoadCell(1, 1, CellType.Enemy);
            int changes = 0;
            vm.OverrideChanged += (_, _) => changes++;
            vm.EnemyTypeValue = EnemyType.Ghost;
            Assert.True(changes >= 1);
        }

        [Fact]
        public void OverrideChanged_fires_on_reset_and_apply_to_all()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, 2, 2, CellType.Wall);
            int changes = 0;
            vm.OverrideChanged += (_, _) => changes++;
            vm.ApplyToAllCommand.Execute(null);
            vm.ResetCommand.Execute(null);
            Assert.Equal(2, changes);
        }

        [Fact]
        public void OverrideChanged_does_not_fire_during_seeding()
        {
            // LoadCell seeds the fields from the existing override; that must not be reported
            // as a change (it would spuriously dirty the maze on selection).
            (CellOverridePanelViewModel vm, _) = Build(new EnemyCellEntity { EnemyType = EnemyType.Ghost, Damage = 2 });
            int changes = 0;
            vm.OverrideChanged += (_, _) => changes++;
            vm.LoadCell(1, 1, CellType.Enemy);
            Assert.Equal(0, changes);
        }
    }
}
