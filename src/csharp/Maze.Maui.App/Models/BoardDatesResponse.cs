using System.Text.Json.Serialization;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The UTC dates a daily game has a non-empty leaderboard for
    /// (<c>GET /scores/board-dates</c>) — the days someone has scored on that day's
    /// <c>def:&lt;id&gt;:&lt;date&gt;</c> board, most recent first. A static game (or an
    /// unplayed daily one) returns an empty list.
    /// </summary>
    public class BoardDatesResponse
    {
        /// <summary><c>yyyy-mm-dd</c> dates with at least one score, most recent first.</summary>
        [JsonPropertyName("dates")]
        public List<string> Dates { get; set; } = new();
    }
}
