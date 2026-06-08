using Maze.Api;

namespace Maze.Maui.App
{
    /// <summary>
    /// Per-cell editor overrides held in C# alongside the character grid, keyed by
    /// zero-based <c>(row, col)</c>. The character grid stays the cell <em>type</em>;
    /// an override layers non-default <em>characteristics</em> (a ghost enemy, a lava
    /// wall, a chest key holder, …) onto an individual cell.
    /// <para>
    /// The store mirrors the web editor's override map: a structural edit remaps the
    /// keys (insert shifts the cells past the edit point; delete drops the removed
    /// band and shifts the rest back), a rewritten cell drops its override, and the
    /// survivors are applied to the maze at save time. Only cells carrying a
    /// non-default characteristic appear here, so an all-default maze keeps an empty
    /// store and serialises as a plain character grid.
    /// </para>
    /// </summary>
    public class CellOverrides
    {
        private Dictionary<(int Row, int Col), CellEntityInfo> _map = new();

        /// <summary>Number of cells currently carrying an override.</summary>
        public int Count => _map.Count;

        /// <summary>The override on <c>(row, col)</c>, or <c>null</c> if none.</summary>
        public CellEntityInfo? Get(int row, int col) =>
            _map.TryGetValue((row, col), out CellEntityInfo? entity) ? entity : null;

        /// <summary>Whether <c>(row, col)</c> carries an override.</summary>
        public bool Has(int row, int col) => _map.ContainsKey((row, col));

        /// <summary>Sets (or replaces) the override on <c>(row, col)</c>.</summary>
        public void Set(int row, int col, CellEntityInfo entity) => _map[(row, col)] = entity;

        /// <summary>Removes the override on <c>(row, col)</c>, if any.</summary>
        public void Remove(int row, int col) => _map.Remove((row, col));

        /// <summary>Drops every override.</summary>
        public void Clear() => _map.Clear();

        /// <summary>
        /// The <c>(cell, entity)</c> pairs — e.g. to stamp each onto a maze at save.
        /// </summary>
        public IEnumerable<KeyValuePair<(int Row, int Col), CellEntityInfo>> Entries => _map;

        /// <summary>Shifts overrides at/after row <paramref name="at"/> down by <paramref name="count"/>.</summary>
        public void InsertRows(int at, int count) => Remap((r, c) => (r >= at ? r + count : r, c));

        /// <summary>Shifts overrides at/after column <paramref name="at"/> right by <paramref name="count"/>.</summary>
        public void InsertCols(int at, int count) => Remap((r, c) => (r, c >= at ? c + count : c));

        /// <summary>
        /// Drops overrides in the deleted rows <c>[at, at + count)</c> and shifts those
        /// below up by <paramref name="count"/>.
        /// </summary>
        public void DeleteRows(int at, int count) => Remap((r, c) =>
        {
            if (r >= at && r < at + count) return null;
            return r >= at + count ? (r - count, c) : (r, c);
        });

        /// <summary>
        /// Drops overrides in the deleted columns <c>[at, at + count)</c> and shifts
        /// those to the right left by <paramref name="count"/>.
        /// </summary>
        public void DeleteCols(int at, int count) => Remap((r, c) =>
        {
            if (c >= at && c < at + count) return null;
            return c >= at + count ? (r, c - count) : (r, c);
        });

        /// <summary>
        /// Rebuilds the map, mapping each key through <paramref name="remap"/>; a
        /// <c>null</c> result drops that entry. A no-op when there are no overrides.
        /// </summary>
        private void Remap(Func<int, int, (int, int)?> remap)
        {
            if (_map.Count == 0)
            {
                return;
            }
            Dictionary<(int Row, int Col), CellEntityInfo> next = new();
            foreach (KeyValuePair<(int Row, int Col), CellEntityInfo> entry in _map)
            {
                (int Row, int Col)? key = remap(entry.Key.Row, entry.Key.Col);
                if (key.HasValue)
                {
                    next[key.Value] = entry.Value;
                }
            }
            _map = next;
        }
    }
}
