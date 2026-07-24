using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure hosted-game URL helper in <see cref="Play3dGameHostUrl"/>.
    /// As with the other client helpers the WebView send path itself is not
    /// exercised; these pin the base-URL derivation (stripping <c>/api/…</c>), the
    /// per-branch query assembly (<c>?id=</c> / <c>?difficulty=</c> / <c>?def=</c> /
    /// <c>&amp;t=</c>), and id/param encoding.
    /// </summary>
    public class Play3dGameHostUrlTests
    {
        private const string ApiRoot = "https://maze.example.com/api/v1/";

        [Fact]
        public void BuildBaseUrl_StripsApiSegment()
        {
            Assert.Equal("https://maze.example.com/game/", Play3dGameHostUrl.BuildBaseUrl(ApiRoot));
        }

        [Fact]
        public void BuildBaseUrl_NoApiSegment_AppendsGame()
        {
            Assert.Equal("https://maze.example.com/game/", Play3dGameHostUrl.BuildBaseUrl("https://maze.example.com/"));
        }

        [Fact]
        public void BuildForDefinition_AppendsDefAndToken()
        {
            Assert.Equal(
                "https://maze.example.com/game/?def=g1&t=abc.def.ghi",
                Play3dGameHostUrl.BuildForDefinition(ApiRoot, "g1", "abc.def.ghi"));
        }

        [Fact]
        public void BuildForDefinition_EncodesId()
        {
            Assert.Equal(
                "https://maze.example.com/game/?def=a%20b%2Fc&t=tok",
                Play3dGameHostUrl.BuildForDefinition(ApiRoot, "a b/c", "tok"));
        }

        [Fact]
        public void BuildForDefinition_NoToken_OmitsT()
        {
            Assert.Equal(
                "https://maze.example.com/game/?def=g1",
                Play3dGameHostUrl.BuildForDefinition(ApiRoot, "g1", null));
        }

        [Fact]
        public void BuildForDifficulty_AppendsDifficultyAndToken()
        {
            Assert.Equal(
                "https://maze.example.com/game/?difficulty=easy&t=tok",
                Play3dGameHostUrl.BuildForDifficulty(ApiRoot, "easy", "tok"));
        }

        [Fact]
        public void BuildForMaze_AppendsIdTokenAndSettings()
        {
            Assert.Equal(
                "https://maze.example.com/game/?id=m1&t=tok&rows=10&cols=10",
                Play3dGameHostUrl.BuildForMaze(ApiRoot, "m1", "tok", "rows=10&cols=10"));
        }

        [Fact]
        public void BuildForMaze_NoSettings_OmitsExtraQuery()
        {
            Assert.Equal(
                "https://maze.example.com/game/?id=m1&t=tok",
                Play3dGameHostUrl.BuildForMaze(ApiRoot, "m1", "tok", null));
        }

        [Fact]
        public void BuildForToken_WithToken()
        {
            Assert.Equal("https://maze.example.com/game/?t=tok", Play3dGameHostUrl.BuildForToken(ApiRoot, "tok"));
        }

        [Fact]
        public void BuildForToken_NoToken_IsBareBase()
        {
            Assert.Equal("https://maze.example.com/game/", Play3dGameHostUrl.BuildForToken(ApiRoot, null));
        }
    }
}
