using System.Globalization;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A single leaderboard row, shaped for display: rank, player, the formatted
    /// run time and score, the completion timestamp, and whether this is the
    /// caller's row on a multi-player board (drives the highlight). Observable so
    /// the player's avatar can be loaded asynchronously and swapped in after the
    /// row is rendered.
    /// </summary>
    public partial class LeaderboardRow : ObservableObject
    {
        /// <summary>1-based position within the loaded board.</summary>
        public int Rank { get; }

        /// <summary>The player's username, or <c>—</c> when unresolved.</summary>
        public string Player { get; }

        /// <summary>The player's user id — drives the avatar fetch.</summary>
        public string UserId { get; }

        /// <summary>The player's avatar cache-buster, or <c>null</c> when they
        /// have no avatar (resolved on the same board JOIN as the username).</summary>
        public string? AvatarUpdatedAt { get; }

        /// <summary>The player's avatar PNG bytes, set asynchronously after the
        /// row is created; <c>null</c> until loaded or when the player has no
        /// avatar (the avatar control then shows the generic placeholder). Bytes
        /// (not a UI image type) keep this model free of MAUI runtime types.</summary>
        [ObservableProperty]
        private byte[]? avatarBytes;

        /// <summary>Run time formatted as <c>m:ss.mmm</c>.</summary>
        public string Time { get; }

        /// <summary>Final score.</summary>
        public string Score { get; }

        /// <summary>Local completion timestamp.</summary>
        public string Completed { get; }

        /// <summary>Whether this row is the caller's on a multi-player board.</summary>
        public bool IsHighlighted { get; }

        /// <summary>Whether the Player column is shown for this board (Play-3D
        /// only). Drives both the cell visibility and the column's width collapse.</summary>
        public bool ShowPlayer { get; }

        /// <summary>
        /// Builds a display row from a recorded entry.
        /// </summary>
        /// <param name="rank">1-based board position</param>
        /// <param name="entry">The recorded run</param>
        /// <param name="isHighlighted">Whether to highlight this as the caller's row</param>
        /// <param name="showPlayer">Whether the board shows the Player column</param>
        public LeaderboardRow(int rank, ScoreEntry entry, bool isHighlighted, bool showPlayer)
        {
            Rank = rank;
            Player = string.IsNullOrEmpty(entry.Username) ? "—" : entry.Username!;
            Time = FormatElapsed(entry.ElapsedMs);
            Score = entry.Score.ToString(CultureInfo.CurrentCulture);
            Completed = entry.RecordedAt.ToLocalTime().ToString("g", CultureInfo.CurrentCulture);
            IsHighlighted = isHighlighted;
            ShowPlayer = showPlayer;
            UserId = entry.UserId;
            AvatarUpdatedAt = entry.AvatarUpdatedAt;
        }

        /// <summary>
        /// Formats an elapsed-run duration as <c>m:ss.mmm</c> (e.g. 42137 →
        /// <c>0:42.137</c>), matching the web leaderboard and Bevy win-overlay.
        /// </summary>
        /// <param name="ms">Elapsed time in milliseconds</param>
        /// <returns>The formatted duration</returns>
        public static string FormatElapsed(long ms)
        {
            long total = Math.Max(0, ms);
            long minutes = total / 60000;
            long seconds = (total % 60000) / 1000;
            long millis = total % 1000;
            return $"{minutes}:{seconds:D2}.{millis:D3}";
        }
    }
}
