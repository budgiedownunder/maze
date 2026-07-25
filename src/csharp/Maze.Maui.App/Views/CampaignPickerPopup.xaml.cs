using CommunityToolkit.Maui.Extensions;
using CommunityToolkit.Maui.Views;
using Maze.Maui.App.Models;

namespace Maze.Maui.App.Views
{
    /// <summary>
    /// The Campaign collection picker — the ordered member games as level cards
    /// (number · radio dot · name/description · state), with completed / current /
    /// locked state coloured green / primary / disabled. Locked levels are dimmed
    /// and non-selectable; the current level is preselected (or the first when the
    /// campaign is already complete, so any level can be replayed). Play returns the
    /// chosen <see cref="GameDefinition"/>; Cancel returns <c>null</c>.
    /// </summary>
    public partial class CampaignPickerPopup : Popup
    {
        /// <summary>Constructor</summary>
        /// <param name="collectionName">Collection name shown in the title</param>
        /// <param name="levels">The ordered levels with their states</param>
        public CampaignPickerPopup(string collectionName, IReadOnlyList<CampaignLevel> levels)
        {
            InitializeComponent();
            TitleLabel.Text = $"Play: {collectionName}";
            BindableLayout.SetItemsSource(GamesList, levels);

            // Preselect the current level, or the first when the campaign is already
            // complete (any level is replayable then).
            CampaignLevel? preselect = null;
            foreach (CampaignLevel level in levels)
            {
                if (level.State == CampaignLevelState.Current)
                {
                    preselect = level;
                    break;
                }
            }
            preselect ??= levels.Count > 0 ? levels[0] : null;
            if (preselect is not null)
                RadioButtonGroup.SetSelectedValue(GamesList, preselect.Definition);
        }

        // Selecting anywhere on a (non-locked) card checks its input-transparent radio.
        private void OnCardTapped(object sender, TappedEventArgs e)
        {
            if (sender is Element element && element.BindingContext is CampaignLevel level && level.IsSelectable)
                RadioButtonGroup.SetSelectedValue(GamesList, level.Definition);
        }

        private async void OnPlayClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<GameDefinition?>(RadioButtonGroup.GetSelectedValue(GamesList) as GameDefinition);

        private async void OnCancelClicked(object sender, EventArgs e)
            => await Navigation.ClosePopupAsync<GameDefinition?>(null);
    }
}
