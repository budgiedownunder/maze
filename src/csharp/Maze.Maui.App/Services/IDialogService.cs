namespace Maze.Maui.App.Services
{
    /// <summary>
    /// The action chosen from the 2D game pause menu.
    /// </summary>
    public enum PauseMenuResult
    {
        /// <summary>Resume the current game (default — also used if the popup is dismissed).</summary>
        Resume = 0,
        /// <summary>Restart the current maze from the beginning.</summary>
        Restart = 1,
    }

    /// <summary>
    /// The action chosen from the Play 3D launch chooser.
    /// </summary>
    public enum Play3dLaunchChoice
    {
        /// <summary>Cancel the launch (default — also used if the popup is dismissed).</summary>
        Cancel = 0,
        /// <summary>Launch with the maze's saved game settings.</summary>
        Run = 1,
        /// <summary>Open the settings popup for a one-off (non-persisted) launch.</summary>
        CustomRun = 2,
    }

    /// <summary>
    /// Represents a dialog service interface
    /// </summary>
    public interface IDialogService
    {
        /// <summary>
        /// Displays an alert message to the user with a single `cancel` choice
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <returns>A task that contains the alert</returns>
        public Task ShowAlert(string title, string message, string cancel);
        /// <summary>
        /// Displays a confirmation message to the user with `accept` and `cancel` choices
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="accept">Text to display for `accept`</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <param name="isDestructive">If true, styles the accept button to indicate a destructive action</param>
        /// <returns>A task that contains the user's choice as a boolean value, where `true` indicates that the user chose to accept and `false` indicates that they chose to cancel</returns>
        public Task<bool> ShowConfirmation(string title, string message, string accept, string cancel, bool isDestructive = false);
        /// <summary>
        /// Displays a confirmation message to the user with `accept`, `cancel`, and `dismiss` choices
        /// </summary>
        /// <param name="title">Title</param>
        /// <param name="message">Message</param>
        /// <param name="accept">Text to display for `accept`</param>
        /// <param name="cancel">Text to display for `cancel`</param>
        /// <param name="dismiss">Text to display for `dismiss`</param>
        /// <returns>A task that contains the user's choice: <c>true</c> = accept, <c>false</c> = cancel, <c>null</c> = dismiss</returns>
        public Task<bool?> ShowConfirmation(string title, string message, string accept, string cancel, string dismiss);
        /// <summary>
        /// Displays a prompt to the user with the intent to capture a single string value, together with `accept` and `cancel` choices
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
        public Task<string> DisplayPrompt(string title, string message, string valueName, string accept = "OK", string cancel = "Cancel",
            string? placeholder = null, int maxlength = -1, Keyboard? keyboard = null, string? initialValue = "", bool allowEmpty = false, bool trimResult = true);
        /// <summary>
        /// Displays the game result popup with the given message. A win shows the
        /// celebration sprite; a loss shows the game-over (skull) image.
        /// </summary>
        /// <param name="message">Result message to display</param>
        /// <param name="won">Whether the game was won (celebration) or lost (game-over image)</param>
        /// <returns>A task that resolves to <c>true</c> when the player chose Play Again, otherwise <c>false</c></returns>
        public Task<bool> ShowGameResult(string message, bool won);
        /// <summary>
        /// Displays the 2D game pause menu (Resume / Restart).
        /// </summary>
        /// <returns>A task resolving to the chosen <see cref="PauseMenuResult"/> (<see cref="PauseMenuResult.Resume"/> if dismissed)</returns>
        public Task<PauseMenuResult> ShowPauseMenu();
        /// <summary>
        /// Displays the Arcade collection picker — a radio list of the collection's
        /// accessible member games, defaulting to the first — so the user chooses
        /// one to play.
        /// </summary>
        /// <param name="collectionName">Collection name shown in the popup title</param>
        /// <param name="definitions">The accessible member games, in order</param>
        /// <returns>A task containing the chosen game, or <c>null</c> if the user cancelled</returns>
        public Task<Models.GameDefinition?> ShowArcadePickerAsync(string collectionName, IReadOnlyList<Models.GameDefinition> definitions);

        /// <summary>
        /// Displays the Play 3D launch chooser (Run / Custom Run… / Cancel) for a
        /// user-edited maze.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <returns>A task containing the chosen <see cref="Play3dLaunchChoice"/> (<see cref="Play3dLaunchChoice.Cancel"/> if dismissed)</returns>
        public Task<Play3dLaunchChoice> ShowPlay3dLaunchChooserAsync(string? mazeName = null);

        /// <summary>
        /// Displays the Play 3D settings popup (sky / wall texture / landmark
        /// toggles / time limit) for a one-off custom launch of a user-edited maze,
        /// seeded from <paramref name="current"/> (or defaults when none). The
        /// returned settings drive this launch only; nothing is persisted.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <param name="current">Settings to seed the popup with, or null for defaults</param>
        /// <returns>A task containing the chosen settings, or <c>null</c> if the user cancelled</returns>
        public Task<Models.MazeGameSettings?> ShowMazeGameSettingsAsync(string? mazeName = null, Models.MazeGameSettings? current = null);

        /// <summary>
        /// Displays the per-maze game-settings editor for a user-edited maze,
        /// seeded from the maze's current settings (or defaults when none).
        /// On Apply the returned settings are the caller's to persist with the
        /// maze; nothing is written to the device store.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <param name="current">The maze's current settings, or null for defaults</param>
        /// <returns>A task containing the edited settings, or <c>null</c> if the user cancelled</returns>
        public Task<Models.MazeGameSettings?> ShowMazeGameSettingsEditorAsync(string? mazeName, Models.MazeGameSettings? current);
    }
}
