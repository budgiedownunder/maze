using CommunityToolkit.Mvvm.ComponentModel;

namespace Maze.Maui.App.Models
{
    /// <summary>
    /// A single card in the Play 3D browser — a stored game or a collection,
    /// projected to just what the card shows (image · name · description) plus the
    /// bits the Play / Leaderboard actions need. The image bytes arrive
    /// asynchronously (fetched per id by the view model), hence the observable
    /// <see cref="ImageBytes"/>; every other field is set once at map time.
    /// </summary>
    public partial class Play3dCardItem : ObservableObject
    {
        /// <summary>Whether this card is a game or a collection.</summary>
        public GameEntityKind Kind { get; init; }

        /// <summary>The definition or collection id (drives Play / image fetch).</summary>
        public string Id { get; init; } = "";

        /// <summary>Display name.</summary>
        public string Name { get; init; } = "";

        /// <summary>Optional description; <c>null</c> when unset.</summary>
        public string? Description { get; init; }

        /// <summary>The image cache-buster marker, or <c>null</c> when the entity has no image.</summary>
        public string? ImageUpdatedAt { get; init; }

        /// <summary>How a collection is played (<c>arcade</c>/<c>campaign</c>); <c>null</c> for a game.</summary>
        public string? PlayMode { get; init; }

        /// <summary>True when this card is a collection (its Play resolves member access first).</summary>
        public bool IsCollection => Kind == GameEntityKind.Collection;

        /// <summary>True when the Leaderboard action applies — games have a board, collections do not.</summary>
        public bool ShowLeaderboard => Kind == GameEntityKind.Definition;

        /// <summary>The placeholder art shown until (or when there is no) uploaded image — the game vs collection glyph.</summary>
        public string PlaceholderImage => Kind == GameEntityKind.Collection ? "workshop_game_collection.png" : "workshop_game.png";

        /// <summary>The fetched thumbnail bytes, or <c>null</c> until loaded / when absent.</summary>
        [ObservableProperty]
        private byte[]? imageBytes;

        /// <summary>Projects a game definition to a card.</summary>
        /// <param name="definition">The game definition</param>
        /// <returns>The card</returns>
        public static Play3dCardItem FromDefinition(GameDefinition definition) => new()
        {
            Kind = GameEntityKind.Definition,
            Id = definition.Id,
            Name = definition.Name,
            Description = definition.Description,
            ImageUpdatedAt = definition.ImageUpdatedAt,
        };

        /// <summary>Projects a collection to a card.</summary>
        /// <param name="collection">The collection</param>
        /// <returns>The card</returns>
        public static Play3dCardItem FromCollection(GameCollection collection) => new()
        {
            Kind = GameEntityKind.Collection,
            Id = collection.Id,
            Name = collection.Name,
            Description = collection.Description,
            ImageUpdatedAt = collection.ImageUpdatedAt,
            PlayMode = collection.PlayMode,
        };

        /// <summary>
        /// Projects a featured-catalogue row to a card, dispatching on whichever
        /// hydrated payload is present. Returns <c>null</c> for an unhydrated /
        /// dangling row (neither definition nor collection), which the caller drops.
        /// </summary>
        /// <param name="item">The featured catalogue row</param>
        /// <returns>The card, or <c>null</c> when the row carries no payload</returns>
        public static Play3dCardItem? FromFeatured(FeaturedGameItem item)
        {
            if (item.Definition is not null)
                return FromDefinition(item.Definition);
            if (item.Collection is not null)
                return FromCollection(item.Collection);
            return null;
        }
    }
}
