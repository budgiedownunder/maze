using Maze.Maui.App.Models;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Resolves and plays today's daily challenge — the shared implementation
    /// behind the Home page's Today's Challenge tile and the flyout's Today's
    /// Challenge item (the C# analogue of the web client's shared
    /// <c>launchTodaysChallenge</c>). Finds the curated "Daily Challenges"
    /// collection in the featured catalogue and launches its daily member (the
    /// host page date-mixes the seed for the current UTC day); alerts when there is
    /// nothing to play, or when the lookup fails.
    /// </summary>
    public static class DailyChallengeLauncher
    {
        /// <summary>
        /// Resolves and launches today's daily challenge.
        /// </summary>
        /// <param name="gameLibrary">Game-library read service (featured catalogue + collection detail)</param>
        /// <param name="navigation">Navigation service (Play 3D launch)</param>
        /// <param name="dialog">Dialog service (nothing-to-play / error alerts)</param>
        /// <returns>Task</returns>
        public static async Task LaunchAsync(
            IGameLibraryService gameLibrary,
            INavigationService navigation,
            IDialogService dialog)
        {
            try
            {
                FeaturedGameItemsListResponse featured = await gameLibrary.GetFeaturedGameItemsAsync(100, 0);
                GameCollection? collection = DailyChallenge.FindCollection(featured.Items);
                GameDefinition? daily = null;
                if (collection is not null)
                {
                    GameCollectionDetailResponse detail = await gameLibrary.GetGameCollectionAsync(collection.Id);
                    daily = DailyChallenge.PickDaily(detail.Definitions);
                }

                if (daily is null)
                {
                    await dialog.ShowAlert("Daily Challenge", "There's no daily challenge to play right now.", "OK");
                    return;
                }

                // String route (not nameof) — kept free of Page types so the view
                // models that call this stay unit-testable (the test project links
                // them, but not the Page types).
                await navigation.GoToAsync("Play3dGamePage", new Dictionary<string, object>
                {
                    { "def", daily.Id },
                });
            }
            catch (Exception ex)
            {
                await dialog.ShowAlert("Daily Challenge", $"Couldn't load today's challenge\n\n{ex.Message}", "OK");
            }
        }
    }
}
