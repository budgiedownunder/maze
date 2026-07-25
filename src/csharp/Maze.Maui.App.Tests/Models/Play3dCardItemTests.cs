using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for the pure card projections in <see cref="Play3dCardItem"/> — the
    /// definition / collection / featured-row mappers and the derived Play /
    /// Leaderboard flags. The image fetch and view-model wiring are not exercised here.
    /// </summary>
    public class Play3dCardItemTests
    {
        [Fact]
        public void FromDefinition_MapsFieldsAndFlags()
        {
            var def = new GameDefinition
            {
                Id = "g1",
                Name = "Deep Cave",
                Description = "A tricky descent",
                ImageUpdatedAt = "2026-03-01T00:00:00Z",
            };

            Play3dCardItem card = Play3dCardItem.FromDefinition(def);

            Assert.Equal(GameEntityKind.Definition, card.Kind);
            Assert.Equal("g1", card.Id);
            Assert.Equal("Deep Cave", card.Name);
            Assert.Equal("A tricky descent", card.Description);
            Assert.Equal("2026-03-01T00:00:00Z", card.ImageUpdatedAt);
            Assert.Null(card.PlayMode);
            Assert.False(card.IsCollection);
            Assert.True(card.ShowLeaderboard);
            Assert.Equal("workshop_game.png", card.PlaceholderImage);
        }

        [Fact]
        public void FromCollection_MapsFieldsAndFlags()
        {
            var collection = new GameCollection
            {
                Id = "c1",
                Name = "Difficulty",
                Description = "Easy to hard",
                ImageUpdatedAt = null,
                PlayMode = GameVocabulary.PlayMode.Campaign,
            };

            Play3dCardItem card = Play3dCardItem.FromCollection(collection);

            Assert.Equal(GameEntityKind.Collection, card.Kind);
            Assert.Equal("c1", card.Id);
            Assert.Equal("Difficulty", card.Name);
            Assert.Equal("Easy to hard", card.Description);
            Assert.Null(card.ImageUpdatedAt);
            Assert.Equal(GameVocabulary.PlayMode.Campaign, card.PlayMode);
            Assert.True(card.IsCollection);
            Assert.False(card.ShowLeaderboard);
            Assert.Equal("workshop_game_collection.png", card.PlaceholderImage);
        }

        [Fact]
        public void FromFeatured_DefinitionPayload_MapsToDefinitionCard()
        {
            var item = new FeaturedGameItem
            {
                Kind = "definition",
                Definition = new GameDefinition { Id = "g1", Name = "Deep Cave" },
            };

            Play3dCardItem? card = Play3dCardItem.FromFeatured(item);

            Assert.NotNull(card);
            Assert.Equal(GameEntityKind.Definition, card!.Kind);
            Assert.Equal("g1", card.Id);
        }

        [Fact]
        public void FromFeatured_CollectionPayload_MapsToCollectionCard()
        {
            var item = new FeaturedGameItem
            {
                Kind = "collection",
                Collection = new GameCollection { Id = "c1", Name = "Difficulty" },
            };

            Play3dCardItem? card = Play3dCardItem.FromFeatured(item);

            Assert.NotNull(card);
            Assert.Equal(GameEntityKind.Collection, card!.Kind);
            Assert.Equal("c1", card.Id);
        }

        [Fact]
        public void FromFeatured_NoPayload_ReturnsNull()
        {
            var item = new FeaturedGameItem { Kind = "definition" };

            Assert.Null(Play3dCardItem.FromFeatured(item));
        }
    }
}
