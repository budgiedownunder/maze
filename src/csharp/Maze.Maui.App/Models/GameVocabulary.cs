namespace Maze.Maui.App.Models
{
    /// <summary>
    /// The lenient lowercase wire values for the game / collection string enums —
    /// named constants so callers avoid magic strings. The server accepts and
    /// emits exactly these; an unrecognised value degrades to the first (default /
    /// most restrictive) entry, so compare defensively rather than assuming the
    /// set is closed.
    /// </summary>
    public static class GameVocabulary
    {
        /// <summary>Access-tier values (default <see cref="Private"/>).</summary>
        public static class Visibility
        {
            /// <summary>Owner-only.</summary>
            public const string Private = "private";
            /// <summary>Explicit grantees.</summary>
            public const string Shared = "shared";
            /// <summary>Any signed-in user.</summary>
            public const string Public = "public";
            /// <summary>Admin-featured.</summary>
            public const string Curated = "curated";
        }

        /// <summary>Layout/board rotation values (default <see cref="Static"/>).</summary>
        public static class Rotation
        {
            /// <summary>One fixed layout and board.</summary>
            public const string Static = "static";
            /// <summary>A fresh layout and board each UTC day.</summary>
            public const string Daily = "daily";
        }

        /// <summary>Collection play-mode values (default <see cref="Arcade"/>).</summary>
        public static class PlayMode
        {
            /// <summary>Free choice of member game.</summary>
            public const string Arcade = "arcade";
            /// <summary>Ordered progression through the members.</summary>
            public const string Campaign = "campaign";
        }
    }
}
