using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure collection-play decision in <see cref="Play3dCollectionLaunch"/>:
    /// none accessible → guarded, exactly one → launch it, more than one → the Arcade
    /// picker or the Campaign progression per the collection's play mode.
    /// </summary>
    public class Play3dCollectionLaunchTests
    {
        private static GameDefinition Def(string id) => new() { Id = id, Name = id };

        [Fact]
        public void Resolve_Empty_IsNoneAccessible()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new List<GameDefinition>(), GameVocabulary.PlayMode.Arcade);

            Assert.Equal(Play3dCollectionPlayKind.NoneAccessible, play.Kind);
            Assert.Null(play.DefinitionId);
        }

        [Fact]
        public void Resolve_SingleMember_LaunchesThatMember_RegardlessOfMode()
        {
            Play3dCollectionPlay arcade = Play3dCollectionLaunch.Resolve(new[] { Def("g1") }, GameVocabulary.PlayMode.Arcade);
            Play3dCollectionPlay campaign = Play3dCollectionLaunch.Resolve(new[] { Def("g1") }, GameVocabulary.PlayMode.Campaign);

            Assert.Equal(Play3dCollectionPlayKind.LaunchSingle, arcade.Kind);
            Assert.Equal("g1", arcade.DefinitionId);
            Assert.Equal(Play3dCollectionPlayKind.LaunchSingle, campaign.Kind);
            Assert.Equal("g1", campaign.DefinitionId);
        }

        [Fact]
        public void Resolve_MultipleArcadeMembers_OpensArcadePicker()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new[] { Def("g1"), Def("g2") }, GameVocabulary.PlayMode.Arcade);

            Assert.Equal(Play3dCollectionPlayKind.Arcade, play.Kind);
            Assert.Null(play.DefinitionId);
        }

        [Fact]
        public void Resolve_MultipleCampaignMembers_OpensCampaign()
        {
            Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(new[] { Def("g1"), Def("g2") }, GameVocabulary.PlayMode.Campaign);

            Assert.Equal(Play3dCollectionPlayKind.Campaign, play.Kind);
            Assert.Null(play.DefinitionId);
        }
    }
}
