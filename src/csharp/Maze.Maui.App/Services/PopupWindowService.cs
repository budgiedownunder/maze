using CommunityToolkit.Maui.Extensions;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a popup window dialog service
    /// </summary>
    public class PopupWindowService : IDialogService
    {
        private readonly IScoresService _scoresService;
        private readonly IGameLibraryService _gameLibrary;

        /// <summary>Constructor</summary>
        /// <param name="scoresService">Injected scores service (campaign progress lookup)</param>
        /// <param name="gameLibrary">Injected game-library service (leaderboard game picker)</param>
        public PopupWindowService(IScoresService scoresService, IGameLibraryService gameLibrary)
        {
            _scoresService = scoresService;
            _gameLibrary = gameLibrary;
        }

        /// <summary>
        /// Displays a alert message to the user as a popup window with a single `cancel` button
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <returns>A task that contains the alert</returns>
        public async Task ShowAlert(string title, string message, string cancel)
        {
            await Shell.Current.DisplayAlertAsync(title, message, cancel);
        }
        /// <summary>
        /// Displays a confirmation message to the user as a popup window with `accept` and `cancel` buttons
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="accept">Text to display for `accept`</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <param name="isDestructive">If true, styles the accept button to indicate a destructive action</param>
        /// <returns>A task that contains the user's choice as a boolean value, where `true` indicates that the user chose to accept and `false` indicates that they chose to cancel</returns>
        public async Task<bool> ShowConfirmation(string title, string message, string accept, string cancel, bool isDestructive = false)
        {
            var popup = new Views.ConfirmationPopup(title, message, accept, cancel, isDestructive: isDestructive);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<bool?>(popup);
            return result.Result == true;
        }
        /// <summary>
        /// Displays a confirmation message to the user as a popup window with `accept`, `cancel`, and `dismiss` buttons
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="accept">Text to display for `accept`</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <param name="dismiss">Text to display for `dismiss`</param>
        /// <returns>A task that contains the user's choice: <c>true</c> = accept, <c>false</c> = cancel, <c>null</c> = dismiss</returns>
        public async Task<bool?> ShowConfirmation(string title, string message, string accept, string cancel, string dismiss)
        {
            var popup = new Views.ConfirmationPopup(title, message, accept, cancel, dismiss);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<bool?>(popup);
            return result.Result;
        }
        /// <summary>
        /// Displays the game result (win or loss) to the user as a popup window.
        /// </summary>
        /// <param name="message">Result message</param>
        /// <param name="won">Whether the player won</param>
        /// <returns>A task that completes when the popup is dismissed</returns>
        public async Task<bool> ShowGameResult(string message, bool won)
        {
            var popup = new Views.GameResultPopup(message, won);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<bool>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the 2D game pause menu (Resume / Restart) as a popup window.
        /// </summary>
        /// <returns>The chosen <see cref="PauseMenuResult"/> (<see cref="PauseMenuResult.Resume"/> if dismissed)</returns>
        public async Task<PauseMenuResult> ShowPauseMenu()
        {
            var popup = new Views.PausePopup();
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<PauseMenuResult>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Arcade collection picker (radio list of member games) as a popup window.
        /// </summary>
        /// <param name="collectionName">Collection name shown in the popup title</param>
        /// <param name="definitions">The accessible member games, in order</param>
        /// <returns>The chosen game, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.GameDefinition?> ShowArcadePickerAsync(string collectionName, IReadOnlyList<Models.GameDefinition> definitions)
        {
            var popup = new Views.ArcadePickerPopup(collectionName, definitions);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.GameDefinition?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Campaign collection picker as a popup window, resolving each
        /// level's completed / current / locked state from the caller's scores first.
        /// </summary>
        /// <param name="collectionName">Collection name shown in the popup title</param>
        /// <param name="definitions">The accessible member games, in campaign order</param>
        /// <returns>The chosen game, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.GameDefinition?> ShowCampaignPickerAsync(string collectionName, IReadOnlyList<Models.GameDefinition> definitions)
        {
            List<string> keys = definitions.Select(Models.CampaignLevel.ChallengeKey).ToList();
            Models.CompletedChallengesResponse completed = await _scoresService.GetCompletedChallengesAsync(keys);
            IReadOnlyList<Models.CampaignLevel> levels = Models.CampaignLevel.Build(definitions, completed.Completed);

            var popup = new Views.CampaignPickerPopup(collectionName, levels);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.GameDefinition?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Leaderboards game picker as a popup window.
        /// </summary>
        /// <returns>The chosen game, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.GameDefinition?> ShowGamePickerAsync()
        {
            var popup = new Views.LeaderboardGamePickerPopup(_gameLibrary);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.GameDefinition?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Play 3D launch chooser (Run / Custom Run… / Cancel) as a popup window.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <returns>A task that contains the chosen <see cref="Play3dLaunchChoice"/> (<see cref="Play3dLaunchChoice.Cancel"/> if dismissed)</returns>
        public async Task<Play3dLaunchChoice> ShowPlay3dLaunchChooserAsync(string? mazeName = null)
        {
            var popup = new Views.Play3dLaunchChooserPopup(mazeName);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Play3dLaunchChoice>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Play 3D settings popup (sky / wall texture / landmark
        /// toggles / time limit) for a one-off custom launch as a popup window,
        /// seeded from <paramref name="current"/> (or defaults when none). The
        /// returned settings drive this launch only; nothing is persisted.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <param name="current">Settings to seed the popup with, or null for defaults</param>
        /// <returns>A task that contains the chosen settings, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.MazeGameSettings?> ShowMazeGameSettingsAsync(string? mazeName = null, Models.MazeGameSettings? current = null)
        {
            var popup = new Views.MazeGameSettingsPopup(mazeName, current, submitButtonText: "Play");
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.MazeGameSettings?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the per-maze game-settings editor, seeded from the maze's
        /// current settings. On Apply the returned settings are the caller's to
        /// persist with the maze; the popup writes nothing to the device store.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <param name="current">The maze's current settings, or null for defaults</param>
        /// <returns>A task that contains the edited settings, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.MazeGameSettings?> ShowMazeGameSettingsEditorAsync(string? mazeName, Models.MazeGameSettings? current)
        {
            string title = !string.IsNullOrWhiteSpace(mazeName) ? $"Game settings — {mazeName}" : "Game settings";
            var popup = new Views.MazeGameSettingsPopup(mazeName, current, title: title, submitButtonText: "Apply");
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.MazeGameSettings?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays a prompt to the user as a popup window with the intent to capture a single string value, together with `accept` and `cancel` buttons
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="valueName">Value name</param>
        /// <param name="accept">Text to display for `accept`</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <param name="placeholder">Placeholder text displayed if no value is entered</param>
        /// <param name="maxlength">Maximum text length</param>
        /// <param name="keyboard">Keyboard to use</param>
        /// <param name="initialValue">Intial value to offer</param>
        /// <param name="allowEmpty">Allow an empty value?</param>
        /// <param name="trimResult">Trim the result of any leading/trailing blanks?</param>
        /// <returns>A task that contains the user's choice as a string value which will be `null` if they chose to cancel</returns>
        public async Task<string> DisplayPrompt(string title, string message, string valueName, string accept = "OK", string cancel = "Cancel",
            string? placeholder = null, int maxlength = -1, Keyboard? keyboard = null, string? initialValue = "", bool allowEmpty = false, bool trimResult = true)
        {
            string? result = null;
            bool finished = false;

            while (!finished)
            {
                result = await Shell.Current.DisplayPromptAsync(title, message, accept, cancel, placeholder, maxlength, keyboard, initialValue);

                if (result is not null)
                {
                    initialValue = result;

                    if (trimResult)
                        result = result.Trim();

                    if (allowEmpty || result.Length > 0)
                        finished = true;
                    else
                        await ShowAlert(title, $"{valueName} cannot be empty or blank", "OK");
                }
                else
                    finished = true;
            }
            return result!;
        }
    }
}
