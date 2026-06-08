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
    }
}
