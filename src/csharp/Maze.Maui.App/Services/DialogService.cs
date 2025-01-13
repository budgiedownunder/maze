using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Maze.Maui.App.Services
{
    class DialogService : IDialogService
    {
        public async Task ShowAlert(string title, string message, string cancel)
        {
            await Shell.Current.DisplayAlert(title, message, cancel);
        }

        public async Task<bool> ShowConfirmation(string title, string message, string accept, string cancel)
        {
            return await Shell.Current.DisplayAlert(title, message, accept, cancel);
        }

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
