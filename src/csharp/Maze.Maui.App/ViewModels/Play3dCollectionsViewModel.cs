using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// A scope-filtered, paged list of game collections rendered as browse cards —
    /// the Collections tab of a <see cref="Play3dScopeBrowserViewModel"/>. Adds the
    /// list fetch (and its server <c>q</c>/<c>sort</c>) to <see cref="Play3dListViewModel"/>.
    /// </summary>
    public sealed partial class Play3dCollectionsViewModel : Play3dListViewModel
    {
        private readonly GameListScope _scope;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="scope">The ownership scope this list reads</param>
        public Play3dCollectionsViewModel(IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService, GameListScope scope)
            : base(gameLibrary, navigationService, dialogService)
        {
            _scope = scope;
        }

        /// <inheritdoc />
        public override string SearchPlaceholder => _scope == GameListScope.Public ? "Search collections…" : "Filter collections…";

        /// <inheritdoc />
        protected override bool UsesServerSearch => _scope == GameListScope.Public;

        /// <inheritdoc />
        protected override string EmptyMessage => _scope switch
        {
            GameListScope.Mine => "You haven't created any collections yet.",
            GameListScope.Shared => "No collections have been shared with you yet.",
            _ => "No collections have been published yet.",
        };

        /// <inheritdoc />
        protected override async Task<Play3dCardPage> FetchPageAsync(int offset, int limit)
        {
            GameCollectionListResponse response = await GameLibrary.ListGameCollectionsAsync(_scope, ServerQuery, SortOrder, limit, offset);
            List<Play3dCardItem> cards = response.Collections.Select(Play3dCardItem.FromCollection).ToList();
            return new Play3dCardPage(cards, response.HasMore);
        }
    }
}
