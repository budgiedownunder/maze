using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The Arcade collection picker — a radio list of a collection's accessible
    /// member games (each a bordered card with a native radio dot, name + description),
    /// defaulting to the first. Play returns the chosen <see cref="GameDefinition"/>;
    /// Cancel returns <c>null</c>. The caller launches the chosen game via
    /// <c>/game/?def=&lt;id&gt;</c>.
    /// </summary>
    public partial class ArcadePickerPopup : Popup
    {
        /// <summary>Constructor</summary>
        /// <param name="collectionName">Collection name shown in the title</param>
        /// <param name="definitions">The accessible member games, in order</param>
        public ArcadePickerPopup(string collectionName, IReadOnlyList<GameDefinition> definitions)
        {
            InitializeComponent();
            TitleLabel.Text = $"Play: {collectionName}";
            BindableLayout.SetItemsSource(GamesList, definitions);
            if (definitions.Count > 0)
                RadioButtonGroup.SetSelectedValue(GamesList, definitions[0]);
        }

        // Selecting anywhere on a card checks its (input-transparent) radio.
        private void OnCardTapped(object sender, TappedEventArgs e)
        {
            if (sender is Element element && element.BindingContext is GameDefinition definition)
                RadioButtonGroup.SetSelectedValue(GamesList, definition);
        }

        private async void OnPlayClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<GameDefinition?>(RadioButtonGroup.GetSelectedValue(GamesList) as GameDefinition);

        private async void OnCancelClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<GameDefinition?>(null);
    }
}
