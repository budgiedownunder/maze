using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Maze.Maui.App.Services
{
    public interface IDialogService
    {
        public Task ShowAlert(string title, string message, string cancel);

        public Task<bool> ShowConfirmation(string title, string message, string accept, string cancel);

        public Task<string> DisplayPrompt(string title, string message, string valueName, string accept = "OK", string cancel = "Cancel",
            string? placeholder = null, int maxlength = -1, Keyboard? keyboard = null, string? initialValue = "", bool allowEmpty = false, bool trimResult = true);
    }
}
