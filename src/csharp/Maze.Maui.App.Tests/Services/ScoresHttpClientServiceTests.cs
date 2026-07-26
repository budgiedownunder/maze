using System.Text.Json;
using Maze.Maui.App.Models;
using Maze.Maui.App.Services;
using Xunit;

namespace Maze.Maui.App.Tests.Services
{
    /// <summary>
    /// Tests for the pure request-path helpers in <see cref="ScoreRequestPaths"/>
    /// plus the score DTO contract. The HTTP send path itself is not unit-tested
    /// here (as with the other client services — behaviour is covered via the
    /// service interface at the ViewModel layer); these tests pin the query-string
    /// assembly, the subject-exclusivity guard, the challenge convention, and the
    /// snake_case JSON mapping.
    /// </summary>
    public class ScoresHttpClientServiceTests
    {
        [Fact]
        public void BuildLeaderboardPath_MazeSubject_OnlySetsMazeId()
        {
            string path = ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: "abc", challenge: null, metric: null, direction: null, limit: null, offset: null, includeUsernames: null);

            Assert.Equal("scores?maze_id=abc", path);
        }

        [Fact]
        public void BuildLeaderboardPath_ChallengeSubject_OnlySetsChallenge()
        {
            string path = ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: null, challenge: "easy:42", metric: null, direction: null, limit: null, offset: null, includeUsernames: null);

            // ':' is percent-encoded by Uri.EscapeDataString.
            Assert.Equal("scores?challenge=easy%3A42", path);
        }

        [Fact]
        public void BuildLeaderboardPath_EncodesPathLikeMazeId()
        {
            // FileStore maze ids are Windows file paths — they must be encoded.
            string path = ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: @"C:\data\Maze_1.json", challenge: null, metric: null, direction: null, limit: null, offset: null, includeUsernames: null);

            Assert.Equal("scores?maze_id=C%3A%5Cdata%5CMaze_1.json", path);
        }

        [Fact]
        public void BuildLeaderboardPath_AppendsAllOptionalParams()
        {
            string path = ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: null, challenge: "hard:7",
                metric: ScoreMetric.Score, direction: SortDirection.Descending,
                limit: 20, offset: 40, includeUsernames: true);

            Assert.Equal("scores?challenge=hard%3A7&metric=score&direction=desc&limit=20&offset=40&include_usernames=true", path);
        }

        [Fact]
        public void BuildLeaderboardPath_TimeAscendingAndExcludeUsernames()
        {
            string path = ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: "m1", challenge: null,
                metric: ScoreMetric.Time, direction: SortDirection.Ascending,
                limit: null, offset: null, includeUsernames: false);

            Assert.Equal("scores?maze_id=m1&metric=time&direction=asc&include_usernames=false", path);
        }

        [Fact]
        public void BuildLeaderboardPath_ThrowsWhenNeitherSubjectSet()
        {
            Assert.Throws<ArgumentException>(() => ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: null, challenge: null, metric: null, direction: null, limit: null, offset: null, includeUsernames: null));
        }

        [Fact]
        public void BuildLeaderboardPath_ThrowsWhenBothSubjectsSet()
        {
            Assert.Throws<ArgumentException>(() => ScoreRequestPaths.BuildLeaderboardPath(
                mazeId: "m1", challenge: "easy:1", metric: null, direction: null, limit: null, offset: null, includeUsernames: null));
        }

        [Fact]
        public void BuildHistoryPath_NoParams_IsBare()
        {
            Assert.Equal("scores/me", ScoreRequestPaths.BuildHistoryPath(null, null));
        }

        [Fact]
        public void BuildHistoryPath_AppendsPaging()
        {
            Assert.Equal("scores/me?limit=20&offset=20", ScoreRequestPaths.BuildHistoryPath(20, 20));
        }

        [Fact]
        public void BuildHistoryPath_OffsetOnly()
        {
            Assert.Equal("scores/me?offset=5", ScoreRequestPaths.BuildHistoryPath(null, 5));
        }

        [Theory]
        [InlineData(ScoreMetric.Time, "time")]
        [InlineData(ScoreMetric.Score, "score")]
        public void ScoreMetric_ToQueryValue(ScoreMetric metric, string expected)
        {
            Assert.Equal(expected, metric.ToQueryValue());
        }

        [Theory]
        [InlineData(SortDirection.Ascending, "asc")]
        [InlineData(SortDirection.Descending, "desc")]
        public void SortDirection_ToQueryValue(SortDirection direction, string expected)
        {
            Assert.Equal(expected, direction.ToQueryValue());
        }

        [Fact]
        public void ForDefinition_SetsDefChallengeNotMazeId()
        {
            var subject = ScoreSubject.ForDefinition("g1");

            Assert.Null(subject.MazeId);
            Assert.Equal("def:g1", subject.Challenge);
        }

        [Fact]
        public void ForMaze_SetsMazeIdNotChallenge()
        {
            var subject = ScoreSubject.ForMaze("m1");

            Assert.Equal("m1", subject.MazeId);
            Assert.Null(subject.Challenge);
        }

        [Fact]
        public void ScoreboardResponse_DeserializesSnakeCaseJson()
        {
            const string json = """
                {
                  "scores": [
                    {"id":"r1","user_id":"u1","maze_id":null,"challenge":"easy:42","score":7,"elapsed_ms":12345,"recorded_at":"2026-06-13T12:00:00Z","username":"alice"},
                    {"id":"r2","user_id":"u2","maze_id":"m9","challenge":null,"score":3,"elapsed_ms":60000,"recorded_at":"2026-06-13T12:01:00Z"}
                  ],
                  "limit": 20,
                  "offset": 0,
                  "has_more": true
                }
                """;

            var board = JsonSerializer.Deserialize<ScoreboardResponse>(json);

            Assert.NotNull(board);
            Assert.Equal(20, board!.Limit);
            Assert.Equal(0, board.Offset);
            Assert.True(board.HasMore);
            Assert.Equal(2, board.Scores.Count);

            var first = board.Scores[0];
            Assert.Equal("r1", first.Id);
            Assert.Equal("u1", first.UserId);
            Assert.Null(first.MazeId);
            Assert.Equal("easy:42", first.Challenge);
            Assert.Equal(7ul, first.Score);
            Assert.Equal(12345, first.ElapsedMs);
            Assert.Equal("alice", first.Username);

            // The second row omits `username` entirely → null.
            Assert.Null(board.Scores[1].Username);
            Assert.Equal("m9", board.Scores[1].MazeId);
        }

        [Fact]
        public void BuildBoardDatesPath_PassesDefinitionIdAsSnakeCaseParam()
        {
            // The board-dates endpoint uses the snake_case `definition_id` param.
            Assert.Equal("scores/board-dates?definition_id=g1", ScoreRequestPaths.BuildBoardDatesPath("g1"));
        }

        [Fact]
        public void BuildBoardDatesPath_EncodesDefinitionId()
        {
            Assert.Equal("scores/board-dates?definition_id=a%2Fb", ScoreRequestPaths.BuildBoardDatesPath("a/b"));
        }

        [Fact]
        public void BuildCompletedPath_IsBarePostPath()
        {
            Assert.Equal("scores/me/completed", ScoreRequestPaths.BuildCompletedPath());
        }

        [Fact]
        public void BoardDatesResponse_DeserializesDates()
        {
            const string json = """{ "dates": ["2026-07-10", "2026-07-05"] }""";

            var result = JsonSerializer.Deserialize<BoardDatesResponse>(json)!;

            Assert.Equal(2, result.Dates.Count);
            Assert.Equal("2026-07-10", result.Dates[0]);
        }

        [Fact]
        public void CompletedChallenges_RoundTripsRequestAndResponse()
        {
            // Request serialises the caller's challenge keys under `challenges`.
            var request = new CompletedChallengesRequest { Challenges = new() { "def:a", "def:b" } };
            string requestJson = JsonSerializer.Serialize(request);
            Assert.Contains("\"challenges\":[\"def:a\",\"def:b\"]", requestJson);

            // Response reports the completed subset under `completed`.
            var response = JsonSerializer.Deserialize<CompletedChallengesResponse>("""{ "completed": ["def:a"] }""")!;
            Assert.Single(response.Completed);
            Assert.Equal("def:a", response.Completed[0]);
        }
    }
}
