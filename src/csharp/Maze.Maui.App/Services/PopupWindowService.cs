using CommunityToolkit.Maui.Extensions;

namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Represents a popup window dialog service
    /// </summary>
    public class PopupWindowService : IDialogService
    {
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
        /// Displays the Play 3D difficulty picker (Easy / Tricky / Hard) as a popup window
        /// </summary>
        /// <returns>A task that contains the chosen <see cref="Models.Difficulty"/>, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.Difficulty?> ShowPlay3dDifficultyAsync()
        {
            var popup = new Views.Play3dDifficultyPopup();
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.Difficulty?>(popup);
            return result.Result;
        }

        /// <summary>
        /// Displays the Play 3D custom-launch picker (sky / wall texture /
        /// landmark toggles / time limit) for a user-edited maze as a popup
        /// window. Pre-fills from
        /// <see cref="Services.Play3dCustomLaunchSettingsStore.Load"/>; on
        /// Play, the popup persists the chosen settings before returning
        /// them.
        /// </summary>
        /// <param name="mazeName">Maze name shown in the popup title</param>
        /// <returns>A task that contains the chosen settings, or <c>null</c> if the user cancelled</returns>
        public async Task<Models.Play3dCustomLaunchSettings?> ShowPlay3dCustomLaunchAsync(string? mazeName = null)
        {
            var popup = new Views.Play3dCustomLaunchPopup(mazeName);
            var result = await Shell.Current.CurrentPage.ShowPopupAsync<Models.Play3dCustomLaunchSettings?>(popup);
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
