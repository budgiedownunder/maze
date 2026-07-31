using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for <see cref="Play3dFailureReport"/> — the pure part of what the 3D
    /// game page does when a run dies. As with the other client helpers the page's
    /// WebView and navigation path is not exercised; these pin the player-facing
    /// message and the log's subject label.
    /// </summary>
    public class Play3dFailureReportTests
    {
        [Fact]
        public void AlertMessage_UsesTheReportedReason()
        {
            var failure = new GameFailure { Reason = "The game ran out of memory." };

            Assert.Equal("The game ran out of memory.", Play3dFailureReport.AlertMessage(failure));
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void AlertMessage_FallsBackWhenNoReasonWasReported(string reason)
        {
            // An unclassifiable failure still has to say something — a blank alert
            // reads as a broken app rather than a stopped game.
            var failure = new GameFailure { Reason = reason };

            Assert.Equal(GameFailure.GenericReason, Play3dFailureReport.AlertMessage(failure));
        }

        [Fact]
        public void AlertMessage_TrimsSurroundingWhitespace()
        {
            var failure = new GameFailure { Reason = "  The game stopped.  " };

            Assert.Equal("The game stopped.", Play3dFailureReport.AlertMessage(failure));
        }

        [Fact]
        public void Subject_NamesTheMazeWhenLaunchedFromOne()
        {
            Assert.Equal("maze m1", Play3dFailureReport.Subject("m1", null));
        }

        [Fact]
        public void Subject_NamesTheDefinitionWhenLaunchedFromOne()
        {
            Assert.Equal("definition g1", Play3dFailureReport.Subject(null, "g1"));
        }

        [Fact]
        public void Subject_PrefersTheMazeWhenBothAreSet()
        {
            // Mirrors the launch-URL builder, which takes the maze branch first.
            Assert.Equal("maze m1", Play3dFailureReport.Subject("m1", "g1"));
        }

        [Fact]
        public void Subject_ReportsTheBareLaunchWithNoSubject()
        {
            Assert.Equal("no subject", Play3dFailureReport.Subject(null, null));
            Assert.Equal("no subject", Play3dFailureReport.Subject("", ""));
        }
    }
}
