using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure request-path helpers in
    /// <see cref="GameLibraryRequestPaths"/>. As with the other client services the
    /// HTTP send path itself is not unit-tested; these pin the query-string
    /// assembly, param encoding, and the scope/sort/kind token mapping.
    /// </summary>
    public class GameLibraryRequestPathsTests
    {
        [Fact]
        public void BuildDefinitionListPath_NoArgs_IsBarePath()
        {
            Assert.Equal("game-definitions", GameLibraryRequestPaths.BuildDefinitionListPath(null, null, null, null, null));
        }

        [Fact]
        public void BuildDefinitionListPath_AppendsAllParamsInOrder()
        {
            string path = GameLibraryRequestPaths.BuildDefinitionListPath(GameListScope.Public, "deep cave", GameListSort.Newest, 20, 40);
            Assert.Equal("game-definitions?scope=public&q=deep%20cave&sort=newest&limit=20&offset=40", path);
        }

        [Fact]
        public void BuildCollectionListPath_ScopeOnly()
        {
            Assert.Equal("game-collections?scope=mine", GameLibraryRequestPaths.BuildCollectionListPath(GameListScope.Mine, null, null, null, null));
        }

        [Fact]
        public void BuildCollectionListPath_OmitsBlankQuery()
        {
            // An empty query string is dropped (not sent as `q=`).
            Assert.Equal("game-collections?scope=shared", GameLibraryRequestPaths.BuildCollectionListPath(GameListScope.Shared, "", null, null, null));
        }

        [Fact]
        public void BuildDefinitionPath_EncodesId()
        {
            Assert.Equal("game-definitions/a%20b%2Fc", GameLibraryRequestPaths.BuildDefinitionPath("a b/c"));
        }

        [Fact]
        public void BuildCollectionPath_EncodesId()
        {
            Assert.Equal("game-collections/c1", GameLibraryRequestPaths.BuildCollectionPath("c1"));
        }

        [Fact]
        public void BuildFeaturedPath_NoPaging_IsBarePath()
        {
            Assert.Equal("featured-game-items", GameLibraryRequestPaths.BuildFeaturedPath(null, null));
        }

        [Fact]
        public void BuildFeaturedPath_WithPaging()
        {
            Assert.Equal("featured-game-items?limit=10&offset=0", GameLibraryRequestPaths.BuildFeaturedPath(10, 0));
        }

        [Fact]
        public void BuildImagePath_DefinitionWithoutMarker()
        {
            Assert.Equal("game-definitions/g1/image", GameLibraryRequestPaths.BuildImagePath(GameEntityKind.Definition, "g1", null));
        }

        [Fact]
        public void BuildImagePath_CollectionWithMarkerCacheBuster()
        {
            string path = GameLibraryRequestPaths.BuildImagePath(GameEntityKind.Collection, "c1", "2026-03-01T00:00:00Z");
            Assert.Equal("game-collections/c1/image?v=2026-03-01T00%3A00%3A00Z", path);
        }

        [Theory]
        [InlineData(GameListScope.Visible, "visible")]
        [InlineData(GameListScope.Mine, "mine")]
        [InlineData(GameListScope.Shared, "shared")]
        [InlineData(GameListScope.Public, "public")]
        public void ScopeTokens_MatchServer(GameListScope scope, string expected)
        {
            Assert.Equal(expected, scope.ToQueryValue());
        }

        [Theory]
        [InlineData(GameEntityKind.Definition, "definition", "game-definitions")]
        [InlineData(GameEntityKind.Collection, "collection", "game-collections")]
        public void KindTokens_MatchServer(GameEntityKind kind, string wire, string segment)
        {
            Assert.Equal(wire, kind.ToWireString());
            Assert.Equal(segment, kind.ToPathSegment());
        }
    }
}
