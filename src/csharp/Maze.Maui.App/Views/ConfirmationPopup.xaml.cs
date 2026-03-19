using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// A popup that displays a confirmation dialog with accept and cancel buttons, and an optional
    /// dismiss button. Returns <c>true</c> for accept, <c>false</c> for cancel, and <c>null</c> for dismiss.
    /// </summary>
    public partial class ConfirmationPopup : Popup
    {
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="title">Dialog title</param>
        /// <param name="message">Dialog message</param>
        /// <param name="accept">Text for the accept button</param>
        /// <param name="cancel">Text for the cancel button</param>
        /// <param name="dismiss">Text for the optional dismiss button; omit for a 2-button dialog</param>
        /// <param name="isDestructive">If true, styles the accept button red to indicate a destructive action</param>
        public ConfirmationPopup(string title, string message, string accept, string cancel, string? dismiss = null, bool isDestructive = false)
        {
            InitializeComponent();
            TitleLabel.Text = title;
            MessageLabel.Text = message;
            AcceptButton.Text = accept;
            CancelButton.Text = cancel;

            if (dismiss is null)
            {
                DismissButton.IsVisible = false;
                ButtonRow.ColumnDefinitions = new ColumnDefinitionCollection
                {
                    new ColumnDefinition(GridLength.Star),
                    new ColumnDefinition(GridLength.Star)
                };
                Grid.SetColumn(CancelButton, 0);
                Grid.SetColumn(AcceptButton, 1);

                CancelButton.BackgroundColor = (Color)Application.Current!.Resources["Gray200"];
                CancelButton.TextColor = (Color)Application.Current!.Resources["Gray950"];

                if (isDestructive)
                {
                    AcceptButton.BackgroundColor = Colors.Red;
                    AcceptButton.TextColor = Colors.White;
                }
            }
            else
            {
                DismissButton.Text = dismiss;
            }
        }

        /// <summary>
        /// Handles the Accept button click.
        /// </summary>
        private async void OnAcceptClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<bool?>(true);
        }

        /// <summary>
        /// Handles the Cancel button click.
        /// </summary>
        private async void OnCancelClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<bool?>(false);
        }

        /// <summary>
        /// Handles the Dismiss button click.
        /// </summary>
        private async void OnDismissClicked(object sender, EventArgs e)
        {
            await Navigation.ClosePopupAsync<bool?>(null);
        }
    }
}
