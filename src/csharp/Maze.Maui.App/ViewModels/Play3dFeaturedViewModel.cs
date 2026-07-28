using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// The Featured sub-page of the Play 3D browser — the admin-ordered catalogue of
    /// curated games and collections, as a single mixed, paged card list. Adds only
    /// the page fetch; paging, images and launch routing come from
    /// <see cref="Play3dListViewModel"/>.
    /// </summary>
    public sealed partial class Play3dFeaturedViewModel : Play3dListViewModel
    {
        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service</param>
        public Play3dFeaturedViewModel(IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService)
            : base(gameLibrary, navigationService, dialogService)
        {
            Title = "Featured";
        }

        /// <inheritdoc />
        public override string SearchPlaceholder => "Filter featured…";

        /// <inheritdoc />
        protected override async Task<Play3dCardPage> FetchPageAsync(int offset, int limit)
        {
            FeaturedGameItemsListResponse response = await GameLibrary.GetFeaturedGameItemsAsync(limit, offset);
            List<Play3dCardItem> cards = response.Items
                .Select(Play3dCardItem.FromFeatured)
                .OfType<Play3dCardItem>()
                .ToList();
            return new Play3dCardPage(cards, response.HasMore);
        }
    }
}
