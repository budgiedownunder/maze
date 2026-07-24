using System.Text.Json;
using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Pins the JSON contract of the read-side game-library DTOs — the game
    /// entities are camelCase on the wire, so each property maps explicitly via
    /// <c>[JsonPropertyName]</c>. The opaque <c>config</c> round-trips as a raw
    /// <see cref="JsonElement"/>.
    /// </summary>
    public class GameLibraryDtoTests
    {
        [Fact]
        public void GamePlayResponse_DeserializesCamelCaseAndFlattenedFields()
        {
            const string json = """
            {
              "id": "g1", "ownerId": "o1", "name": "Tower",
              "description": null, "visibility": "public", "seed": 8080808,
              "rotation": "daily", "config": { "rows": 9, "cols": 9 },
              "imageUpdatedAt": "2026-03-01T00:00:00Z",
              "createdAt": "2026-01-01T00:00:00Z", "updatedAt": "2026-01-02T00:00:00Z",
              "challengeKey": "def:g1:2026-07-24", "leaderboardTracked": true
            }
            """;

            var play = JsonSerializer.Deserialize<GamePlayResponse>(json)!;

            Assert.Equal("g1", play.Id);
            Assert.Equal("o1", play.OwnerId);
            Assert.Equal(GameVocabulary.Visibility.Public, play.Visibility);
            Assert.Equal(GameVocabulary.Rotation.Daily, play.Rotation);
            Assert.Equal(8080808UL, play.Seed);
            Assert.Null(play.Description);
            Assert.Equal("2026-03-01T00:00:00Z", play.ImageUpdatedAt);
            // The opaque config survives as a raw JSON element.
            Assert.Equal(9, play.Config.GetProperty("rows").GetInt32());
            // The flattened play-fetch extras.
            Assert.Equal("def:g1:2026-07-24", play.ChallengeKey);
            Assert.True(play.LeaderboardTracked);
        }

        [Fact]
        public void GameDefinitionListResponse_DeserializesPage()
        {
            const string json = """
            { "definitions": [ { "id": "g1", "name": "A" }, { "id": "g2", "name": "B" } ],
              "limit": 20, "offset": 0, "hasMore": true }
            """;

            var page = JsonSerializer.Deserialize<GameDefinitionListResponse>(json)!;

            Assert.Equal(2, page.Definitions.Count);
            Assert.Equal("g2", page.Definitions[1].Id);
            Assert.Equal(20, page.Limit);
            Assert.True(page.HasMore);
        }

        [Fact]
        public void GameCollectionDetailResponse_HydratesMembersAndPlayMode()
        {
            const string json = """
            { "id": "c1", "ownerId": "o1", "name": "Campaign",
              "visibility": "curated", "playMode": "campaign",
              "createdAt": "2026-01-01T00:00:00Z", "updatedAt": "2026-01-01T00:00:00Z",
              "definitions": [ { "id": "g1", "name": "Level 1" }, { "id": "g2", "name": "Level 2" } ] }
            """;

            var detail = JsonSerializer.Deserialize<GameCollectionDetailResponse>(json)!;

            Assert.Equal("c1", detail.Id);
            Assert.Equal(GameVocabulary.PlayMode.Campaign, detail.PlayMode);
            Assert.Equal(GameVocabulary.Visibility.Curated, detail.Visibility);
            Assert.Equal(2, detail.Definitions.Count);
            Assert.Equal("Level 2", detail.Definitions[1].Name);
        }

        [Fact]
        public void GameCollection_DeserializesItemsAndMarkers()
        {
            const string json = """
            { "id": "c1", "ownerId": "o1", "name": "Set",
              "visibility": "shared", "playMode": "arcade",
              "imageUpdatedAt": "2026-03-02T00:00:00Z",
              "items": [ { "definitionId": "g1", "sortOrder": 0 }, { "definitionId": "g2", "sortOrder": 1 } ],
              "createdAt": "2026-01-01T00:00:00Z", "updatedAt": "2026-01-01T00:00:00Z" }
            """;

            var collection = JsonSerializer.Deserialize<GameCollection>(json)!;

            Assert.Equal(GameVocabulary.PlayMode.Arcade, collection.PlayMode);
            Assert.Equal("2026-03-02T00:00:00Z", collection.ImageUpdatedAt);
            Assert.Equal(2, collection.Items.Count);
            Assert.Equal("g2", collection.Items[1].DefinitionId);
            Assert.Equal(1u, collection.Items[1].SortOrder);
        }

        [Fact]
        public void FeaturedGameItemsListResponse_DeserializesMixedKinds()
        {
            const string json = """
            { "items": [
                { "kind": "definition", "ownerUsername": "ann", "definition": { "id": "g1", "name": "A" } },
                { "kind": "collection", "ownerUsername": "bob", "collection": { "id": "c1", "name": "Bundle" } }
              ], "limit": 20, "offset": 0, "hasMore": false }
            """;

            var page = JsonSerializer.Deserialize<FeaturedGameItemsListResponse>(json)!;

            Assert.Equal(2, page.Items.Count);
            Assert.Equal("definition", page.Items[0].Kind);
            Assert.Equal("A", page.Items[0].Definition!.Name);
            Assert.Null(page.Items[0].Collection);
            Assert.Equal("collection", page.Items[1].Kind);
            Assert.Equal("Bundle", page.Items[1].Collection!.Name);
            Assert.Null(page.Items[1].Definition);
            Assert.False(page.HasMore);
        }
    }
}
