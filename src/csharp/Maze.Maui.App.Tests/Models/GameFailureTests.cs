using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for <see cref="GameFailure.FromJson"/> — the parser for the failure
    /// half of the hosted <c>/game/</c> page's message contract. A run that died
    /// mid-frame may not be able to describe itself fully, so every field beyond
    /// the reason is optional and a partial payload must still parse.
    /// </summary>
    public class GameFailureTests
    {
        [Fact]
        public void FromJson_ParsesAFullPayload()
        {
            const string json = """
                {"kind":"failure","reason":"The game ran out of memory","detail":"RangeError: Out of memory","phase":"play"}
                """;

            var failure = GameFailure.FromJson(json);

            Assert.NotNull(failure);
            Assert.Equal("The game ran out of memory", failure!.Reason);
            Assert.Equal("RangeError: Out of memory", failure.Detail);
            Assert.Equal("play", failure.Phase);
        }

        [Fact]
        public void FromJson_ParsesAPayloadCarryingOnlyAReason()
        {
            var failure = GameFailure.FromJson("""{"kind":"failure","reason":"The game stopped unexpectedly"}""");

            Assert.NotNull(failure);
            Assert.Equal("The game stopped unexpectedly", failure!.Reason);
            Assert.Null(failure.Detail);
            Assert.Null(failure.Phase);
        }

        [Fact]
        public void FromJson_DefaultsTheReasonToEmptyWhenAbsent()
        {
            // An unclassifiable failure still reports; the page decides what to
            // show when there is no reason text.
            var failure = GameFailure.FromJson("""{"kind":"failure"}""");

            Assert.NotNull(failure);
            Assert.Equal("", failure!.Reason);
        }

        [Fact]
        public void FromJson_ReturnsNullOnMalformedJson()
        {
            // Matches GameResult.FromJson — the bridge path logs and ignores
            // rather than throwing out of an event handler.
            Assert.Null(GameFailure.FromJson("{not json"));
        }

        [Fact]
        public void Unreadable_KeepsTheRawPayloadAsDetail()
        {
            // The bridge falls back to this when a failure payload won't parse:
            // a crash that can't be read is still a crash and must not vanish,
            // so the player gets the generic reason and the log gets the raw text.
            var failure = GameFailure.Unreadable("{not json");

            Assert.Equal(GameFailure.GenericReason, failure.Reason);
            Assert.Equal("{not json", failure.Detail);
            Assert.Equal("unknown", failure.Phase);
        }
    }
}
