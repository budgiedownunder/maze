using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>How a collection's Play action should resolve, given its accessible members and play mode.</summary>
    public enum Play3dCollectionPlayKind
    {
        /// <summary>No member the caller can play — guard instead of launching.</summary>
        NoneAccessible,
        /// <summary>Exactly one accessible member — launch it directly.</summary>
        LaunchSingle,
        /// <summary>More than one member, Arcade mode — offer the free-choice picker.</summary>
        Arcade,
        /// <summary>More than one member, Campaign mode — offer the ordered progression.</summary>
        Campaign,
    }

    /// <summary>The resolved Play action for a collection: the kind plus, for <see cref="Play3dCollectionPlayKind.LaunchSingle"/>, the definition to launch.</summary>
    /// <param name="Kind">Which action applies</param>
    /// <param name="DefinitionId">The single member's id when <paramref name="Kind"/> is <see cref="Play3dCollectionPlayKind.LaunchSingle"/>; else <c>null</c></param>
    public readonly record struct Play3dCollectionPlay(Play3dCollectionPlayKind Kind, string? DefinitionId);

    /// <summary>
    /// Pure decision for playing a collection from the browser: no accessible member
    /// is guarded, a single one launches directly, and more than one opens the
    /// free-choice Arcade picker or the ordered Campaign progression per the
    /// collection's play mode. Kept free of navigation / dialog dependencies so the
    /// rule is unit-testable in isolation (the view model applies the result). The
    /// members must already be the collection detail's <b>access-filtered</b> list.
    /// </summary>
    public static class Play3dCollectionLaunch
    {
        /// <summary>Resolves the Play action from a collection's accessible members and play mode.</summary>
        /// <param name="accessibleDefinitions">The collection detail's access-filtered member definitions, in order</param>
        /// <param name="playMode">The collection's play mode (<c>arcade</c> / <c>campaign</c>); anything but campaign is treated as arcade</param>
        /// <returns>The action to take</returns>
        public static Play3dCollectionPlay Resolve(IReadOnlyList<GameDefinition> accessibleDefinitions, string playMode) => accessibleDefinitions.Count switch
        {
            0 => new Play3dCollectionPlay(Play3dCollectionPlayKind.NoneAccessible, null),
            1 => new Play3dCollectionPlay(Play3dCollectionPlayKind.LaunchSingle, accessibleDefinitions[0].Id),
            _ => playMode == GameVocabulary.PlayMode.Campaign
                ? new Play3dCollectionPlay(Play3dCollectionPlayKind.Campaign, null)
                : new Play3dCollectionPlay(Play3dCollectionPlayKind.Arcade, null),
        };
    }
}
