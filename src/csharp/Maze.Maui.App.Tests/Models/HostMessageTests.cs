using Maze.Maui.App.Models;
using Xunit;

namespace Maze.Maui.App.Tests.Models
{
    /// <summary>
    /// Tests for <see cref="HostMessage.KindOf"/> — the discriminator that routes
    /// the single WebView bridge channel to the result, failure, teardown or
    /// leaderboard path.
    /// The critical guarantees are that a failure is never mistaken for a result
    /// (a failure payload is valid JSON against <see cref="GameResult"/> and would
    /// otherwise deserialise into a bogus win), and that an untagged payload still
    /// reads as a result, which is what the host page sent before the envelope
    /// existed.
    /// </summary>
    public class HostMessageTests
    {
        [Fact]
        public void KindOf_TagsFailurePayloadAsFailure()
        {
            const string json = """
                {"kind":"failure","reason":"The game ran out of memory","detail":"RangeError","phase":"play"}
                """;

            Assert.Equal(HostMessageKind.Failure, HostMessage.KindOf(json));
        }

        [Fact]
        public void KindOf_TagsResultPayloadAsResult()
        {
            const string json = """
                {"kind":"result","outcome":"win","elapsedMs":12345,"score":7,"rows":8,"cols":8}
                """;

            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf(json));
        }

        [Fact]
        public void KindOf_TagsStoppedPayloadAsStopped()
        {
            // The teardown handshake: the game confirming it has released, so a
            // host can wait for that instead of guessing at a delay.
            Assert.Equal(HostMessageKind.Stopped, HostMessage.KindOf("""{"kind":"stopped"}"""));
            Assert.Equal(HostMessageKind.Stopped, HostMessage.KindOf("""{"kind":"Stopped"}"""));
        }

        [Fact]
        public void KindOf_TagsLeaderboardPayloadAsLeaderboard()
        {
            // The end-of-run overlay asking the host to open this run's board. It
            // carries no payload, so misrouting it to the result path would produce
            // an "unparseable payload" warning instead of a navigation.
            Assert.Equal(HostMessageKind.Leaderboard, HostMessage.KindOf("""{"kind":"leaderboard"}"""));
            Assert.Equal(HostMessageKind.Leaderboard, HostMessage.KindOf("""{"kind":"Leaderboard"}"""));
        }

        [Fact]
        public void KindOf_TreatsAnUnknownKindAsResult()
        {
            // A tag from a newer host page than this app understands still routes
            // to the result path rather than being dropped.
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("""{"kind":"something-new"}"""));
        }

        [Fact]
        public void KindOf_TreatsAnUntaggedPayloadAsResult()
        {
            // The envelope is newer than the result contract, so a payload from a
            // host page that predates it carries no `kind` and must still route to
            // the result path.
            const string json = """
                {"outcome":"lose","elapsedMs":60000,"score":0,"rows":8,"cols":8}
                """;

            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf(json));
        }

        [Fact]
        public void KindOf_TreatsMalformedJsonAsResult()
        {
            // Garbage keeps reaching GameResult.FromJson, which returns null and
            // produces the existing "unparseable payload" warning — the envelope
            // must not silently reclassify it as a failure.
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("{not json"));
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf(""));
        }

        [Fact]
        public void KindOf_IgnoresAKindThatIsNotAString()
        {
            // A non-string `kind` is a malformed envelope, not a failure.
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("""{"kind":3}"""));
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("""{"kind":null}"""));
        }

        [Fact]
        public void KindOf_MatchesTheFailureTagCaseInsensitively()
        {
            Assert.Equal(HostMessageKind.Failure, HostMessage.KindOf("""{"kind":"Failure"}"""));
        }

        [Fact]
        public void KindOf_TreatsANonObjectPayloadAsResult()
        {
            // Valid JSON that isn't an object has no envelope to read.
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("\"failure\""));
            Assert.Equal(HostMessageKind.Result, HostMessage.KindOf("[]"));
        }
    }
}
