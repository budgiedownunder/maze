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
        /// Displays a prompt to the user as a popup window with the intent to capture a single string value, together with `accept` and `cancel` buttons
        /// </summary>
        /// <param name="message">Message</param>
        /// <returns>A task that completes when the popup is dismissed</returns>
        public async Task ShowGameResult(string message)
        {
            var popup = new Views.GameResultPopup(message);
            await Shell.Current.CurrentPage.ShowPopupAsync(popup);
        }

        /// <inheritdoc/>
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
