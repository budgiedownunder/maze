using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Resolves today's daily challenge from the featured catalogue — the C#
    /// mirror of the web client's <c>launchTodaysChallenge</c>. The daily games
    /// live in a curated "Daily Challenges" collection (seeded at server startup),
    /// so the Today's Challenge entry point finds it by name and plays its daily
    /// member (the host page date-mixes the seed for the current UTC day). Pure, so
    /// the resolution is unit-testable without navigation or HTTP.
    /// </summary>
    public static class DailyChallenge
    {
        /// <summary>The curated collection the daily games live in.</summary>
        public const string CollectionName = "Daily Challenges";

        /// <summary>
        /// Finds the "Daily Challenges" collection in the featured catalogue.
        /// </summary>
        /// <param name="items">The featured catalogue items</param>
        /// <returns>The collection, or <c>null</c> when it isn't featured</returns>
        public static GameCollection? FindCollection(IReadOnlyList<FeaturedGameItem> items)
        {
            foreach (FeaturedGameItem item in items)
            {
                if (item.Collection is not null && item.Collection.Name == CollectionName)
                    return item.Collection;
            }
            return null;
        }

        /// <summary>
        /// Picks the member to play — the first <c>daily</c>-rotation game, else
        /// the first member (a defensive fallback).
        /// </summary>
        /// <param name="members">The collection's member definitions</param>
        /// <returns>The game to play, or <c>null</c> when the collection is empty</returns>
        public static GameDefinition? PickDaily(IReadOnlyList<GameDefinition> members)
        {
            foreach (GameDefinition member in members)
            {
                if (member.Rotation == GameVocabulary.Rotation.Daily)
                    return member;
            }
            return members.Count > 0 ? members[0] : null;
        }
    }
}
