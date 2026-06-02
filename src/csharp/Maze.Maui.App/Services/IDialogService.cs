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
        /// Displays the Play 3D difficulty picker (Easy / Tricky / Hard).
        /// </summary>
        /// <returns>A task containing the chosen <see cref="Models.Difficulty"/>, or <c>null</c> if the user cancelled</returns>
        public Task<Models.Difficulty?> ShowPlay3dDifficultyAsync();

        /// <summary>
        /// Displays the Play 3D custom-launch picker (sky / wall texture /
        /// landmark toggles / time limit) for a user-edited maze.
        /// Pre-fills from the user's previously-saved settings (MAUI
        /// <see cref="Microsoft.Maui.Storage.Preferences"/>); on Play, the
        /// returned <see cref="Models.Play3dCustomLaunchSettings"/> is also
        /// persisted by the popup itself for next time.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <returns>A task containing the chosen settings, or <c>null</c> if the user cancelled</returns>
        public Task<Models.Play3dCustomLaunchSettings?> ShowPlay3dCustomLaunchAsync(string? mazeName = null);
    }
}
