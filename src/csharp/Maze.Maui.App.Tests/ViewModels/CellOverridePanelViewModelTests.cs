using Maze.Api;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;
using CellType = Maze.Api.Maze.CellType;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for <see cref="CellOverridePanelViewModel"/> — the editor's per-cell
    /// override panel. Mirrors the web CellOverridePanel tests: seeding per type, live
    /// apply, clear-on-default, the two-tier wall, and reset.
    /// </summary>
    public class CellOverridePanelViewModelTests
    {
        private static (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) Build(CellEntityInfo? seed = null)
        {
            Mock<ICellOverrideEditor> editor = new();
            editor.Setup(e => e.GetCellOverride(It.IsAny<int>(), It.IsAny<int>())).Returns(seed);
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
            Assert.Equal(WallType.Lava, vm.SpecialWallType);
            Assert.Null(vm.WallTexture);
            Assert.False(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Seeds_a_solid_texture_under_wall_and_shows_the_texture_picker()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.DressedStone });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Null(vm.SpecialWallType);
            Assert.Equal(WallType.DressedStone, vm.WallTexture);
            Assert.True(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Applies_a_special_wall_type()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            vm.SpecialWallType = WallType.Water;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<WallCellEntity>(x => x.WallType == WallType.Water)), Times.Once);
            Assert.False(vm.IsWallTextureVisible);
        }

        [Fact]
        public void Applies_a_solid_texture_chosen_under_wall()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build();
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTexture = WallType.Brick;
            editor.Verify(e => e.SetCellOverride(1, 1,
                It.Is<WallCellEntity>(x => x.WallType == WallType.Brick)), Times.Once);
        }

        [Fact]
        public void Clears_the_override_when_texture_returns_to_default_under_wall()
        {
            (CellOverridePanelViewModel vm, Mock<ICellOverrideEditor> editor) = Build(new WallCellEntity { WallType = WallType.Wood });
            vm.LoadCell(1, 1, CellType.Wall);
            vm.WallTexture = null;
            editor.Verify(e => e.ClearCellOverride(1, 1), Times.Once);
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
        public void Wall_type_index_maps_special_types()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.Lava });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(2, vm.WallTypeIndex); // [Wall, Water, Lava, Iron Fence] → Lava
            vm.WallTypeIndex = 1; // Water
            Assert.Equal(WallType.Water, vm.SpecialWallType);
        }

        [Fact]
        public void Wall_texture_index_maps_solid_textures()
        {
            (CellOverridePanelViewModel vm, _) = Build(new WallCellEntity { WallType = WallType.DressedStone });
            vm.LoadCell(1, 1, CellType.Wall);
            Assert.Equal(2, vm.WallTextureIndex); // [Default, Brick, Dressed Stone, ...] → Dressed Stone
            vm.WallTextureIndex = 1; // Brick
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
            vm.SpecialWallType = WallType.Water;
            Assert.Equal("water.png", vm.WallPreviewImage);
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
    }
}
