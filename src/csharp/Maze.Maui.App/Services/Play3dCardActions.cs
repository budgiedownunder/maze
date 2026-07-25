using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Shared Play / Leaderboard behaviour for a Play 3D browse card, reused by every
    /// list surface (Featured + the scope browsers) so a game — or a single-member
    /// collection — launches straight into the host page, while a multi-member
    /// collection is guarded until its Arcade / Campaign picker exists. Depends only
    /// on the injected services, so the behaviour lives in one place.
    /// </summary>
    public static class Play3dCardActions
    {
        /// <summary>
        /// Plays a card: a game launches directly; a collection resolves its
        /// access-filtered members first (so a collection whose only member is
        /// inaccessible guards instead of 404ing), then launches the sole member,
        /// opens the Arcade free-choice picker for a multi-game Arcade collection, or
        /// guards an empty / Campaign collection.
        /// </summary>
        /// <param name="card">The card to play</param>
        /// <param name="navigationService">The navigation service</param>
        /// <param name="gameLibrary">The game-library read service</param>
        /// <param name="dialogService">The dialog service (picker + launch guards)</param>
        /// <returns>Task</returns>
        public static async Task PlayAsync(Play3dCardItem card, INavigationService navigationService, IGameLibraryService gameLibrary, IDialogService dialogService)
        {
            if (!card.IsCollection)
            {
                await Play3dLauncher.LaunchDefinitionAsync(navigationService, card.Id);
                return;
            }

            try
            {
                GameCollectionDetailResponse detail = await gameLibrary.GetGameCollectionAsync(card.Id);
                Play3dCollectionPlay play = Play3dCollectionLaunch.Resolve(detail.Definitions, detail.PlayMode);
                switch (play.Kind)
                {
                    case Play3dCollectionPlayKind.LaunchSingle:
                        await Play3dLauncher.LaunchDefinitionAsync(navigationService, play.DefinitionId!);
                        break;
                    case Play3dCollectionPlayKind.Arcade:
                        GameDefinition? chosen = await dialogService.ShowArcadePickerAsync(card.Name, detail.Definitions);
                        if (chosen is not null)
                            await Play3dLauncher.LaunchDefinitionAsync(navigationService, chosen.Id);
                        break;
                    case Play3dCollectionPlayKind.Campaign:
                        await dialogService.ShowAlert("Coming soon", "Campaign collections aren't playable yet.", "OK");
                        break;
                    default:
                        await dialogService.ShowAlert("Unavailable", "This collection has no games you can play.", "OK");
                        break;
                }
            }
            catch (Exception ex)
            {
                await dialogService.ShowAlert("Error", ex.Message, "OK");
            }
        }

        /// <summary>
        /// Opens the Leaderboards page for a game card (collections have no board).
        /// Preselecting this game's board is wired when the board selector learns
        /// stored-game subjects.
        /// </summary>
        /// <param name="card">The card whose board to open</param>
        /// <param name="navigationService">The navigation service</param>
        /// <returns>Task</returns>
        public static Task ShowLeaderboardAsync(Play3dCardItem card, INavigationService navigationService)
            => card.IsCollection ? Task.CompletedTask : navigationService.GoToAsync("LeaderboardsPage");
    }
}
