using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the shared Play 3D launch chooser orchestration: Run uses the maze's
    /// saved settings (or defaults), Custom Run returns the one-off settings seeded from
    /// saved, a cancelled Custom Run loops back to the chooser, and Cancel aborts.
    /// </summary>
    public class Play3dLaunchResolverTests
    {
        [Fact]
        public async Task Run_ReturnsSavedSettings()
        {
            var saved = new MazeGameSettings { SkyType = "dungeon" };
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.Run);

            var result = await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", saved);

            Assert.Same(saved, result);
            dialog.Verify(d => d.ShowMazeGameSettingsAsync(It.IsAny<string?>(), It.IsAny<MazeGameSettings?>()), Times.Never);
        }

        [Fact]
        public async Task Run_WithNullSaved_ReturnsDefaults()
        {
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.Run);

            var result = await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", null);

            Assert.NotNull(result);
            Assert.Equal("night", result!.SkyType); // defaults
        }

        [Fact]
        public async Task CustomRun_ReturnsTheOneOffSettings()
        {
            var saved = new MazeGameSettings { SkyType = "dungeon" };
            var custom = new MazeGameSettings { SkyType = "day" };
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.CustomRun);
            dialog.Setup(d => d.ShowMazeGameSettingsAsync(It.IsAny<string?>(), It.IsAny<MazeGameSettings?>()))
                  .ReturnsAsync(custom);

            var result = await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", saved);

            Assert.Same(custom, result);
        }

        [Fact]
        public async Task CustomRun_SeedsTheSettingsPopupFromSaved()
        {
            var saved = new MazeGameSettings { SkyType = "dungeon" };
            MazeGameSettings? seed = null;
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.CustomRun);
            dialog.Setup(d => d.ShowMazeGameSettingsAsync(It.IsAny<string?>(), It.IsAny<MazeGameSettings?>()))
                  .Callback<string?, MazeGameSettings?>((_, s) => seed = s)
                  .ReturnsAsync(new MazeGameSettings());

            await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", saved);

            Assert.Same(saved, seed);
        }

        [Fact]
        public async Task CustomRunCancelled_ReturnsToChooser()
        {
            var saved = new MazeGameSettings();
            var dialog = new Mock<IDialogService>();
            dialog.SetupSequence(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.CustomRun)
                  .ReturnsAsync(Play3dLaunchChoice.Run);
            dialog.Setup(d => d.ShowMazeGameSettingsAsync(It.IsAny<string?>(), It.IsAny<MazeGameSettings?>()))
                  .ReturnsAsync((MazeGameSettings?)null); // cancelled

            var result = await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", saved);

            Assert.Same(saved, result); // looped back to the chooser, then Run
            dialog.Verify(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()), Times.Exactly(2));
        }

        [Fact]
        public async Task Cancel_ReturnsNull()
        {
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowPlay3dLaunchChooserAsync(It.IsAny<string?>()))
                  .ReturnsAsync(Play3dLaunchChoice.Cancel);

            var result = await Play3dLaunchResolver.ResolveAsync(dialog.Object, "Alpha", new MazeGameSettings());

            Assert.Null(result);
        }
    }
}
