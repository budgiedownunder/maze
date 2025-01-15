namespace Maze.Maui.App.Services
{
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
        /// <returns>A task that contains the user's choice as a boolean value, where `true` indicates that the user chose to accept and `false` indicates that they chose to cancel</returns>
        public Task<bool> ShowConfirmation(string title, string message, string accept, string cancel);
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
    }
}
