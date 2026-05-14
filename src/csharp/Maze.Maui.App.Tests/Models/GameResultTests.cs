using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for <see cref="GameResult.FromJson"/> — the parser for the
    /// cross-language Bevy → JSON → C# game-result contract. Guards that the
    /// camelCase payload, the lowercase outcome enum, and the optional
    /// seed / extras fields all round-trip as expected.
    /// </summary>
    public class GameResultTests
    {
        [Fact]
        public void FromJson_ParsesWinPayloadWithAllFields()
        {
            const string json = """
                {"outcome":"win","elapsedMs":12345,"difficulty":"easy","rows":8,"cols":8,"seed":8080808}
                """;

            var result = GameResult.FromJson(json);

            Assert.NotNull(result);
            Assert.Equal(GameOutcome.Win, result!.Outcome);
            Assert.Equal(12345, result.ElapsedMs);
            Assert.Equal("easy", result.Difficulty);
            Assert.Equal(8u, result.Rows);
            Assert.Equal(8u, result.Cols);
            Assert.Equal(8080808ul, result.Seed);
        }

        [Fact]
        public void FromJson_ParsesLosePayload()
        {
            const string json = """
                {"outcome":"lose","elapsedMs":60000,"difficulty":"hard","rows":25,"cols":25,"seed":25252525}
                """;

            var result = GameResult.FromJson(json);

            Assert.NotNull(result);
            Assert.Equal(GameOutcome.Lose, result!.Outcome);
            Assert.Equal(60000, result.ElapsedMs);
            Assert.Equal("hard", result.Difficulty);
        }

        [Fact]
        public void FromJson_ParsesPayloadWithNullDifficultyAndOmittedSeed()
        {
            // The /game/?id=… (specific stored maze) path reports no difficulty
            // or seed — difficulty is null and seed is omitted entirely.
            const string json = """
                {"outcome":"win","elapsedMs":4200,"difficulty":null,"rows":7,"cols":7}
                """;

            var result = GameResult.FromJson(json);

            Assert.NotNull(result);
            Assert.Null(result!.Difficulty);
            Assert.Null(result.Seed);
            Assert.Equal(7u, result.Rows);
        }

        [Fact]
        public void FromJson_OutcomeMatchIsCaseInsensitive()
        {
            // Defensive: the contract is lowercase, but the parser must not be
            // brittle to casing.
            Assert.Equal(GameOutcome.Win, GameResult.FromJson("""{"outcome":"WIN","elapsedMs":1,"rows":3,"cols":3}""")!.Outcome);
            Assert.Equal(GameOutcome.Lose, GameResult.FromJson("""{"outcome":"Lose","elapsedMs":1,"rows":3,"cols":3}""")!.Outcome);
        }

        [Theory]
        [InlineData("")]
        [InlineData("not json")]
        [InlineData("{ \"outcome\": ")]
        public void FromJson_ReturnsNullForMalformedJson(string json)
        {
            Assert.Null(GameResult.FromJson(json));
        }
    }
}
