using Maze.Maui.App.Models;
using Maze.Maui.App.Services;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// A scope-filtered, paged list of game definitions rendered as browse cards —
    /// the Games tab of a <see cref="Play3dScopeBrowserViewModel"/>. Adds the list
    /// fetch (and its server <c>q</c>/<c>sort</c>) to <see cref="Play3dListViewModel"/>.
    /// </summary>
    public sealed partial class Play3dDefinitionsViewModel : Play3dListViewModel
    {
        private readonly GameListScope _scope;

        /// <summary>Constructor</summary>
        /// <param name="gameLibrary">Injected game-library read service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="scope">The ownership scope this list reads</param>
        public Play3dDefinitionsViewModel(IGameLibraryService gameLibrary, INavigationService navigationService, IDialogService dialogService, GameListScope scope)
            : base(gameLibrary, navigationService, dialogService)
        {
            _scope = scope;
        }

        /// <inheritdoc />
        public override string SearchPlaceholder => _scope == GameListScope.Public ? "Search games…" : "Filter games…";

        /// <inheritdoc />
        protected override bool UsesServerSearch => _scope == GameListScope.Public;

        /// <inheritdoc />
        protected override string EmptyMessage => _scope switch
        {
            GameListScope.Mine => "You haven't created any 3D games yet.",
            GameListScope.Shared => "No games have been shared with you yet.",
            _ => "No games have been published yet.",
        };

        /// <inheritdoc />
        protected override async Task<Play3dCardPage> FetchPageAsync(int offset, int limit)
        {
            GameDefinitionListResponse response = await GameLibrary.ListGameDefinitionsAsync(_scope, ServerQuery, SortOrder, limit, offset);
            List<Play3dCardItem> cards = response.Definitions.Select(Play3dCardItem.FromDefinition).ToList();
            return new Play3dCardPage(cards, response.HasMore);
        }
    }
}
