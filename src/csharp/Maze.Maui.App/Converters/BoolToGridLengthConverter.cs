using System.Globalization;

namespace Maze.Maui.App.Converters
{
    /// <summary>
    /// Converts a <see cref="bool"/> to a <see cref="GridLength"/>: <c>true</c>
    /// yields the width passed as the converter parameter (absolute pixels), and
    /// <c>false</c> yields a collapsed <c>0</c> width. Used to collapse the
    /// leaderboard Player column (rather than merely hiding its content) on
    /// boards where the player is always the caller.
    /// </summary>
    public class BoolToGridLengthConverter : IValueConverter
    {
        /// <inheritdoc/>
        public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
        {
            bool show = value is bool b && b;
            if (!show)
                return new GridLength(0);
            if (parameter is string s && double.TryParse(s, NumberStyles.Any, CultureInfo.InvariantCulture, out double width))
                return new GridLength(width);
            return GridLength.Auto;
        }

        /// <inheritdoc/>
        public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture) =>
            throw new NotSupportedException();
    }
}
