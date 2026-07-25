namespace Maze.Maui.App.Models
{
    /// <summary>A campaign level's progression state.</summary>
    public enum CampaignLevelState
    {
        /// <summary>The caller has a score on this game's board (globally) — replayable.</summary>
        Completed,
        /// <summary>The first game with no score — the level to play next.</summary>
        Current,
        /// <summary>No score and after the current level — not yet playable.</summary>
        Locked,
    }

    /// <summary>
    /// One ordered level of a Campaign collection: its 1-based number, the member
    /// game, and its <see cref="CampaignLevelState"/>. Completion is <b>global</b> —
    /// a game scored on in any collection shows completed here too — so a completed
    /// level can sit after the current one; the current level is the first with no
    /// score, and everything after it with no score is locked.
    /// </summary>
    public sealed class CampaignLevel
    {
        /// <summary>1-based level number.</summary>
        public int Number { get; init; }

        /// <summary>The member game this level plays.</summary>
        public GameDefinition Definition { get; init; } = new();

        /// <summary>The level's progression state.</summary>
        public CampaignLevelState State { get; init; }

        /// <summary>The game's display name.</summary>
        public string Name => Definition.Name;

        /// <summary>The game's optional description.</summary>
        public string? Description => Definition.Description;

        /// <summary>True when the level is locked (not selectable).</summary>
        public bool IsLocked => State == CampaignLevelState.Locked;

        /// <summary>True when the level can be selected / played.</summary>
        public bool IsSelectable => !IsLocked;

        /// <summary>The status label shown on the row.</summary>
        public string StatusText => State switch
        {
            CampaignLevelState.Completed => "✓ Completed",
            CampaignLevelState.Current => "Play",
            _ => "Locked",
        };

        /// <summary>
        /// The leaderboard challenge key for a campaign member — <c>def:&lt;id&gt;</c>.
        /// Campaign members are treated as static; daily-in-campaign is deferred.
        /// </summary>
        /// <param name="definition">The member game</param>
        /// <returns>The <c>def:&lt;id&gt;</c> challenge key</returns>
        public static string ChallengeKey(GameDefinition definition) => $"def:{definition.Id}";

        /// <summary>
        /// Builds the ordered levels with state from the members and the caller's
        /// completed challenge keys (from <c>/scores/me/completed</c>).
        /// </summary>
        /// <param name="members">The ordered, access-filtered member games</param>
        /// <param name="completedChallenges">The challenge keys the caller has scored on</param>
        /// <returns>The ordered levels with their states</returns>
        public static IReadOnlyList<CampaignLevel> Build(IReadOnlyList<GameDefinition> members, IReadOnlyCollection<string> completedChallenges)
        {
            var completedSet = new HashSet<string>(completedChallenges);
            bool[] done = members.Select(m => completedSet.Contains(ChallengeKey(m))).ToArray();
            int current = Array.FindIndex(done, scored => !scored); // first unscored; -1 when all done

            var levels = new List<CampaignLevel>(members.Count);
            for (int i = 0; i < members.Count; i++)
            {
                CampaignLevelState state = done[i]
                    ? CampaignLevelState.Completed
                    : i == current ? CampaignLevelState.Current : CampaignLevelState.Locked;
                levels.Add(new CampaignLevel { Number = i + 1, Definition = members[i], State = state });
            }

            return levels;
        }
    }
}
