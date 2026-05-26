using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for the per-launch settings POCO that the Play 3D popup
    /// emits and the game-launch URL consumes. Coverage focuses on the
    /// wire-string emission (<see cref="Play3dCustomLaunchSettings.ToQueryString"/>)
    /// and the enum validators the store uses to reject stale values —
    /// these are the bits the wasm boundary at <c>/game/index.html</c>
    /// depends on for `doorStyle` and `keyHolder`, the two parity
    /// dropdowns added in 8I.
    /// </summary>
    public class Play3dCustomLaunchSettingsTests
    {
        [Fact]
        public void Defaults_MatchReactDefaults()
        {
            var s = new Play3dCustomLaunchSettings();
            Assert.Equal("night", s.SkyType);
            Assert.Equal("brick", s.WallType);
            Assert.Equal("swing", s.DoorStyle);
            Assert.Equal("pedestal", s.KeyHolder);
            Assert.False(s.WallTint);
            Assert.False(s.WallMaterialVariation);
            Assert.True(s.DeadEndObjects);
            Assert.True(s.WallDecorations);
            Assert.True(s.FloorAccents);
            Assert.Equal(60, s.TimerSeconds);
        }

        [Fact]
        public void ToQueryString_EmitsDoorStyleAndKeyHolderInTheCamelCaseGameWireFormat()
        {
            var s = new Play3dCustomLaunchSettings
            {
                SkyType = "day",
                WallType = "dressed_stone",
                DoorStyle = "portcullis",
                KeyHolder = "chest",
                WallTint = true,
                WallMaterialVariation = false,
                DeadEndObjects = false,
                WallDecorations = true,
                FloorAccents = false,
                TimerSeconds = 90,
            };
            string q = s.ToQueryString();
            // Field names match the camelCase tokens /game/index.html reads
            // and forwards into the Bevy WASM StartConfig.
            Assert.Contains("doorStyle=portcullis", q);
            Assert.Contains("keyHolder=chest", q);
            Assert.Contains("skyType=day", q);
            Assert.Contains("wallType=dressed_stone", q);
            Assert.Contains("wallTint=1", q);
            Assert.Contains("wallMaterialVariation=0", q);
            Assert.Contains("deadEndObjects=0", q);
            Assert.Contains("wallDecorations=1", q);
            Assert.Contains("floorAccents=0", q);
            Assert.Contains("timerSeconds=90", q);
        }

        [Theory]
        [InlineData("swing", true)]
        [InlineData("slide", true)]
        [InlineData("portcullis", true)]
        [InlineData("dissolve", true)]
        [InlineData("hinged_swing", false)] // older internal name; not on the wire
        [InlineData("", false)]
        [InlineData("SWING", false)]        // case-sensitive wire format
        public void IsValidDoorStyle_AcceptsCurrentWireVariantsOnly(string s, bool expected)
        {
            Assert.Equal(expected, Play3dCustomLaunchSettings.IsValidDoorStyle(s));
        }

        [Theory]
        [InlineData("pedestal", true)]
        [InlineData("chest", true)]
        [InlineData("floating_key", true)]
        [InlineData("floatingkey", false)]  // missing underscore
        [InlineData("", false)]
        [InlineData("Pedestal", false)]     // case-sensitive
        public void IsValidKeyHolder_AcceptsCurrentWireVariantsOnly(string s, bool expected)
        {
            Assert.Equal(expected, Play3dCustomLaunchSettings.IsValidKeyHolder(s));
        }
    }
}
