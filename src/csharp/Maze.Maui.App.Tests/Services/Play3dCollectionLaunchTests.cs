using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure collection-play decision in <see cref="Play3dCollectionLaunch"/>:
    /// none accessible → guarded, exactly one → launch it, more than one → wait for
    /// the Arcade / Campaign picker.
    /// </summary>
    public class Play3dCollectionLaunchTests
    {
        private static GameDefinition Def(string id) => new() { Id = id, Name = id };

        [Fact]
        public void Resolve_Empty_IsNoneAccessible()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new List<GameDefinition>());

            Assert.Equal(Play3dCollectionPlayKind.NoneAccessible, play.Kind);
            Assert.Null(play.DefinitionId);
        }

        [Fact]
        public void Resolve_SingleMember_LaunchesThatMember()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new[] { Def("g1") });

            Assert.Equal(Play3dCollectionPlayKind.LaunchSingle, play.Kind);
            Assert.Equal("g1", play.DefinitionId);
        }

        [Fact]
        public void Resolve_MultipleMembers_IsUnsupported()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new[] { Def("g1"), Def("g2") });

            Assert.Equal(Play3dCollectionPlayKind.MultiMemberUnsupported, play.Kind);
            Assert.Null(play.DefinitionId);
        }
    }
}
